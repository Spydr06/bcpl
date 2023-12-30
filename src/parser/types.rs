use crate::{
    ast::types::TypeIndex,
    token::TokenKind,
    source_file::{Located, WithLocation}
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
}
