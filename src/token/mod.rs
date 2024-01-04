use std::fmt::Display;

use crate::source_file::Location;

pub(crate) mod lexer;

#[derive(Debug)]
pub struct Token<'a> {
    kind: TokenKind<'a>,
    loc: Location
}

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind<'a> {
    // Lexer Signals
    Eof,
    Error(Option<String>),

    // Identifiers
    Ident(&'a str),
    Atom(&'a str),

    // Literals
    IntegerLit(u64),
    FloatLit(&'a str, Option<&'a str>),
    StringLit(&'a str),
    CharLit(&'a str),

    // Symbols
    LParen, // `(`
    RParen, // `)`
    LBrace, // `{`
    RBrace, // `}`
    LBracket, // `[`
    RBracket, // `]`

    Semicolon, // `;`
    Comma, // `,`
    Period, // `.`
    Colon, // `:`
    Assign, // `:=`
    Condition, // `->`
    Arrow, // `=>`
    QuestionMark, // `?`
    Bang, // `!`
    At, // `@`
    Compound, // `<>`
    Range, // `..`

    Plus,
    Minus,
    Star,
    Slash,

    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,

    Not,
    LogAnd,
    LogOr,
    XOr,
    LShift,
    RShift,

    // Keywords
    True,
    False,
    Let,
    And,
    ValOf,
    ResultIs,
    Return,
    Break,
    Next,
    If,
    Else,
    Unless,
    While,
    For,
    Until,
    SwitchOn,
    Match,
    Every,
    Case,
    Default,
    Into,
    Do,
    To,
    By,
    Of,
    Be,
    Section,
    Require,
    Global,
    Manifest,
    Static,
    Mod,
    Abs,
    Type
}

impl<'a> Display for TokenKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TokenKind as TK;
        match self {
            TK::Error(Some(err)) => return write!(f, "{err}"),
            TK::Atom(atom) => return write!(f, "#{atom}"),
            TK::IntegerLit(int) => return write!(f, "{int}"),
            TK::StringLit(s) => return write!(f, "\"{s}\""),
            TK::CharLit(ch) => return write!(f, "'{ch}'"),
            _ => ()
        }

        let s = match self {
            TK::Eof => "end of file",
            TK::Error(None) => "error",
            
            TK::Ident(ident) => ident,

            TK::FloatLit(a, _) => a, // TODO: print correctly

            TK::LParen => "(",
            TK::RParen => ")",
            TK::LBrace => "{",
            TK::RBrace => "}",
            TK::LBracket => "[",
            TK::RBracket => "]",
            
            TK::Semicolon => ";",
            TK::Comma => ",",
            TK::Period => ".",
            TK::Colon => ":",
            TK::Assign => ":=",
            TK::Condition => "->",
            TK::Arrow => "=>",
            TK::QuestionMark => "?",
            TK::Bang => "!",
            TK::At => "@",
            TK::Compound => "<>",
            TK::Range => "..",

            TK::Plus => "+",
            TK::Minus => "-",
            TK::Star => "*",
            TK::Slash => "/",

            TK::Eq => "=",
            TK::Ne => "~=",
            TK::Gt => ">",
            TK::Ge => ">=",
            TK::Lt => "<",
            TK::Le => "<=",

            TK::Not => "~",
            TK::LogAnd => "&",
            TK::LogOr => "|",
            TK::XOr => "^",
            TK::LShift => "<<",
            TK::RShift => ">>",

            TK::True => "true",
            TK::False => "false",
            TK::Let => "let",
            TK::And => "and",
            TK::ValOf => "valof",
            TK::ResultIs => "resultis",
            TK::Return => "return",
            TK::Next => "next",
            TK::Break => "break",
            TK::If => "if",
            TK::Else => "else",
            TK::Unless => "unless",
            TK::While => "while",
            TK::For => "for",
            TK::Until => "until",
            TK::SwitchOn => "switchon",
            TK::Match => "match",
            TK::Every => "every",
            TK::Case => "case",
            TK::Default => "default",
            TK::Into => "into",
            TK::Do => "do",
            TK::To => "to",
            TK::By => "by",
            TK::Of => "::",
            TK::Be => "be",
            TK::Section => "section",
            TK::Require => "require",
            TK::Global => "global",
            TK::Manifest => "manifest",
            TK::Static => "static",
            TK::Mod => "mod",
            TK::Abs => "abs",
            _ => "<unexpected>"
        };

        write!(f, "{s}")
    }
}

impl<'a> TryFrom<char> for TokenKind<'a> {
    type Error = ();

    fn try_from(value: char) -> Result<Self, ()> {
        use TokenKind as TK;
        match value {
            '(' => Ok(TK::LParen),
            ')' => Ok(TK::RParen),
            '{' => Ok(TK::LBrace),
            '}' => Ok(TK::RBrace),
            '[' => Ok(TK::LBracket),
            ']' => Ok(TK::RBracket),
            ';' => Ok(TK::Semicolon),
            '+' => Ok(TK::Plus),
            '*' => Ok(TK::Star),
            '!' => Ok(TK::Bang),
            '?' => Ok(TK::QuestionMark),
            ',' => Ok(TK::Comma),
            '@' => Ok(TK::At),
            '|' => Ok(TK::LogOr),
            '&' => Ok(TK::LogAnd),
            '^' => Ok(TK::XOr),
            _ => Err(())
        } 
    }
}

impl<'a> From<&'a str> for TokenKind<'a> {
    fn from(value: &'a str) -> Self {
        use TokenKind as TK;
        match value {
            "true" => TK::True,
            "false" => TK::False,
            "let" => TK::Let,
            "and" => TK::And,
            "valof" => TK::ValOf,
            "resultis" => TK::ResultIs,
            "return" => TK::Return,
            "next" => TK::Next,
            "break" => TK::Break,
            "if" => TK::If,
            "else" => TK::Else,
            "unless" => TK::Unless,
            "until" => TK::Until,
            "switchon" => TK::SwitchOn,
            "match" => TK::Match,
            "every" => TK::Every,
            "while" => TK::While,
            "case" => TK::Case,
            "default" => TK::Default,
            "into" => TK::Into,
            "do" => TK::Do,
            "to" => TK::To,
            "by" => TK::By,
            "of" => TK::Of,
            "be" => TK::Be,
            "section" => TK::Section,
            "require" => TK::Require,
            "global" => TK::Global,
            "manifest" => TK::Manifest,
            "static" => TK::Static,
            "mod" => TK::Mod,
            "abs" => TK::Abs,
            "for" => TK::For,
            "type" => TK::Type,
            _ => TK::Ident(value.into())
        } 
    }
}

