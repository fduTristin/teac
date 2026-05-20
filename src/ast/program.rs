use super::decl::{FnDeclStmt, FnDef, StructDef, VarDeclStmt};

#[derive(Debug, Clone)]
pub struct UseStmt {
    pub module_name: String,
}

#[derive(Debug, Clone)]
pub struct ImplDef {
    pub type_name: String,
    pub fns: Vec<FnDef>,
}

#[derive(Debug, Clone)]
pub enum ProgramElementInner {
    VarDeclStmt(Box<VarDeclStmt>),
    StructDef(Box<StructDef>),
    FnDeclStmt(Box<FnDeclStmt>),
    FnDef(Box<FnDef>),
    ImplDef(Box<ImplDef>),
}

#[derive(Debug, Clone)]
pub struct ProgramElement {
    pub inner: ProgramElementInner,
}

pub type ProgramElementList = Vec<ProgramElement>;

#[derive(Debug, Clone)]
pub struct Program {
    pub use_stmts: Vec<UseStmt>,
    pub elements: ProgramElementList,
}
