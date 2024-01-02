use std::{
    str::Chars,
    iter::Peekable,
    ops::Deref
};

use crate::{source_file::{SourceFile, Location}, token::{Token, TokenKind}};

const ESCAPE_CHAR: char = '\\';

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
enum Base {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16
}

#[derive(Debug)]
pub struct Lexer<'a> {
    source_file: &'a SourceFile,
    iter: Peekable<Chars<'a>>,
    offset: usize,
    line: usize,
    column: usize
}

impl<'a> Lexer<'a> {
    pub fn from(source_file: &'a SourceFile) -> Self {
        Self {
            source_file,
            iter: source_file.contents().chars().peekable(),
            offset: 0,
            line: 1,
            column: 0
        }
    }

    pub fn current_loc(&self) -> Location {
        Location::new(self.source_file, self.line, self.column, 1)        
    }

    fn next_char(&mut self) {
        self.offset += 1;

        if self.iter.next().unwrap() == '\n' {
            self.line += 1;
            self.column = 0;
        }
        else {
            self.column += 1;
        }
    }

    fn skip_comment(&mut self) {
        while let Some(&ch) = self.iter.peek() && ch != '\n' {
            self.next_char();
        }
    }

    fn skip_multiline_comment(&mut self) {
        let mut depth = 1;
        while let Some(&ch) = self.iter.peek() && depth > 0 {
            self.next_char();
            if ch == '/' && let Some(&ch) = self.iter.peek() && ch == '*' {
                self.next_char();
                depth += 1;
            }
            else if ch == '*' && let Some(&ch) = self.iter.peek() && ch == '/' {
                self.next_char();
                depth -= 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.iter.peek() && ch.is_whitespace() {
            self.next_char();     
        }
    }

    fn parse_ident(&mut self) -> &'a str {
        let start = self.offset;
        while let Some(&ch) = self.iter.peek() && (ch.is_alphanumeric() || ch == '_') {
            self.next_char();
        }

        &self.source_file.contents()[start..self.offset]
    }

    fn skip_digits(&mut self, base: Base) {
        while let Some(&ch) = self.iter.peek() && ch.is_digit(base as u32) {
            self.next_char();
        }
    }

    fn parse_number(&mut self) -> (TokenKind<'a>, usize) {
        let mut base = Base::Decimal;

        let mut start = self.offset;
        if self.iter.peek().copied().unwrap_or('\0') == '0' {
            self.next_char();

            base = match self.iter.peek() {
                Some('b' | 'B') => Base::Binary,
                Some('o' | 'O') => Base::Octal,
                Some('x' | 'X') => Base::Hexadecimal,  
                _ => Base::Decimal
            };

            if base != Base::Decimal {
                self.next_char();
                start = self.offset;
            }
        }

        self.skip_digits(base);
        
        let mut is_float = self.iter.peek().copied().unwrap_or('\0') == '.';
        if is_float {
            self.next_char();
            self.skip_digits(base);
        }

        let end = self.offset;
        let mut exponent_start = 0;

        let has_exponent = "eE".contains(self.iter.peek().copied().unwrap_or('\0'));
        if has_exponent {
            self.next_char();
            is_float = true;
            exponent_start = self.offset;

            if "+-".contains(self.iter.peek().copied().unwrap_or('\0')) {
                self.next_char();
                self.skip_digits(base);
            }
        }

        let exponent_end = self.offset;

        (
            if is_float {
                if base != Base::Decimal {
                    TokenKind::FloatLit(
                        &self.source_file.contents()[start..end], 
                        has_exponent.then(|| &self.source_file.contents()[exponent_start..exponent_end])
                    )
                } else {
                    TokenKind::Error(Some("float literals have to be of base 10.".into()))
                }
            }
            else {
                TokenKind::IntegerLit(self.source_file.contents()[start..end].parse().expect("could not parse integer literal"))
            }, 
            exponent_end - start + if base != Base::Decimal { 2 } else { 0 }
        )
    }

    fn parse_string_lit(&mut self, quote: char) -> &'a str {
        let start = self.offset;

        while let Some(&ch) = self.iter.peek() && ch != quote {
            self.next_char();
            if ch == ESCAPE_CHAR {
                self.next_char();
            }
            else if ch == '\n' {
                self.line += 1;
                self.column = 0;
            }
        }

        self.next_char(); // expect `"`
        &self.source_file.contents()[start..self.offset - 1]
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let loc = self.current_loc(); 

        let next = self.iter.peek().copied();
        if next.is_none() {
            return Some(Token::eof(loc)) 
        }

        let ch = next.unwrap();
        match ch {
            _ if ch.is_alphabetic() || ch == '_' => {
                Some(Token::ident(loc, self.parse_ident()))
            }
            _ if ch.is_numeric() => {
                let (kind, width) = self.parse_number();
                Some(Token::with_width(loc, width, kind))
            }
            '"' => {
                self.next_char();
                Some(Token::string_lit(loc, self.parse_string_lit('"')))
            }
            '\'' => {
                self.next_char();
                Some(Token::char_lit(loc, self.parse_string_lit('"')))
            }
            '#' => {
                self.next_char();
                let atom = self.parse_ident();
                if atom.len() == 0 {
                    Some(Token::error(self.current_loc(), Some("expect atom identifier after `#`".into())))
                }
                else {
                    Some(Token::with_width(loc, atom.len() + 1, TokenKind::Atom(atom)))
                }
            }
            '(' | ')' | '{' | '}' | '[' | ']' | ';' | '+' | '*' | '=' | '!' | '?' | ',' | '@' | '|' | '&' | '^' => {
                self.next_char();
                Some(Token::new(loc, TokenKind::try_from(ch).expect("invalid character")))
            }
            '-' => {
                self.next_char();
                if let Some(&ch) = self.iter.peek() && ch == '>' {
                    self.next_char();
                    Some(Token::with_width(loc, 2, TokenKind::Condition))
                }
                else {
                    Some(Token::new(loc, TokenKind::Minus))
                }
            }
            '/' => {
                self.next_char();
                if let Some(&ch) = self.iter.peek() {
                    if ch == '/' {
                        self.skip_comment();
                        return self.next()
                    }
                    else if ch == '*' {
                        self.skip_multiline_comment();
                        return self.next()
                    }
                }
                
                Some(Token::new(loc, TokenKind::Slash))
            }
            ':' => {
                self.next_char();
                if let Some(&ch) = self.iter.peek() {
                    if ch == '=' {
                        self.next_char();
                        return Some(Token::with_width(loc, 2, TokenKind::Assign))
                    }
                    else if ch == ':' {
                        self.next_char();
                        return Some(Token::with_width(loc, 2, TokenKind::Of))
                    }
                }

                Some(Token::new(loc, TokenKind::Colon))
            }
            '.' => {
                self.next_char();
                if let Some(&ch) = self.iter.peek() && ch == '.' {
                    self.next_char();
                    return Some(Token::with_width(loc, 2, TokenKind::Range))
                }
                else {
                    return Some(Token::new(loc, TokenKind::Period))
                }
            }
            '<' => {
                self.next_char();
                if let Some(&ch) = self.iter.peek() {
                    if ch == '=' {
                        self.next_char();
                        return Some(Token::with_width(loc, 2, TokenKind::Le))
                    }
                    else if ch == '>' {
                        self.next_char();
                        return Some(Token::with_width(loc, 2, TokenKind::Compound))
                    }
                    else if ch == '<' {
                        self.next_char();
                        return Some(Token::with_width(loc, 2, TokenKind::LShift))
                    }
                }
                
                Some(Token::new(loc, TokenKind::Lt))
            }
            '>' => {
                self.next_char();
                if let Some(&ch) = self.iter.peek() {
                    if ch == '=' {
                        self.next_char();
                        return Some(Token::with_width(loc, 2, TokenKind::Ge))
                    }
                    else if ch == '>' {
                        self.next_char();
                        return Some(Token::with_width(loc, 2, TokenKind::RShift))
                    }
                }

                Some(Token::new(loc, TokenKind::Gt)) 
            }
            '~' => {
                self.next_char();
                if let Some(&ch) = self.iter.peek() && ch == '=' {
                    self.next_char();
                    Some(Token::with_width(loc, 2, TokenKind::Ne))
                }
                else {
                    Some(Token::new(loc, TokenKind::Not))
                } 
            }
            _ => {
                self.next_char();
                Some(Token::error(loc, Some(format!("unexpected character `{}`", ch))))
            }
        }
    }
}

impl<'a> Deref for Lexer<'a> {
    type Target = &'a SourceFile;

    fn deref(&self) -> &Self::Target {
        &self.source_file
    }
}

