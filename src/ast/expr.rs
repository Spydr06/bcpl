use crate::source_file::Location;

use super::{types::TypeIndex, stmt::Stmt};

#[derive(Debug)]
pub struct Expr {
    loc: Location,
    typ: TypeIndex,

    kind: ExprKind,
}

#[derive(Debug)]
pub enum ExprKind {
    Ident(String),

    IntLit(u64),
    FloatLit(f64),
    CharLit(char),
    StringLit(String),

    True,
    False,

    Cast(Box<Expr>),
    ImplicitCast(Box<Expr>),
    ValOf(Box<Stmt>), 
    FuncCall(Box<Expr>, Vec<Expr>)
}
