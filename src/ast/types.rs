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

    fn new_builtin(kind: TypeKind) -> Self {
        Self {
            loc: None,
            size: kind.try_get_size().expect(&format!("internal error when initializing builtin type {kind:?}")),
            is_builtin: true,
            kind
        }
    }

    pub fn location(&self) -> &Option<Location> {
        &self.loc
    }

    pub fn set_location(&mut self, loc: Location) {
        self.loc = Some(loc);
    }

    pub fn set_kind(&mut self, kind: TypeKind) {
        self.kind = kind;
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
    Slice(TypeIndex),

    Alias(String, Option<TypeIndex>),

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
            TypeKind::UInt64 | TypeKind::Int64 | TypeKind::Float64 => Some(8),
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

const BUILTIN_TYPE_KINDS: [TypeKind; 14] = [
    TypeKind::UInt8,
    TypeKind::UInt16,
    TypeKind::UInt32,
    TypeKind::UInt64,
    TypeKind::Int8,
    TypeKind::Int16,
    TypeKind::Int32,
    TypeKind::Int64,
    TypeKind::Float32,
    TypeKind::Float64,
    TypeKind::Bool,
    TypeKind::Char,
    TypeKind::Unit,
    TypeKind::Atom
];

#[derive(Debug)]
pub struct TypeList {
    types: Vec<Type>
}

impl TypeList {
    pub fn builtin_by_ident(&self, ident: &str) -> Option<TypeIndex> {
        self.by_kind(&TypeKind::try_from(ident).ok()?)
    }

    pub fn find_alias(&self, ident: &str) -> Option<TypeIndex> {
        self.types.iter()
            .enumerate()
            .find_map(|(i, typ)| match &typ.kind {
                TypeKind::Alias(alias, _) if alias == ident => Some(i as u32),
                _ => None
            })
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

    pub fn get_mut(&mut self, index: TypeIndex) -> Option<&mut Type> {
        self.types.get_mut(index as usize)
    }
}

impl Default for TypeList {
    fn default() -> Self {
        Self {
            types: BUILTIN_TYPE_KINDS.into_iter()
                .map(Type::new_builtin)
                .collect()
        }
    }
}
