use crate::ast;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, PartialEq, Debug)]
pub enum Dtype {
    Void,
    I1,
    I32,
    F32,
    Struct {
        type_name: String,
    },
    Pointer {
        pointee: Box<Dtype>,
    },
    Array {
        element: Box<Dtype>,
        length: Option<usize>,
    },
    Undecided,
}

impl Dtype {
    pub fn ptr_to(pointee: Self) -> Self {
        Self::Pointer {
            pointee: Box::new(pointee),
        }
    }

    pub fn array_of(elem: Self, len: usize) -> Self {
        Self::Array {
            element: Box::new(elem),
            length: Some(len),
        }
    }

    pub fn struct_type_name(&self) -> Option<&String> {
        match self {
            Dtype::Struct { type_name } => Some(type_name),
            Dtype::Pointer { pointee } => pointee.struct_type_name(),
            Dtype::Array { element, .. } => element.struct_type_name(),
            _ => None,
        }
    }
}

impl Display for Dtype {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Dtype::I1 => write!(f, "i1"),
            Dtype::I32 => write!(f, "i32"),
            Dtype::F32 => write!(f, "f32"),
            Dtype::Void => write!(f, "void"),
            Dtype::Struct { type_name } => write!(f, "%{}", type_name),
            Dtype::Pointer { .. } => write!(f, "ptr"),
            Dtype::Array {
                element,
                length: Some(length),
            } => write!(f, "[{} x {}]", length, element.as_ref()),
            Dtype::Array {
                element,
                length: None,
            } => write!(f, "{}", element.as_ref()),
            Dtype::Undecided => write!(f, "?"),
        }
    }
}

pub struct StructMember {
    pub offset: i32,
    pub dtype: Dtype,
}

pub struct StructType {
    pub elements: Vec<(String, StructMember)>,
}

#[derive(Clone, PartialEq)]
pub struct FunctionType {
    pub return_dtype: Dtype,
    pub arguments: Vec<(String, Dtype)>,
}

impl PartialEq<ast::FnDecl> for FunctionType {
    fn eq(&self, rhs: &ast::FnDecl) -> bool {
        let rhs_dtype = match rhs.return_dtype.as_ref().map(Dtype::from) {
            Some(dtype) => dtype,
            None => Dtype::Void,
        };

        if self.return_dtype != rhs_dtype {
            return false;
        }

        let mut rhs_args = Vec::new();
        if let Some(params) = &rhs.param_decl {
            for decl in params.decls.iter() {
                let identifier = decl.identifier.clone();
                let dtype = match Dtype::try_from(decl) {
                    Ok(t) => t,
                    Err(_) => return false,
                };

                rhs_args.push((identifier, dtype));
            }
        }

        let num_args = self.arguments.len();
        if rhs_args.len() != num_args {
            return false;
        }

        for ((lhs_id, lhs_dtype), (rhs_id, rhs_dtype)) in self.arguments.iter().zip(rhs_args) {
            if lhs_id != &rhs_id || lhs_dtype != &rhs_dtype {
                return false;
            }
        }

        true
    }
}
