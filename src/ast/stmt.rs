use crate::source_file::Location;

use super::expr::Expr;

#[derive(Debug)]
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
}

#[derive(Debug)]
pub enum StmtKind {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
    ResultIs(Box<Expr>)
}
