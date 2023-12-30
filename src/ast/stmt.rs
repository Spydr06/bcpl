use crate::source_file::Location;

use super::expr::Expr;

#[derive(Debug)]
pub struct Stmt {
    loc: Location,

    kind: StmtKind
}

#[derive(Debug)]
pub enum StmtKind {
    Expr(Box<Expr>),
    Block(Vec<Stmt>),
    ResultIs(Box<Expr>)
}
