use crate::source_file::{Located, WithLocation};

use super::expr::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum Pattern {
    Any, // `?`  
    Query(String), // identifier
    Term(PatternTerm), // expression/range/operator

    Or(Box<Located<Pattern>>, Box<Located<Pattern>>), // `A | B`
    And(Box<Located<Pattern>>, Box<Located<Pattern>>), // `A & B`
    
    Variant(String, Vec<Located<Pattern>>), // `Foo(Bar, Baz, ...)`
    List(Vec<Located<Pattern>>), // `[A, B, C, ...]`
    Remaining, // `..`
}

impl WithLocation for Pattern {}

#[derive(Clone, Debug, PartialEq)]
pub enum PatternTerm {
    Basic(Expr),
    Lt(Expr),
    Le(Expr),
    Gt(Expr),
    Ge(Expr),
    Ne(Expr),
    Eq(Expr),
    Range(Expr, Expr)
}

