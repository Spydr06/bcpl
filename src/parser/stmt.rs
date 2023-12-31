use std::cell::RefCell;

use crate::{
    ast::{stmt::{Stmt, StmtKind}, expr::{Expr, ExprKind}, types::TypeIndex}, 
    source_file::{WithLocation, Located},
    token::{Token, TokenKind}
};

use super::{Parser, ParseResult, ParseError};

pub(super) enum StmtContext<'a> {
    ValOf(&'a RefCell<Option<TypeIndex>>, &'a StmtContext<'a>),
    Block(&'a StmtContext<'a>),
    Empty
}

impl<'a> StmtContext<'a> {
    pub(super) fn last_valof_type(&self) -> Option<&RefCell<Option<TypeIndex>>> {
        match self {
            Self::ValOf(typ, _) => Some(typ),
            Self::Block(outer) => outer.last_valof_type(),
            Self::Empty => None
        }
    }

    fn require_semicolon(&self) -> bool {
        matches!(self, Self::Block(_))
    }
}

impl<'a> Parser<'a> {
    pub(super) fn parse_stmt(&mut self, context: &StmtContext) -> ParseResult<'a, Stmt> {
        match self.current().kind() {
            TokenKind::LBrace => self.parse_block(context),
            TokenKind::ResultIs => self.parse_resultis(context),
            _ => Err(
                ParseError::UnexpectedToken(self.current().kind().to_string(), vec![TokenKind::Ident("statement")])
                    .with_location(self.current().location().clone())
            )
        }
    }

    fn parse_block(&mut self, context: &StmtContext) -> ParseResult<'a, Stmt> {
        let loc = self.current().location().clone();
        self.expect(&[TokenKind::LBrace])?;

        let mut stmts = vec![];

        while self.current().kind() != &TokenKind::RBrace {
            stmts.push(self.parse_stmt(&StmtContext::Block(context))?)
        }

        self.advance()?;

        Ok(Stmt::new(loc, StmtKind::Block(stmts)))
    }

    fn parse_resultis(&mut self, context: &StmtContext) -> ParseResult<'a, Stmt> {
        let loc = self.current().location().clone();
        self.expect(&[TokenKind::ResultIs])?;
        
        let expr = self.parse_expr(context)?;
        let valof_typ = context.last_valof_type()
            .ok_or_else(|| 
                ParseError::InvalidStmt("resultis".into(), "valof".into())
                    .with_location(loc.clone())
            )?; 

        let vt = valof_typ.borrow().clone();
        let expr = match (vt, expr.typ()) {
            (Some(_), _) if &vt != expr.typ() => Expr::new(loc.clone(), vt, ExprKind::ImplicitCast(Box::new(expr))),
            (None, Some(typ)) => {
                *valof_typ.borrow_mut() = Some(*typ);
                expr
            }
            _ => expr
        };

        self.semicolon_if_required(context)?;

        Ok(Stmt::new(loc, StmtKind::ResultIs(Box::new(expr))))
    }

    fn semicolon_if_required(&mut self, context: &StmtContext) -> ParseResult<'a, ()> {
        if context.require_semicolon() {
            self.expect(&[TokenKind::Semicolon])?;
        }
        Ok(())
    }
}