impl<'a> Token<'a> {
    pub fn new(loc: Location, kind: TokenKind<'a>) -> Self {
        Self {
            kind,
            loc
        }
    }

    pub fn with_width(mut loc: Location, width: usize, kind: TokenKind<'a>) -> Self {
        loc.set_width(width);
        Self {
            kind,
            loc
        }
    }

    pub fn eof(loc: Location) -> Self {
        Self {
            kind: TokenKind::Eof,
            loc
        }
    }

    pub fn is_eof(&self) -> bool {
        match self.kind {
            TokenKind::Eof => true,
            _ => false
        }
    }

    pub fn error(loc: Location, msg: Option<String>) -> Self {
        Self {
            kind: TokenKind::Error(msg),
            loc
        }
    }

    pub fn ident(mut loc: Location, ident: &'a str) -> Self {
        loc.set_width(ident.len());
        Self {
            kind: TokenKind::from(ident),
            loc
        }
    }

    pub fn string_lit(mut loc: Location, val: &'a str) -> Self {
        loc.set_width(val.len() + 2);
        Self {
            kind: TokenKind::StringLit(val),
            loc
        }
    }

    pub fn char_lit(mut loc: Location, val: &'a str) -> Self {
        loc.set_width(val.len() + 2);
        Self {
            kind: TokenKind::CharLit(val),
            loc
        }
    }

    pub fn kind(&self) -> &TokenKind {
        &self.kind
    }
    
    pub fn location(&self) -> &Location {
        &self.loc
    }
}
