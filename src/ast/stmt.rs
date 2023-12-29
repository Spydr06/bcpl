use crate::source_file::Location;

use super::expr::Expr;

pub struct Stmt {
    loc: Location,

    kind: StmtKind
}

pub enum StmtKind {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
    ResultIs(Box<Expr>)
}
