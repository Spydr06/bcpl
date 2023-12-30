use crate::{
    token::TokenKind, 
    source_file::{Location, Located, WithLocation}, 
    ast::{Decl, Function, Program, FunctionBody, Param, IntoDecl, Section}
};

use super::{Parser, ParseResult, ParseError};

impl<'a> Parser<'a> {
    pub(super) fn parse_section(&mut self) -> ParseResult<'a, ()> {
        let section_loc = self.current_token.location().clone();
        self.expect(&[TokenKind::Section])?;

        let mut section = Section::new(self.expect_ident()?.into(), section_loc);

        let mut had_decls = false;
        loop {
            match self.current().kind() {
                TokenKind::Eof | TokenKind::Section => break,
                TokenKind::Require => {
                    if had_decls {
                        self.push_warning(ParseError::RequireAfterDecl.with_location(self.current_token.location().clone()));
                    }
                    section.add_require(self.parse_require()?);
                }
                _ => {
                    let decl = self.parse_decl()?;
                    if let Some(prev) = section.defines(decl.ident()) {
                        return Err(ParseError::Redefinition(prev.location().clone(), decl.ident().clone()).with_location(decl.location().clone()))
                    }
                    section.declare(decl);
                    had_decls = true;
                }
            }
        }

        self.ast.lock().unwrap().add_section(section);
        Ok(())
    }

    pub(super) fn parse_require(&mut self) -> ParseResult<'a, Located<String>> {
        let loc = self.current_token.location().clone();
        self.expect(&[TokenKind::Require])?;

        Ok(self.expect_ident()?.to_string().with_location(loc))
    }

    pub(super) fn parse_decl(&mut self) -> ParseResult<'a, Box<dyn Decl>> {
        let loc = self.current_token.location().clone();
        let decl_tok = self.expect(&[TokenKind::Let, TokenKind::And, TokenKind::Global, TokenKind::Manifest, TokenKind::Static])?;
        match decl_tok.kind() {
            TokenKind::Let => self.parse_function_decl(loc, false).map(Function::into_decl),
            TokenKind::And => self.parse_function_decl(loc, true).map(Function::into_decl),
            _ => unreachable!()
        }
    }

    pub(super) fn parse_function_decl(&mut self, decl_loc: Location, tailcall_recursive: bool) -> ParseResult<'a, Function> {
        let ident = self.expect_ident()?;
        
        let params = self.parse_optional_list(TokenKind::LParen, TokenKind::RParen, TokenKind::Comma, Self::parse_function_param)?;

        let func = Function::new(decl_loc, ident, params, 0, tailcall_recursive, FunctionBody::Nothing);
        Ok(func)
    }

    pub(super) fn parse_function_param(&mut self) -> ParseResult<'a, Param> {
        let loc = self.current_token.location().clone();
        let ident = self.expect_ident()?;
        
        let typ = if self.advance_if(&[TokenKind::Of])?.is_some() {
            Some(self.parse_type()?)
        }
        else {
            None
        };

        let value = if self.advance_if(&[TokenKind::Eq])?.is_some() {
            Some(self.parse_expr()?)
        }
        else {
            None
        };

        match (typ, value) {
            (None, None) => Err(
                    ParseError::Generic("Parameter requires either a type or default value.".into())
                        .with_location(loc)
                ),
            (Some(typ), Some(value)) => {
                if value.typ() != &typ && typ.is_some() {
                    Ok(Param::new(loc, ident, typ, Some(value.implicit_cast(typ.unwrap())))) 
                }
                else {
                    Ok(Param::new(loc, ident, typ, Some(value)))
                }
            }
            (_, value) => Ok(Param::new(loc, ident, typ.flatten(), value))
        }
    }
}

