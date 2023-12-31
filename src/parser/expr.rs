use std::cell::RefCell;

use crate::{
    ast::{expr::{Expr, ExprKind},
    types::TypeKind},
    token::TokenKind
};

use super::{Parser, ParseResult, stmt::StmtContext};

impl<'a> Parser<'a> {
    pub(super) fn parse_expr(&mut self, context: &StmtContext) -> ParseResult<'a, Expr> {
        self.parse_prefix_expr(context) // TODO: infix/postfix exprs
    }

    fn parse_prefix_expr(&mut self, context: &StmtContext) -> ParseResult<'a, Expr> {
        match self.current().kind() {
            TokenKind::True | TokenKind::False => self.parse_bool_lit(),
            TokenKind::IntegerLit(int) => self.parse_integer_lit(*int),
            TokenKind::ValOf => self.parse_valof(context),
            _ => self.unexpected(&[TokenKind::Ident("expression".into())])
        }
    }

    fn parse_bool_lit(&mut self) -> ParseResult<'a, Expr> {
        let loc = self.current().location().clone();
        let t = matches!(self.expect(&[TokenKind::True, TokenKind::False])?.kind(), TokenKind::True);

        Ok(Expr::new(loc, self.get_type(TypeKind::Bool), if t { ExprKind::True } else { ExprKind::False} ))
    }

    fn parse_integer_lit(&mut self, value: u64) -> ParseResult<'a, Expr> {
        let loc = self.current().location().clone();
        self.advance()?;

        let typ = match value {
            _ if value > std::i64::MAX as u64 => TypeKind::UInt64,
            _ if value > std::u32::MAX as u64 => TypeKind::Int64,
            _ if value > std::i32::MAX as u64 => TypeKind::UInt32,
            _ => TypeKind::Int32
        };

        Ok(Expr::new(loc, self.get_type(typ), ExprKind::IntLit(value)))
    }

    fn parse_valof(&mut self, context: &StmtContext) -> ParseResult<'a, Expr> {
        let loc = self.current().location().clone();
        self.expect(&[TokenKind::ValOf])?;

        let typ = RefCell::new(None);
        let stmt = self.parse_stmt(&StmtContext::ValOf(&typ, context))?;

        Ok(Expr::new(loc, typ.take(), ExprKind::ValOf(Box::new(stmt))))
    }
}
