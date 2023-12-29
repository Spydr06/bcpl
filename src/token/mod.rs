use crate::source_file::Location;

pub(crate) mod lexer;

#[derive(Debug)]
pub struct Token<'a> {
    kind: TokenKind<'a>,
    loc: Location
}

#[derive(Debug)]
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
    Comma, // `;`
    Period, // `.`
    Colon, // `:`
    Assign, // `:=`
    Condition, // `->`
    QuestionMark, // `?`
    Bang, // `!`
    At, // `@`

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
    Finish,
    Skip,
    Repeat,
    Break,
    If,
    Unless,
    While,
    For,
    Until,
    SwitchOn,
    Match,
    Every,
    Case,
    Default,
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
    Abs
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
            '=' => Ok(TK::Eq),
            '!' => Ok(TK::Bang),
            '?' => Ok(TK::QuestionMark),
            ',' => Ok(TK::Comma),
            '@' => Ok(TK::At),
            '.' => Ok(TK::Period),
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
            "finish" => TK::Finish,
            "skip" => TK::Skip,
            "repeat" => TK::Repeat,
            "break" => TK::Break,
            "if" => TK::If,
            "unless" => TK::Unless,
            "until" => TK::Until,
            "switchon" => TK::SwitchOn,
            "match" => TK::Match,
            "every" => TK::Every,
            "while" => TK::While,
            "case" => TK::Case,
            "default" => TK::Default,
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
}
