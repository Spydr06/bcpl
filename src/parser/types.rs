use crate::{
    ast::types::{TypeIndex, TypeKind, Type, SumVariant},
    token::TokenKind, source_file::WithLocation
};

use super::{Parser, ParseResult, ParseError, stmt::StmtContext};

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
                let ident = ident.to_string();
                self.advance()?;
                if [TokenKind::LParen, TokenKind::Colon].contains(self.current().kind()) {
                    self.parse_sum_type(ident)
                }
                else {
                    Ok(self.type_ident(ident))
                }
            }
            TokenKind::LParen => {
                self.advance()?;
                let typ = self.parse_type()?;
                self.expect(&[TokenKind::RParen])?;
                Ok(typ)
            },
            TokenKind::LBracket => self.parse_array_type(),
            TokenKind::LogAnd => {
                self.advance()?;
                let inner_typ = self.parse_type()?;
                Ok(self.pointer_to(inner_typ))
            },
            _ => self.unexpected(&[TokenKind::Ident("type name")])
        }
    }

    fn parse_type_param(&mut self, _: &()) -> ParseResult<'a, TypeIndex> {
        self.parse_type()
    }

    fn parse_sum_variant(&mut self, ident: String) -> ParseResult<'a, SumVariant> {
        Ok(SumVariant::Basic(
            ident,
            self.parse_optional_list(TokenKind::LParen, TokenKind::RParen, TokenKind::Comma, Self::parse_type_param, &())?
        ))
    }

    fn parse_sum_type(&mut self, first: String) -> ParseResult<'a, TypeIndex> {
        let mut variants = vec![self.parse_sum_variant(first)?];

        while let TokenKind::Colon = self.current().kind() {
            self.advance()?;

            let ident = self.expect_ident()?;
            variants.push(self.parse_sum_variant(ident)?);
        }

        Ok(self.get_type(TypeKind::Sum(variants)))
    }

    fn type_ident(&self, ident: String) -> TypeIndex {
        let mut ast = self.ast.lock().unwrap(); 
        let types = ast.types_mut();
        if let Some(typ) = types.builtin_by_ident(&ident) {
            typ
        }
        else if let Some(typ) = types.find_alias(&ident) {
            typ
        }
        else {
            types.define(Type::new(None, TypeKind::Alias(ident.to_string(), None)))
        }
    }

    fn parse_array_type(&mut self) -> ParseResult<'a, TypeIndex> {
        self.advance()?;
        let inner = self.parse_type()?;
        if let TokenKind::Comma = self.expect(&[TokenKind::RBracket, TokenKind::Comma])?.kind() {
            let mut expr = self.parse_expr(&StmtContext::Empty)?;
            self.expect(&[TokenKind::RBracket])?;
            
            let index_type = self.get_type(TypeKind::UInt64);
            if expr.typ() != &Some(index_type) {
                expr = expr.implicit_cast(index_type)
            }

            Ok(self.get_type(TypeKind::Array(inner, Box::new(expr))))
        }
        else {
            Ok(self.get_type(TypeKind::Slice(inner)))
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
