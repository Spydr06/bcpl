use std::ops::Deref;

use crate::{
    token::{lexer::Lexer, Token, TokenKind},
    source_file::{Location, Located, WithLocation},
    ast::{Program, Decl},
    error::{IntoCompilerError, CompilerError, Severity}
};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    warnings: Vec<Located<ParseError<'a>>>,
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
    fn push_warning(&mut self, warning: Located<ParseError<'a>>) {
        self.warnings.push(warning);
    }

    pub fn warnings(&self) -> &Vec<Located<ParseError<'a>>> {
        &self.warnings
    }

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

    fn expect_ident(&mut self) -> ParseResult<'a, String> {
        if let TokenKind::Ident(ident) = self.current().kind() {
            let ident = ident.to_string();
            self.advance()?;
            Ok(ident)
        }
        else {
            Err(
                ParseError::UnexpectedToken(self.current().kind().to_string(), TokenKind::Ident("section name"))
                    .with_location(self.current().location().clone())
            )
        }
    }

    pub fn parse(&mut self, ast: &mut Program) -> ParseResult<'a, ()> {
        self.advance()?;
        
        while !self.current_token.is_eof() {
            self.parse_section(ast)?;
        }

        Ok(())
    }

    fn parse_section(&mut self, ast: &mut Program) -> ParseResult<'a, ()> {
        let section_loc = self.current_token.location().clone();
        self.expect(TokenKind::Section)?;

        let section = ast.add_section(&self.expect_ident()?.into(), section_loc); 

        let mut had_decls = false;
        loop {
            match self.current().kind() {
                TokenKind::Eof | TokenKind::Section => return Ok(()),
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
                    had_decls = true;
                }
            }
        }
    }

    fn parse_require(&mut self) -> ParseResult<'a, Located<String>> {
        let loc = self.current_token.location().clone();
        self.expect(TokenKind::Require)?;

        Ok(self.expect_ident()?.to_string().with_location(loc))
    }

    fn parse_decl(&mut self) -> ParseResult<'a, Box<dyn Decl>> {
        Err(ParseError::NotImplemented.with_location(self.current_token.location().clone()))
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
    UnexpectedEof(TokenKind<'a>),
    UnexpectedToken(String, TokenKind<'a>),
    Redefinition(Location, String),
    RequireAfterDecl
}

impl<'a> ParseError<'a> {
    fn severity(&self) -> Severity {
        match self {
            Self::RequireAfterDecl => Severity::Warning,
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

impl<'a> WithLocation for ParseError<'a> {}

impl<'a> ToString for ParseError<'a> {
    fn to_string(&self) -> String {
        match self {
            Self::NotImplemented => "<internal> Not Implemented.".into(),
            Self::Generic(err) => err.clone(),
            Self::UnexpectedEof(tk) => format!("Unexpected end of file; Expected `{tk}`."),
            Self::UnexpectedToken(got, want) => format!("Unexpected token `{got}`; Expected `{want}`."),
            Self::Redefinition(_, ident) => format!("Redefinition of `{ident}`."),
            Self::RequireAfterDecl => format!("Encountered `require` after declarations.")
        }
    }
}

impl<'a> IntoCompilerError for ParseError<'a> {}
impl<'a> Into<CompilerError> for ParseError<'a> {
    fn into(self) -> CompilerError {
        CompilerError::new(self.severity(), self.to_string(), self.hint(), self.additional())   
    }
}
