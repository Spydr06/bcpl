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

#[derive(Debug)]
pub struct TypeList {
    types: HashMap<TypeIndex, Type>,
    next_type_index: TypeIndex,
}

impl TypeList {
    pub fn by_ident(&self, ident: &str) -> Option<TypeIndex> {
        self.by_kind(&TypeKind::try_from(ident).ok()?)
    }

    pub fn by_kind(&self, kind: &TypeKind) -> Option<TypeIndex> {
        self.types.iter()
            .find(|(_, typ)| &typ.kind == kind)
            .map(|(i, _)| *i)
    }

    pub fn define(&mut self, typ: Type) -> TypeIndex {
        self.next_type_index += 1;
        self.types.insert(self.next_type_index, typ);
        self.next_type_index
    }
}

const CUSTOM_TYPE_INDEX_START: TypeIndex = 1000;

impl Default for TypeList {
    fn default() -> Self {
        Self {
            types: BUILTIN_TYPE_KINDS.iter()
                .enumerate()
                .map(|(i, (kind, size))| ((i + 1) as TypeIndex, Type::new_builtin(kind.clone(), *size)))
                .collect(),
            next_type_index: CUSTOM_TYPE_INDEX_START
        }
    }
}
