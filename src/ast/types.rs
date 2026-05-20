pub type Pos = usize;

#[derive(Debug, Clone)]
pub enum BuiltIn {
    Int,
    Float,
}

#[derive(Debug, Clone)]
pub enum TypeSpecifierInner {
    BuiltIn(BuiltIn),
    Composite(String),
    Reference(Box<TypeSpecifier>),
    /// Fixed-size array type `[T; n]` (supports nested `T`).
    Array {
        elem: Box<TypeSpecifier>,
        len: usize,
    },
}

#[derive(Debug, Clone)]
pub struct TypeSpecifier {
    pub pos: Pos,
    pub inner: TypeSpecifierInner,
}
