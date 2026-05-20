use super::expr::{RightVal, RightValList};
use super::stmt::CodeBlockStmtList;
use super::types::TypeSpecifier;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct VarDeclArray {
    pub len: usize,
}

#[derive(Debug, Clone)]
pub enum VarDeclInner {
    Scalar,
    Array(Box<VarDeclArray>),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub identifier: String,
    pub type_specifier: Option<TypeSpecifier>,
    pub inner: VarDeclInner,
}

pub type VarDeclList = Vec<VarDecl>;

#[derive(Debug, Clone)]
pub struct VarDefScalar {
    pub val: Box<RightVal>,
}

#[derive(Debug, Clone)]
pub enum ArrayInitializer {
    ExplicitList(RightValList),
    Fill { val: Box<RightVal>, count: usize },
}

#[derive(Debug, Clone)]
pub struct VarDefArray {
    pub len: usize,
    pub initializer: ArrayInitializer,
}

#[derive(Debug, Clone)]
pub enum VarDefInner {
    Scalar(Box<VarDefScalar>),
    Array(Box<VarDefArray>),
}

#[derive(Debug, Clone)]
pub struct VarDef {
    pub identifier: String,
    pub type_specifier: Option<TypeSpecifier>,
    pub inner: VarDefInner,
}

#[derive(Debug, Clone)]
pub enum VarDeclStmtInner {
    Decl(Box<VarDecl>),
    Def(Box<VarDef>),
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt {
    pub inner: VarDeclStmtInner,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub identifier: String,
    pub decls: VarDeclList,
}

/// `&self` / `&mut self` as the first parameter in a method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfParam {
    Ref,
    RefMut,
}

#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub self_param: Option<SelfParam>,
    pub decls: VarDeclList,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub identifier: String,
    pub param_decl: Option<Box<ParamDecl>>,
    pub return_dtype: Option<TypeSpecifier>,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub fn_decl: Box<FnDecl>,
    pub stmts: CodeBlockStmtList,
}

#[derive(Debug, Clone)]
pub struct FnDeclStmt {
    pub fn_decl: Box<FnDecl>,
}

impl Deref for FnDeclStmt {
    type Target = FnDecl;

    fn deref(&self) -> &Self::Target {
        &self.fn_decl
    }
}
