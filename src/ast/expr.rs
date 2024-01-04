use crate::source_file::{Location, Located};

use super::{types::TypeIndex, stmt::Stmt, pattern::Pattern};

pub type AtomIndex = u32;

#[derive(Clone, Debug, PartialEq)]
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

    pub fn location(&self) -> &Location {
        &self.loc
    }

    pub fn typ(&self) -> &Option<TypeIndex> {
        &self.typ
    }

    pub fn set_typ(&mut self, typ: TypeIndex) {
        self.typ = Some(typ)
    }

    pub fn has_sideeffect(&self) -> bool {
        match &self.kind {
            ExprKind::Cast(expr) | ExprKind::ImplicitCast(expr) => expr.has_sideeffect(),
            ExprKind::ValOf(_) | ExprKind::FuncCall(..) => true,
            _ => false
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Ident(String),
    Atom(AtomIndex),

    IntLit(u64),
    FloatLit(f64),
    CharLit(char),
    StringLit(String),

    True,
    False,

    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    Mod(Box<Expr>, Box<Expr>),
    Abs(Box<Expr>),

    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    XOr(Box<Expr>, Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),

    LShift(Box<Expr>, Box<Expr>),
    RShift(Box<Expr>, Box<Expr>),

    Ref(Box<Expr>),
    Deref(Box<Expr>),

    Cast(Box<Expr>),
    ImplicitCast(Box<Expr>),
    ValOf(Box<Stmt>), 
    FuncCall(Box<Expr>, Vec<Expr>),

    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),

    Match(Vec<Expr>, Vec<(Vec<Located<Pattern>>, Box<Expr>)>),
    Every(Vec<Expr>, Vec<(Vec<Located<Pattern>>, Box<Expr>)>),
}
