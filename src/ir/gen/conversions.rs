//! Conversions between AST types and IR types.
//!
//! This module provides trait implementations and helper functions to convert
//! AST-level type representations (`ast::TypeSpecifier`, `ast::VarDecl`,
//! `ast::VarDef`, `ast::VarDeclStmt`) into their corresponding IR-level
//! data types (`Dtype`).

use crate::ast;
use crate::ir::types::Dtype;

/// Converts an optional AST type specifier into the corresponding base IR data type (`Dtype`).
///
/// - `Composite` type specifiers (e.g., user-defined structs) map to `Dtype::Struct`.
/// - `Reference` type specifiers (e.g., `&[T]`) map to a pointer to an unsized array,
///   where the element type is resolved recursively.
/// - `BuiltIn` type specifiers (e.g., `i32`) and `None` (absent specifier) both default
///   to `Dtype::I32`.
fn base_dtype(type_specifier: &Option<ast::TypeSpecifier>) -> Dtype {
    match type_specifier.as_ref().map(|t| &t.inner) {
        Some(ast::TypeSpecifierInner::Composite(name)) => Dtype::Struct {
            type_name: name.to_string(),
        },
        Some(ast::TypeSpecifierInner::Reference(inner)) => Dtype::ptr_to(Dtype::Array {
            element: Box::new(base_dtype(&Some(inner.as_ref().clone()))),
            length: None,
        }),
        Some(ast::TypeSpecifierInner::Array { elem, len }) => {
            Dtype::array_of(Dtype::from(elem.as_ref()), *len)
        }
        Some(ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Float)) => Dtype::F32,
        Some(ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Int)) | None => Dtype::I32,
    }
}

/// Combines a scalar element type with an AST declaration shape (scalar vs. array)
/// to produce the storage [`Dtype`] for that declaration.
///
/// Shared by the [`Dtype::try_from<&VarDecl>`] path (for globals & parameters)
/// and by `plan_local_decl_storage` in the function generator, so that the
/// "wrap with Array if the declaration is array-shaped" rule lives in exactly
/// one place.
pub(crate) fn compose_var_decl_dtype(base: Dtype, inner: &ast::VarDeclInner) -> Dtype {
    match inner {
        ast::VarDeclInner::Scalar => base,
        ast::VarDeclInner::Array(arr) => Dtype::array_of(base, arr.len),
    }
}

/// Analogue of [`compose_var_decl_dtype`] for [`ast::VarDef`]s.
pub(crate) fn compose_var_def_dtype(base: Dtype, inner: &ast::VarDefInner) -> Dtype {
    match inner {
        ast::VarDefInner::Scalar(_) => base,
        ast::VarDefInner::Array(arr) => Dtype::array_of(base, arr.len),
    }
}

// ---------------------------------------------------------------------------
// `From` trait implementations: AST TypeSpecifier -> IR Dtype
// ---------------------------------------------------------------------------
//
// These provide infallible conversions from AST type specifiers to IR types.

/// Converts an owned `ast::TypeSpecifier` into a `Dtype` by delegating to the
/// by-reference implementation.
impl From<ast::TypeSpecifier> for Dtype {
    fn from(a: ast::TypeSpecifier) -> Self {
        Self::from(&a)
    }
}

/// Converts a reference to an `ast::TypeSpecifier` into the corresponding `Dtype`.
///
/// - `BuiltIn` maps to `Dtype::I32` (the only built-in type is `i32`).
/// - `Composite` maps to `Dtype::Struct` with the user-defined type name.
/// - `Reference` maps to a pointer to an unsized array whose element type
///   is recursively converted from the inner type specifier.
impl From<&ast::TypeSpecifier> for Dtype {
    fn from(a: &ast::TypeSpecifier) -> Self {
        match &a.inner {
            ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Int) => Self::I32,
            ast::TypeSpecifierInner::BuiltIn(ast::BuiltIn::Float) => Self::F32,
            ast::TypeSpecifierInner::Composite(name) => Self::Struct {
                type_name: name.clone(),
            },
            ast::TypeSpecifierInner::Reference(inner) => Self::ptr_to(Dtype::Array {
                element: Box::new(Self::from(inner.as_ref())),
                length: None,
            }),
            ast::TypeSpecifierInner::Array { elem, len } => {
                Self::array_of(Self::from(elem.as_ref()), *len)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// `TryFrom` trait implementations: AST declarations -> IR Dtype
// ---------------------------------------------------------------------------
//
// These are fallible conversions because certain combinations (e.g., struct
// definitions with initializers) are not supported and produce an error.

/// Converts a variable declaration (`VarDecl`) to its IR data type.
///
/// First resolves the base type from the optional type specifier, then wraps it
/// in an array type if the declaration is for an array (with a known length),
/// or returns the base type directly for scalar declarations.
impl TryFrom<&ast::VarDecl> for Dtype {
    type Error = crate::ir::Error;

    fn try_from(decl: &ast::VarDecl) -> Result<Self, Self::Error> {
        let base = base_dtype(&decl.type_specifier);
        Ok(compose_var_decl_dtype(base, &decl.inner))
    }
}

/// Converts a variable definition (`VarDef`) to its IR data type.
///
/// Similar to the `VarDecl` conversion, but additionally rejects struct types
/// with initializers—struct variables cannot be initialized inline, so
/// attempting to do so returns `Error::StructInitialization`.
impl TryFrom<&ast::VarDef> for Dtype {
    type Error = crate::ir::Error;

    fn try_from(def: &ast::VarDef) -> Result<Self, Self::Error> {
        let base = base_dtype(&def.type_specifier);
        if matches!(&base, Dtype::Struct { .. }) {
            return Err(crate::ir::Error::StructInitialization);
        }
        Ok(compose_var_def_dtype(base, &def.inner))
    }
}

/// Converts a variable declaration statement (`VarDeclStmt`) to its IR data type.
///
/// Delegates to the `TryFrom<&VarDecl>` or `TryFrom<&VarDef>` implementation
/// depending on whether the statement is a pure declaration or a definition.
impl TryFrom<&ast::VarDeclStmt> for Dtype {
    type Error = crate::ir::Error;

    fn try_from(value: &ast::VarDeclStmt) -> Result<Self, Self::Error> {
        match &value.inner {
            ast::VarDeclStmtInner::Decl(d) => Dtype::try_from(d.as_ref()),
            ast::VarDeclStmtInner::Def(d) => Dtype::try_from(d.as_ref()),
        }
    }
}
