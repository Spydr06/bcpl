use std::cell::RefCell;

use crate::{
    token::TokenKind, 
    source_file::{Location, Located, WithLocation}, 
    ast::{Decl, Function, FunctionBody, Param, IntoDecl, Section, types::TypeKind, BasicFunctionBody, pattern::Pattern}
};

use super::{Parser, ParseResult, ParseError, stmt::StmtContext, pattern};

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
                TokenKind::Type => {
                    had_decls = true;
                    self.parse_type_alias()?;
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
        
        let params = self.parse_optional_list(TokenKind::LParen, TokenKind::RParen, TokenKind::Comma, Self::parse_function_param, &())?;

        let context = StmtContext::Function(&params);
        let body = if self.current().kind() == &TokenKind::Colon {
            self.parse_pattern_matched_body(&context)? 
        }
        else {
            self.parse_function_body(&context)?.into() 
        };
        
        let return_type = self.get_return_type(&body);
        Ok(Function::new(decl_loc, ident, params, return_type, tailcall_recursive, body))
    }

    fn parse_function_param(&mut self, _: &()) -> ParseResult<'a, Param> {
        let loc = self.current_token.location().clone();
        let ident = self.expect_ident()?;
        
        let typ = if self.advance_if(&[TokenKind::Of])?.is_some() {
            Some(self.parse_type()?)
        }
        else {
            None
        };

        let value = if self.advance_if(&[TokenKind::Eq])?.is_some() {
            Some(self.parse_expr(&mut StmtContext::Empty)?)
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
                if value.typ() != &Some(typ) {
                    Ok(Param::new(loc, ident, Some(typ), Some(value.implicit_cast(typ)))) 
                }
                else {
                    Ok(Param::new(loc, ident, Some(typ), Some(value)))
                }
            }
            (_, value) => Ok(Param::new(loc, ident, typ, value))
        }
    }

    fn parse_function_body(&mut self, context: &StmtContext) -> ParseResult<'a, BasicFunctionBody> {
        Ok(if let TokenKind::Eq = self.expect(&[TokenKind::Eq, TokenKind::Be])?.kind() {
            let expr = self.parse_expr(context)?;
            self.advance_if(&[TokenKind::Semicolon])?;
            BasicFunctionBody::Expr(expr)
        }
        else {
            BasicFunctionBody::Stmt(self.parse_stmt(context)?)
        })
    }

    fn check_correct_pattern_length(&self, patterns: &Vec<Located<Pattern>>, num_params: usize) -> ParseResult<'a, ()> {
        (patterns.len() == num_params).then(|| ())
            .ok_or_else(|| ParseError::WrongNumOfPatterns(num_params)
                        .with_location(patterns[0].location().clone()))
    }

    fn parse_pattern_matched_stmt_body(&mut self, context: &StmtContext, first_pattern: Vec<Located<Pattern>>) -> ParseResult<'a, FunctionBody> {
        let num_params = context.in_function().unwrap().len();
        self.check_correct_pattern_length(&first_pattern, num_params)?;

        let mut branches = vec![(first_pattern, self.parse_stmt(context)?)];

        while self.advance_if(&[TokenKind::Colon])?.is_some() {
            let pattern = self.parse_pattern_list()?;
            self.expect(&[TokenKind::Be])?;
            let stmt = self.parse_stmt(context)?;
            self.check_correct_pattern_length(&pattern, num_params)?;
            branches.push((pattern, stmt));
        }

        Ok(FunctionBody::PatternMatchedStmt(branches))
    }

    fn parse_pattern_matched_expr_body(&mut self, context: &StmtContext, first_pattern: Vec<Located<Pattern>>) -> ParseResult<'a, FunctionBody> {
        let num_params = context.in_function().unwrap().len();
        self.check_correct_pattern_length(&first_pattern, num_params)?;

        let mut branches = vec![(first_pattern, self.parse_expr(context)?)];

        while self.advance_if(&[TokenKind::Colon])?.is_some() {
            let pattern = self.parse_pattern_list()?;
            self.expect(&[TokenKind::Condition])?;
            let expr = self.parse_expr(context)?;
            self.check_correct_pattern_length(&pattern, num_params)?;
            branches.push((pattern, expr));
        }

        Ok(FunctionBody::PatternMatchedExpr(branches))
    }

    fn parse_pattern_matched_body(&mut self, context: &StmtContext) -> ParseResult<'a, FunctionBody> {
        self.expect(&[TokenKind::Colon])?;

        let pattern = self.parse_pattern_list()?;
        if self.expect(&[TokenKind::Condition, TokenKind::Be])?.kind() == &TokenKind::Be {
            self.parse_pattern_matched_stmt_body(context, pattern)
        }
        else {
            self.parse_pattern_matched_expr_body(context, pattern)
        }
    }

    fn get_return_type(&self, body: &FunctionBody) -> Option<u32> {
        match body {
            FunctionBody::Expr(expr) => expr.typ().clone(),
            FunctionBody::PatternMatchedExpr(bodies) => bodies.first().map(|(_, expr)| expr.typ().clone()).flatten(),
            FunctionBody::Stmt(_) | FunctionBody::PatternMatchedStmt(_) => Some(self.get_type(TypeKind::Unit))
        }
    }
}

