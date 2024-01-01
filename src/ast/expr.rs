use crate::source_file::Location;

use super::{types::TypeIndex, stmt::Stmt};

#[derive(Debug)]
pub struct Expr {
    loc: Location,
    typ: Option<TypeIndex>,

    kind: ExprKind,
}

impl Expr {
    pub fn new(loc: Location, typ: Option<TypeIndex>, kind: ExprKind) -> Self {
        Self {
            loc,
            typ,
            kind
        }
    }

    pub fn implicit_cast(self, typ: TypeIndex) -> Self {
        Self {
            loc: self.loc.clone(),
            typ: Some(typ),
            kind: ExprKind::ImplicitCast(Box::new(self))
        }
    }

    pub fn typ(&self) -> &Option<TypeIndex> {
        &self.typ
    }

    pub fn has_sideeffect(&self) -> bool {
        match &self.kind {
            ExprKind::Cast(expr) | ExprKind::ImplicitCast(expr) => expr.has_sideeffect(),
            ExprKind::ValOf(_) | ExprKind::FuncCall(..) => true,
            _ => false
        }
    }
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
