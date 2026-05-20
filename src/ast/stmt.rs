use super::decl::VarDeclStmt;
use super::expr::{ArithExpr, BoolUnit, FnCall, LeftVal, RightVal};

#[derive(Debug, Clone)]
pub struct AssignmentStmt {
    pub left_val: Box<LeftVal>,
    pub right_val: Box<RightVal>,
}

#[derive(Debug, Clone)]
pub struct CallStmt {
    pub fn_call: Box<FnCall>,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub val: Option<Box<RightVal>>,
}

#[derive(Debug, Clone)]
pub struct ContinueStmt {}

#[derive(Debug, Clone)]
pub struct BreakStmt {}

#[derive(Debug, Clone)]
pub struct NullStmt {}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub bool_unit: Box<BoolUnit>,
    pub if_stmts: CodeBlockStmtList,
    pub else_stmts: Option<CodeBlockStmtList>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub bool_unit: Box<BoolUnit>,
    pub stmts: CodeBlockStmtList,
}

#[derive(Debug, Clone)]
pub enum RangeBound {
    Arith(Box<ArithExpr>),
    FnCall(Box<FnCall>),
    Float(f32),
    Num(i32),
    Id(String),
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub iter_var: String,
    pub start: Box<RangeBound>,
    pub end: Box<RangeBound>,
    pub stmts: CodeBlockStmtList,
}

#[derive(Debug, Clone)]
pub enum CodeBlockStmtInner {
    VarDecl(Box<VarDeclStmt>),
    Assignment(Box<AssignmentStmt>),
    Call(Box<CallStmt>),
    If(Box<IfStmt>),
    While(Box<WhileStmt>),
    For(Box<ForStmt>),
    Return(Box<ReturnStmt>),
    Continue(Box<ContinueStmt>),
    Break(Box<BreakStmt>),
    Null(Box<NullStmt>),
}

#[derive(Debug, Clone)]
pub struct CodeBlockStmt {
    pub inner: CodeBlockStmtInner,
}

pub type CodeBlockStmtList = Vec<CodeBlockStmt>;
