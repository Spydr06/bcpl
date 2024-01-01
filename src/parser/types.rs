use crate::{
    ast::types::{TypeIndex, TypeKind, Type},
    token::TokenKind
};

use super::{Parser, ParseResult};

impl<'a> Parser<'a> {
    pub(super) fn parse_type(&mut self) -> ParseResult<'a, Option<TypeIndex>> {
        let loc = self.current().location().clone();
        match self.current().kind() {
            TokenKind::Ident(ident) => {
                let typ = self.type_ident(ident);
                self.advance()?;
                Ok(typ)
            }
            _ => self.unexpected(&[TokenKind::Ident("type name")])
        }
    }

    fn type_ident(&self, ident: &str) -> Option<TypeIndex> {
        self.ast.lock()
            .unwrap()
            .types()
            .by_ident(ident)
    }

    pub(super) fn get_type(&self, typ: TypeKind) -> TypeIndex {
        let mut ast = self.ast.lock().unwrap();
        let types = ast.types_mut();
        if let Some(typ) = types.by_kind(&typ) {
            return typ
        }

        types.define(Type::new(None, typ)) 
    }

    pub(super) fn get_string_type(&self) -> TypeIndex {
        self.get_type(TypeKind::Pointer(self.get_type(TypeKind::Char)))
    }
}
