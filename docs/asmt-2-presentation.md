# 实验二：函数返回类型推断 — 实现报告

## 一、实验目标

本次实验在 TeaLang 编译器中实现**函数返回类型推断**功能。当函数定义省略了返回类型标注（如 `fn abs(x: i32) { ... }`）时，编译器自动从函数体的 `return` 语句中推断出正确的返回类型。

### 核心挑战

1. **类型在注册时未知** — Pass 2（签名注册阶段）遇到省略返回类型的函数时，尚不知道返回值是 `i32` 还是 `void`
2. **跨函数依赖** — `clamp` 的返回类型依赖于 `max` 和 `min`，需要跨函数传播类型约束
3. **自递归** — `pow` 递归调用自身，返回类型取决于递归终点的返回值

---

## 二、算法设计

### 2.1 类型变量 + 约束求解

为每个省略返回类型的函数分配一个**类型变量** α_f 作为占位符。在分析函数体时，从 `return` 语句中收集约束：

- `return e;` 产生约束 **α_f = typeOf(e)**
- `return;` 产生约束 **α_f = void**

当两个约束冲突时（如 `void` 与 `i32`）立即报错。

### 2.2 并查集（Union-Find）

使用并查集维护类型变量之间的等价类关系，每个等价类的根节点携带一个可选的具体类型：

```
struct UnionFind {
    parent: Vec<TypeId>,           // 父节点指针
    rank: Vec<u32>,               // 按秩合并的秩
    concrete: Vec<Option<Dtype>>, // 根节点关联的具体类型
}
```

**核心不变式**：每个等价类至多关联一个具体类型。

**三种约束处理**：

| 情况 | 左侧 | 右侧 | 处理方式 |
|------|------|------|----------|
| ① | 具体类型 T₁ | 具体类型 T₂ | 必须 T₁ = T₂，否则报错 |
| ② | 类型变量 α | 具体类型 T | 绑定 T 到 α 所在的等价类 |
| ③ | 类型变量 α | 类型变量 β | 合并两个等价类 |

### 2.3 两阶段约束求解

将"约束收集"与"求解"拆分为两个独立阶段：

1. **Phase 1 — Seed（种子）**：扫描所有函数定义，为每个省略返回类型的函数分配类型变量 α_f
2. **Phase 2 — Collect（收集）**：遍历所有函数体，收集约束并加入全局并查集
3. **Phase 3 — Resolve（求解）**：查询每个 α_f 对应的具体类型，写回 Registry

---

## 三、编译器管线扩展

### 从三个 Pass 到 3+1 个 Pass

```
Pass 1 (use) → Pass 2 (注册签名) → [Pass 2.5: 返回类型推断] → Pass 3 (生成 IR)
```

**Pass 2.5** 是本次实验新增的阶段，在 Pass 2（函数签名以 `void` 占位注册）和 Pass 3（使用具体类型生成 IR）之间运行。它通过 Rust 的 Cargo feature `return-type-inference` 控制，默认关闭。

Pass 2.5 退出后，Registry 中所有函数返回类型均为具体类型，Pass 3 无需任何修改。

---

## 四、核心数据结构

### Ty 类型

```rust
enum Ty {
    Concrete(Dtype),  // 具体类型（i32, void 等）
    Var(TypeId),      // 类型变量（α_f）
}
```

### Collector 结构

```rust
struct Collector<'a> {
    registry: &'a Registry,
    uf: &'a mut UnionFind,
    pending: &'a HashMap<String, TypeId>,  // name → α_f 映射
    fn_name: &'a str,
    return_var: Option<TypeId>,             // 当前函数的 α_f
    env: HashMap<String, Ty>,               // 局部变量类型环境
}
```

Collector 负责遍历函数体、收集约束。其 `fork` 方法创建独立的环境副本用于处理 if/else 分支和循环体，分支结束后通过 `unify` 合并环境。

---

## 五、关键实现细节

### 5.1 unify 函数

