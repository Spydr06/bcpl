use std::collections::HashMap;

use crate::source_file::Location;

pub type TypeIndex = u32;

#[derive(Debug)]
pub struct Type {
    loc: Option<Location>,
    size: u32,

    is_builtin: bool,
    kind: TypeKind
}

impl Type {
    pub fn new(loc: Option<Location>, kind: TypeKind) -> Self {
        Self {
            loc,
            size: kind.try_get_size().unwrap_or(0),
            is_builtin: false,
            kind
        }
    }

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

impl TypeKind {
    pub fn try_get_size(&self) -> Option<u32> {
        match self {
            TypeKind::Unit => Some(0),
            TypeKind::UInt8 | TypeKind::Int8 | TypeKind::Char | TypeKind::Bool => Some(1),
            TypeKind::UInt16 | TypeKind::Int16 => Some(2),
            TypeKind::UInt32 | TypeKind::Int32 | TypeKind::Float32 | TypeKind::Atom => Some(4),
            TypeKind::UInt64 | TypeKind::Int64 => Some(8),
            TypeKind::Pointer(_) => Some(std::mem::size_of::<*const ()>() as u32), // TODO: handle crosscompilation
            _ => None,
        } 
    }
}

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

#[derive(Debug, Default)]
pub struct TypeList {
    types: Vec<Type>
}

impl TypeList {
    pub fn by_ident(&self, ident: &str) -> Option<TypeIndex> {
        self.by_kind(&TypeKind::try_from(ident).ok()?)
    }

    pub fn by_kind(&self, kind: &TypeKind) -> Option<TypeIndex> {
        self.types.iter()
            .enumerate()
            .find(|(_, typ)| &typ.kind == kind)
            .map(|(i, _)| i as u32)
    }

    pub fn define(&mut self, typ: Type) -> TypeIndex {
        self.types.push(typ);
        self.types.len() as u32 - 1
    }
}

