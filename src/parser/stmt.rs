use std::cell::RefCell;

use crate::{
    ast::{stmt::{Stmt, StmtKind}, expr::{Expr, ExprKind}, types::TypeIndex}, 
    source_file::{WithLocation, Located},
    token::{Token, TokenKind}
};

use super::{Parser, ParseResult, ParseError};

pub(super) enum StmtContext<'a> {
    ValOf(&'a RefCell<Option<Option<TypeIndex>>>, &'a StmtContext<'a>),
    Block(&'a StmtContext<'a>),
    Function(&'a RefCell<Option<Option<TypeIndex>>>),
    Empty
}

impl<'a> StmtContext<'a> {
    pub(super) fn last_valof_type(&self) -> Option<&RefCell<Option<Option<TypeIndex>>>> {
        match self {
            Self::ValOf(typ, _) => Some(typ),
            Self::Block(outer) => outer.last_valof_type(),
            Self::Function(_) => None,
            Self::Empty => None
        }
    }

    pub(super) fn function_return_type(&self) -> Option<&RefCell<Option<Option<TypeIndex>>>> {
        match self {
            Self::ValOf(_, outer) | Self::Block(outer) => outer.function_return_type(),
            Self::Function(return_type) => Some(return_type),
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
            TokenKind::Return => self.parse_return(context),
            _ => self.parse_expr_stmt(context),
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
        let loc = self.expect(&[TokenKind::ResultIs])?.location().clone();
        
        let expr = self.parse_expr(context)?;
        let valof_typ = context.last_valof_type()
            .ok_or_else(|| 
                ParseError::InvalidStmt("resultis".into(), "valof".into())
                    .with_location(loc.clone())
            )?; 

        let vt = valof_typ.borrow().clone();
        let expr = match vt {
            Some(vt) if &vt != expr.typ() => Expr::new(loc.clone(), vt, ExprKind::ImplicitCast(Box::new(expr))),
            None => {
                *valof_typ.borrow_mut() = Some(expr.typ().clone());
                expr
            }
            _ => expr
        };

        self.semicolon_if_required(context)?;

        Ok(Stmt::new(loc, StmtKind::ResultIs(Box::new(expr))))
    }

    fn parse_return(&mut self, context: &StmtContext) -> ParseResult<'a, Stmt> {
        let loc = self.expect(&[TokenKind::Return])?.location().clone();

        let expr = self.parse_expr(context)?;
        let return_type = context.function_return_type()
            .ok_or_else(||
                ParseError::InvalidStmt("return".into(), "function".into())
                    .with_location(loc.clone())
            )?;
        
        let rt = return_type.borrow().clone();
        let expr = match rt {
            Some(rt) if &rt != expr.typ() => Expr::new(loc.clone(), rt, ExprKind::ImplicitCast(Box::new(expr))),
            None => {
                *return_type.borrow_mut() = Some(expr.typ().clone());
                expr
            }
            _ => expr
        };

        self.semicolon_if_required(context)?;

        Ok(Stmt::new(loc, StmtKind::Return(Box::new(expr))))
    }

    fn parse_expr_stmt(&mut self, context: &StmtContext) -> ParseResult<'a, Stmt> {
        let loc = self.current().location().clone();
        let expr = self.parse_expr(context)?;
        if !expr.has_sideeffect() {
            self.push_warning(ParseError::ExprWithoutSideEffect.with_location(loc.clone()))
        }

        self.semicolon_if_required(context)?;

        Ok(Stmt::new(loc, StmtKind::Expr(Box::new(expr))))
    }

    fn semicolon_if_required(&mut self, context: &StmtContext) -> ParseResult<'a, ()> {
        if context.require_semicolon() {
            self.expect(&[TokenKind::Semicolon])?;
        }
        Ok(())
    }
}