```rust
fn unify(uf: &mut UnionFind, a: &Ty, b: &Ty, symbol: &str) -> Result<(), Error> {
    match (a, b) {
        (Ty::Concrete(x), Ty::Concrete(y)) if x == y => Ok(()),
        (Ty::Var(v), Ty::Concrete(c)) | (Ty::Concrete(c), Ty::Var(v)) => uf.bind(*v, c.clone(), symbol),
        (Ty::Var(x), Ty::Var(y)) => uf.union(*x, *y, symbol),
        _ => Err(Error::TypeMismatch { symbol: symbol.to_string(), expected: x.clone(), actual: y.clone() }),
    }
}
```

这是整个推断系统的核心。所有涉及类型相等的地方（变量定义、分支合并、return 语句）都通过 `unify` 实现。

### 5.2 自递归的解决

以 `pow` 函数为例：

```rust
fn pow(base: i32, exp: i32) {  // α_pow 待推断
    if exp == 0 { return 1; }  // 约束: α_pow = i32
    let half = pow(base, exp / 2);  // half 的类型 = α_pow
    return half * base;
}
```

关键在于 `type_of_fn_call` 对 pending 函数的处理：当调用 `pow` 时，返回类型是 `Ty::Var(α_pow)`（而非立即求解）。当 `return 1;` 处理时，约束 `α_pow = i32` 被加入，递归调用点的类型随之自动确定。

### 5.3 冲突检测（type_infer_5）

```rust
fn modular_inverse(a: i32, m: i32) {
    if g != 1 { return; }      // α = void
    return t;                   // α = i32  → 冲突！
}
```

两条约束进入同一个等价类，`bind(void)` 后再 `bind(i32)` 在并查集中立即检测到类型冲突，抛出 `TypeMismatch` 错误。

---

## 六、测试结果

### 6.1 测试用例覆盖

| 测试用例 | 验证内容 |
|----------|----------|
| `type_infer_basic` | 基线回归，所有函数有显式返回类型 |
| `type_infer_1` | 线性跨函数链：`abs → max → min → clamp → clamp_positive` |
| `type_infer_2` | 自递归：`pow` 内部调用自身 |
| `type_infer_3` | 引用参数 `&[i32]` 与推断返回类型共存 |
| `type_infer_4` | 三级跨函数链：`is_prime → next_prime → nth_prime` |
| `type_infer_5` | 负测试：`return;` 与 `return t;` 冲突检测 |

### 6.2 运行结果

```bash
# 主线测试（feature 关闭）
$ cargo test
test result: ok. 30 passed; 0 failed

# 完整测试（feature 开启）
$ cargo test --features return-type-inference
test result: ok. 35 passed; 0 failed
```

所有 35 个测试全部通过，包括 5 个正测试（推断正确类型）和 1 个负测试（检测到类型冲突）。

---

## 七、代码修改位置

本次实验**仅修改了 `src/experimental/return_infer.rs`**，共实现 10 处 `todo!()`：

1. `UnionFind::bind` — 绑定具体类型到等价类
2. `UnionFind::union` — 合并两个等价类
3. `UnionFind::resolve` — 查询等价类的具体类型
4. `unify` — 统一两个类型
5. `process_var_def` — 处理变量定义
6. `merge_branches` — 合并 if/else 分支
7. `merge_with_body` — 合并循环体
8. `process_return` — 处理 return 语句
9. `type_of_fn_call` — 函数调用类型
10. `resolve_return_types`（Phase 2+3）— 约束收集与求解

---

## 八、总结

本次实验实现了 TeaLang 编译器的函数返回类型推断功能，通过引入**类型变量**和**并查集约束求解**机制，在编译期自动确定省略返回类型的函数的实际返回类型。实现的核心思想是将约束收集与求解分离为两个阶段，使得跨函数依赖、自递归和类型冲突检测都能通过统一的 `unify` 操作自然处理，无需特殊的执行顺序假设。
