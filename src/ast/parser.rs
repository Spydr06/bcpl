use crate::token::lexer::Lexer;

use super::Program;

struct Parser<'a> {
    lexer: Lexer<'a>
}

impl<'a> From<Lexer<'a>> for Parser<'a> {
    fn from(lexer: Lexer<'a>) -> Self {
        Self {
            lexer
        }
    }
}

impl<'a> Parser<'a> {
    fn parse(&mut self, ast: &mut Program) {

    }
}
