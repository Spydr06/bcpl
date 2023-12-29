use std::ops::Deref;

use crate::{token::{lexer::Lexer, Token, TokenKind}, source_file::{Location, Located, WithLocation}};

use super::Program;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    warnings: Vec<ParseError<'a>>,
    current_token: Token<'a>
}

impl<'a> From<Lexer<'a>> for Parser<'a> {
    fn from(lexer: Lexer<'a>) -> Self {
        Self {
            warnings: vec![],
            current_token: Token::eof(lexer.current_loc()),
            lexer,
        }
    }
}

impl<'a> Parser<'a> {
    fn current(&self) -> &Token<'a> {
        &self.current_token
    }

    fn advance(&mut self) -> ParseResult<'a, &Token<'a>> {
        self.current_token = self.lexer.next()
            .unwrap_or_else(|| Token::error(self.lexer.current_loc(), Some("could not get next token".into())));

        if let TokenKind::Error(msg) = self.current().kind() { 
            Err(
                ParseError::Generic(msg.clone().unwrap_or_default())
                    .with_location(self.current().location().clone())
            )
        }
        else {
            Ok(self.current())
        }
    }

    fn expect(&mut self, expect: TokenKind<'a>) -> ParseResult<'a, &Token<'a>> {
        if self.current().is_eof() {
            Err(ParseError::UnexpectedEof(expect).with_location(self.current().location().clone()))
        }
        else if self.current().kind() != &expect {
            Err(ParseError::UnexpectedToken(self.current().kind().to_string(), expect)
                .with_location(self.current().location().clone())) 
        }
        else {
            self.advance()
        }
    }

    pub fn parse(&mut self, ast: &mut Program) -> ParseResult<()> {
        Ok(())
    }
}

impl<'a> Deref for Parser<'a> {
    type Target = Lexer<'a>;
    
    fn deref(&self) -> &Self::Target {
        &self.lexer
    }
}

pub type ParseResult<'a, T> = Result<T, Located<ParseError<'a>>>;

#[derive(Clone, Debug)]
pub enum ParseError<'a> {
    Generic(String),
    UnexpectedEof(TokenKind<'a>),
    UnexpectedToken(String, TokenKind<'a>)
}

impl<'a> WithLocation for ParseError<'a> {}
