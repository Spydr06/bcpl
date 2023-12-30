use std::collections::HashMap;

use crate::{source_file::Location, token::TokenKind};

pub type TypeIndex = u32;

#[derive(Debug)]
pub struct Type {
    loc: Option<Location>,
    size: u32,

    is_builtin: bool,
    kind: TypeKind
}

impl Type {
    fn new_builtin(kind: TypeKind, size: u32) -> Self {
        Self {
            loc: None,
            size,
            is_builtin: true,
            kind
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind {
    UInt8,
    UInt16,
    UInt32,
    UInt64,

    Int8,
    Int16,
    Int32,
    Int64,

    Float32,
    Float64,

    Bool,
    Char,
    Unit,

    Atom,

    Pointer(TypeIndex),
    Array(TypeIndex, usize),

    // Table
    // Function
    // Generic
    // Record
    // ...
}

const BUILTIN_TYPE_KINDS: [(TypeKind, u32); 14] = [
    (TypeKind::UInt8, 1),
    (TypeKind::UInt16, 2),
    (TypeKind::UInt32, 4),
    (TypeKind::UInt64, 8),
    (TypeKind::Int8, 1),
    (TypeKind::Int16, 2),
    (TypeKind::Int32, 4),
    (TypeKind::Int64, 8),
    (TypeKind::Float32, 4),
    (TypeKind::Float64, 8),
    (TypeKind::Bool, 1),
    (TypeKind::Char, 1),
    (TypeKind::Unit, 0),
    (TypeKind::Atom, 4)
];

impl TryFrom<&str> for TypeKind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "UInt8" => Ok(Self::UInt8),
            "UInt16" => Ok(Self::UInt16),
            "UInt32" => Ok(Self::UInt32),
            "UInt64" => Ok(Self::UInt64),
            "Int8" => Ok(Self::Int8),
            "Int16" => Ok(Self::Int16),
            "Int32" => Ok(Self::Int32),
            "Int64" => Ok(Self::Int64),
            "Float32" => Ok(Self::Float32),
            "Float64" => Ok(Self::Float64),
            "Bool" => Ok(Self::Bool),
            "Char" => Ok(Self::Char),
            "Unit" => Ok(Self::Unit),
            "Atom" => Ok(Self::Atom),
            _ => Err(())
        }
    }
}

#[derive(Debug)]
pub struct TypeList {
    types: HashMap<TypeIndex, Type>
}

impl TypeList {
    pub fn by_ident(&self, ident: &str) -> Option<TypeIndex> {
        self.by_kind(TypeKind::try_from(ident).ok()?)
    }

    pub fn by_kind(&self, kind: TypeKind) -> Option<TypeIndex> {
        self.types.iter()
            .find(|(_, typ)| typ.kind == kind)
            .map(|(i, _)| *i)
    }
}

impl Default for TypeList {
    fn default() -> Self {
        Self {
            types: BUILTIN_TYPE_KINDS.iter()
                .enumerate()
                .map(|(i, (kind, size))| ((i + 1) as TypeIndex, Type::new_builtin(kind.clone(), *size)))
                .collect()
        }
    }
}
