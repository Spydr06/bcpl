use std::{ops::Deref, sync::{Arc, Mutex}};

use crate::{
    token::{lexer::Lexer, Token, TokenKind},
    source_file::{Location, Located, WithLocation},
    ast::{Program, stmt::StmtKind},
    error::{IntoCompilerError, CompilerError, Severity}
};

mod types;
mod decls;
mod expr;
mod stmt;
mod pattern;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    ast: Arc<Mutex<Program>>,
    warnings: Vec<Located<ParseError<'a>>>,
    current_token: Token<'a>
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>, ast: Arc<Mutex<Program>>) -> Self {
        Self {
            warnings: vec![],
            current_token: Token::eof(lexer.current_loc()),
            lexer,
            ast
        }
    }

    fn push_warning(&mut self, warning: Located<ParseError<'a>>) {
        self.warnings.push(warning);
    }

    pub fn warnings(&self) -> &Vec<Located<ParseError<'a>>> {
        &self.warnings
    }

    fn current(&self) -> &Token<'a> {
        &self.current_token
    }

    fn advance(&mut self) -> ParseResult<'a, Token<'a>> {
        let last = std::mem::replace(
            &mut self.current_token, 
            self.lexer.next()
                .unwrap_or_else(|| Token::error(self.lexer.current_loc(), Some("could not get next token".into())))
        );

        if let TokenKind::Error(msg) = self.current().kind() { 
            Err(
                ParseError::Generic(msg.clone().unwrap_or_default())
                    .with_location(self.current().location().clone())
            )
        }
        else {
            Ok(last)
        }
    }

    fn advance_if(&mut self, expect: &[TokenKind<'a>]) -> ParseResult<'a, Option<Token<'a>>> {
        if expect.contains(self.current().kind()) {
            self.advance().map(Some)
        }
        else {
            Ok(None)
        }
    }

    fn expect(&mut self, expect: &[TokenKind<'a>]) -> ParseResult<'a, Token<'a>> {
        if self.current().is_eof() {
            Err(ParseError::UnexpectedEof(Vec::from(expect)).with_location(self.current().location().clone()))
        }
        else if !expect.contains(self.current().kind()) {
            self.unexpected(expect)
        }
        else {
            self.advance()
        }
    }

    fn expect_ident(&mut self) -> ParseResult<'a, String> {
        if let TokenKind::Ident(ident) = self.current().kind() {
            let ident = ident.to_string();
            self.advance()?;
            Ok(ident)
        }
        else {
            self.unexpected(&[TokenKind::Ident("section name")])
        }
    }

    fn unexpected<T>(&mut self, want: &[TokenKind<'a>]) -> ParseResult<'a, T> {
        Err(
            ParseError::UnexpectedToken(self.current().kind().to_string(), want.to_vec())
                .with_location(self.current().location().clone())
        )
    }

    pub fn parse(&mut self) -> ParseResult<'a, ()> {
        self.advance()?;
        
        while !self.current_token.is_eof() {
            self.parse_section()?;
        }

        Ok(())
    } 

    fn parse_optional_list<T, U>(&mut self, start: TokenKind<'a>, end: TokenKind<'a>, delim: TokenKind<'a>, parse_func: fn(&mut Self, &U) -> ParseResult<'a, T>, param: &U) -> ParseResult<'a, Vec<T>> {
        if let Some(_) = self.advance_if(&[start])? {
            self.parse_list(end, delim, parse_func, param)
        }
        else {
            Ok(vec![])
        }
    }

    fn parse_list<T, U>(&mut self, end: TokenKind<'a>, delim: TokenKind<'a>, parse_func: fn(&mut Self, &U) -> ParseResult<'a, T>, param: &U) -> ParseResult<'a, Vec<T>> {
        let mut elems = vec![];
        let delims = [end.clone(), delim];
        while self.current_token.kind() != &end {
            elems.push(parse_func(self, param)?); 
            
            if self.expect(&delims)?.kind() == &end {
                break;
            }
        }
        Ok(elems)
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
    NotImplemented,
    Generic(String),
    UnexpectedEof(Vec<TokenKind<'a>>),
    UnexpectedToken(String, Vec<TokenKind<'a>>),
    Redefinition(Location, String),
    InvalidStmt(String, String),
    WrongNumOfPatterns(usize),
    NoResultValue,
    RequireAfterDecl,
    ExprWithoutSideEffect,
}

impl<'a> ParseError<'a> {
    fn severity(&self) -> Severity {
        match self {
            Self::RequireAfterDecl => Severity::Warning,
            Self::ExprWithoutSideEffect => Severity::Warning,
            _ => Severity::Error
        }
    }

    fn hint(&self) -> Option<String> {
        match self {
            Self::RequireAfterDecl => Some("Move this over the first declaration.".into()),
            _ => None
        }
    }

    fn additional(&self) -> Vec<Located<CompilerError>> {
        match self {
            Self::Redefinition(prev_loc, _) => vec![
                CompilerError::new(Severity::Hint, "First defined here.".into(), None, vec![])
                    .with_location(prev_loc.clone())
            ],
            _ => vec![]
        }
    }
}

fn tokens_to_string(list: &[TokenKind]) -> String {
    if list.len() == 1 {
        format!{"`{}`", list[0]}
    }
    else {
        format!(
            "one of {}",
            list.iter()
                .map(|tk| format!("`{tk}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<'a> WithLocation for ParseError<'a> {}

impl<'a> ToString for ParseError<'a> {
    fn to_string(&self) -> String {
        match self {
            Self::NotImplemented => "<internal> Not Implemented.".into(),
            Self::Generic(err) => err.clone(),
            Self::UnexpectedEof(tk) => format!("Unexpected end of file; Expected {}.", tokens_to_string(tk)),
            Self::UnexpectedToken(got, want) => format!("Unexpected token `{got}`; Expected {}.", tokens_to_string(want)),
            Self::Redefinition(_, ident) => format!("Redefinition of `{ident}`."),
            Self::RequireAfterDecl => format!("Encountered `require` after declarations."),
            Self::InvalidStmt(stmt, err) => format!("Encountered `{stmt}` statement outside of `{err}`."),
            Self::NoResultValue => format!("No `resultis` statement found in `valof` body."),
            Self::ExprWithoutSideEffect => format!("Expression has no side-effect when used as a statement."),
            Self::WrongNumOfPatterns(expect) => format!("Wrong number of patterns, expected {expect}."),
        }
    }
}

impl<'a> IntoCompilerError for ParseError<'a> {}
impl<'a> Into<CompilerError> for ParseError<'a> {
    fn into(self) -> CompilerError {
        CompilerError::new(self.severity(), self.to_string(), self.hint(), self.additional())   
    }
}
