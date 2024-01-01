use crate::{
    ast::types::{TypeIndex, TypeKind, Type},
    token::TokenKind, source_file::WithLocation
};

use super::{Parser, ParseResult, ParseError};

impl<'a> Parser<'a> {
    pub(super) fn parse_type_alias(&mut self) -> ParseResult<'a, ()> {
        self.expect(&[TokenKind::Type])?;
        let loc = self.current().location().clone();
        let alias = self.expect_ident()?; 

        self.expect(&[TokenKind::Eq])?;

        let typ = self.parse_type()?;
        let mut ast = self.ast.lock().unwrap();
        if let Some(id) = ast.types().find_alias(&alias) {
            let existing = ast.types_mut().get_mut(id).unwrap();
            if let Some(existing_loc) = existing.location() {
                Err(ParseError::Redefinition(existing_loc.clone(), alias).with_location(loc))
            }
            else {
                existing.set_location(loc);
                existing.set_kind(TypeKind::Alias(alias, Some(typ)));
                Ok(())
            }
        }
        else {
            ast.types_mut().define(Type::new(Some(loc), TypeKind::Alias(alias, Some(typ))));
            Ok(())
        }
    }

    pub(super) fn parse_type(&mut self) -> ParseResult<'a, TypeIndex> {
        match self.current().kind().clone() {
            TokenKind::Ident(ident) => {
                let typ = self.type_ident(ident);
                self.advance()?;
                Ok(typ)
            }
            TokenKind::LParen => {
                self.advance()?;
                let typ = self.parse_type()?;
                self.expect(&[TokenKind::RParen])?;
                Ok(typ)
            },
            TokenKind::LogAnd => {
                self.advance()?;
                let inner_typ = self.parse_type()?;
                Ok(self.pointer_to(inner_typ))
            },
            _ => self.unexpected(&[TokenKind::Ident("type name")])
        }
    }

    fn type_ident(&self, ident: &str) -> TypeIndex {
        let mut ast = self.ast.lock().unwrap(); 
        let types = ast.types_mut();
        if let Some(typ) = types.builtin_by_ident(ident) {
            typ
        }
        else if let Some(typ) = types.find_alias(ident) {
            typ
        }
        else {
            types.define(Type::new(None, TypeKind::Alias(ident.to_string(), None)))
        }
    }

    pub(super) fn get_type(&self, typ: TypeKind) -> TypeIndex {
        let mut ast = self.ast.lock().unwrap();
        let types = ast.types_mut();
        if let Some(typ) = types.by_kind(&typ) {
            return typ
        }

        types.define(Type::new(None, typ)) 
    }

    pub(super) fn pointer_to(&self, typ: TypeIndex) -> TypeIndex {
        self.get_type(TypeKind::Pointer(typ))
    }

    pub(super) fn get_string_type(&self) -> TypeIndex {
        self.pointer_to(self.get_type(TypeKind::Char))
    }
}
