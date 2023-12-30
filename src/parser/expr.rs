use crate::{ast::{expr::{Expr, ExprKind}, types::TypeKind}, source_file::WithLocation, token::TokenKind};

use super::{Parser, ParseError, ParseResult};

impl<'a> Parser<'a> {
    pub(super) fn parse_expr(&mut self) -> ParseResult<'a, Expr> {
        self.parse_prefix_expr() // TODO: infix/postfix exprs
    }

    fn parse_prefix_expr(&mut self) -> ParseResult<'a, Expr> {
        match self.current().kind() {
            TokenKind::True | TokenKind::False => self.parse_bool_lit(),
            _ => self.unexpected(&[TokenKind::Ident("expression".into())])
        }
    }

    fn parse_bool_lit(&mut self) -> ParseResult<'a, Expr> {
        let loc = self.current().location().clone();
        let t = matches!(self.expect(&[TokenKind::True, TokenKind::False])?.kind(), TokenKind::True);

        Ok(Expr::new(loc, self.get_type(TypeKind::Bool), if t { ExprKind::True } else { ExprKind::False} ))
    }
}
