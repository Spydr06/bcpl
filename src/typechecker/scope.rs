use std::{collections::HashMap, rc::Rc};

use crate::ast::{Decl, types::{TypeIndex, TypeKind}, self};

#[derive(Debug)]
pub struct Scope<'a> {
    bindings: HashMap<&'a String, &'a dyn Decl>,
    types: HashMap<&'a String, TypeIndex>,

    outer: Option<&'a Scope<'a>>
}

impl<'a> Scope<'a> {
    pub fn new(outer: Option<&'a Scope<'a>>) -> Self {
        Self {
            bindings: HashMap::new(),
            types: HashMap::new(),
            outer
        }
    }

    pub fn toplevel(ast: &'a ast::Program) -> Self {
        Self {
            types: ast.types().iter().enumerate().filter_map(|(idx, typ)| match typ.kind() {
                TypeKind::Alias(id, _) => Some((id, idx as u32)),
                _ => None 
            }).collect(),
            bindings: HashMap::new(),
            outer: None
        }
    }
}
