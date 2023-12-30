use crate::{ast::expr::Expr, source_file::WithLocation};

use super::{Parser, ParseError, ParseResult};

impl<'a> Parser<'a> {
    pub(super) fn parse_expr(&mut self) -> ParseResult<'a, Expr> {
        Err(ParseError::NotImplemented.with_location(self.current().location().clone())) 
    }
}
