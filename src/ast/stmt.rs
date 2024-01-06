use crate::source_file::{Location, Located};

use super::{expr::Expr, pattern::Pattern};

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    loc: Location,

    kind: StmtKind
}

impl Stmt {
    pub fn new(loc: Location, kind: StmtKind) -> Self {
        Self {
            loc,
            kind
        }
    }

    pub fn kind_mut(&mut self) -> &mut StmtKind {
        &mut self.kind
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    Nop,

    Expr(Box<Expr>),
    Block(Vec<Stmt>),

    ResultIs(Box<Expr>),
    Return,

    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Unless(Box<Expr>, Box<Stmt>),
    
    While(Box<Expr>, Box<Stmt>),
    Until(Box<Expr>, Box<Stmt>),

    //  iterator   init val   target val         stepsize           body
    For(Located<Pattern>, Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>, Box<Stmt>),

    SwitchOn(Box<Expr>, Box<Stmt>),
    Case(Box<Expr>),
    DefaultCase,
    
    Break,
    Next,

    Match(Vec<Expr>, Vec<(Vec<Located<Pattern>>, Box<Stmt>)>),
    Every(Vec<Expr>, Vec<(Vec<Located<Pattern>>, Box<Stmt>)>),

    Binding(Vec<(Located<Pattern>, Expr)>)
}
