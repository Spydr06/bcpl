use crate::{match_decl, ast::{ManifestDecl, FunctionBody, pattern::PatternTerm, stmt::StmtKind, expr::ExprKind}};

use super::{Program, Function, Section, Decl, Param, stmt::Stmt, expr::Expr, pattern::Pattern};

pub enum Action {
    Continue,
    Break
}

macro_rules! act {
    ($inner: expr) => {
        if let Action::Break = ($inner) {
            return Ok(Action::Break)
        }   
    };
}

pub trait Visitor<T, E> where T: Traversable {
    fn visit(&mut self, node: &mut T) -> Result<Action, E>;
    fn visit_before(&mut self, _node: &mut T) -> Result<Action, E> { Ok(Action::Continue) }
}

pub trait ASTVisitor<E> = Visitor<Program, E> 
    + Visitor<Section, E>
    + Visitor<Function, E>
    + Visitor<Param, E>
    + Visitor<Stmt, E>
    + Visitor<Expr, E>
    + Visitor<Pattern, E>;

pub trait Traversable {
    fn traverse<E>(&mut self, visitor: &mut impl ASTVisitor<E>) -> Result<Action, E>;
}

impl Traversable for Program {
    fn traverse<E>(&mut self, visitor: &mut impl ASTVisitor<E>) -> Result<Action, E> {
        act!(visitor.visit_before(self)?);
        
        for (_, section) in &mut self.sections {
            act!(section.traverse(visitor)?)
        }

        visitor.visit(self)
    }
}

impl Traversable for Section {
    fn traverse<E>(&mut self, visitor: &mut impl ASTVisitor<E>) -> Result<Action, E> {
        act!(visitor.visit_before(self)?);

        for (_, decl) in &mut self.declarations {
            match_decl!{
                mut decl;
                func as Function => {
                    act!(func.traverse(visitor)?)
                },
                manifest as ManifestDecl => {
                    println!("manifest: {}", manifest.ident());
                },
                _ => ()
            }
        }

        visitor.visit(self)
    }
}

impl Traversable for Function {
    fn traverse<E>(&mut self, visitor: &mut impl ASTVisitor<E>) -> Result<Action, E> {
        act!(visitor.visit_before(self)?);

        for param in &mut self.params {
            act!(param.traverse(visitor)?)
        }

        match &mut self.body {
            FunctionBody::Expr(expr) => act!(expr.traverse(visitor)?),
            FunctionBody::Stmt(stmt) => act!(stmt.traverse(visitor)?),
            FunctionBody::PatternMatchedExpr(branches) => {
                for (patterns, expr) in branches {
                    for pattern in patterns {
                        act!(pattern.traverse(visitor)?)
                    }
                    act!(expr.traverse(visitor)?)
                }
            }
            FunctionBody::PatternMatchedStmt(branches) => {
                for (patterns, stmt) in branches {
                    for pattern in patterns {
                        act!(pattern.traverse(visitor)?)
                    }
                    act!(stmt.traverse(visitor)?)
                } 
            }
        }

        visitor.visit(self)
    }
}

impl Traversable for Param {
    fn traverse<E>(&mut self, visitor: &mut impl ASTVisitor<E>) -> Result<Action, E> {
        act!(visitor.visit_before(self)?);

        act!(self.ident.traverse(visitor)?);
        if let Some(default_value) = &mut self.default_value {
            act!(default_value.traverse(visitor)?);
        }

        visitor.visit(self)
    }
}

impl Traversable for Stmt {
    fn traverse<E>(&mut self, visitor: &mut impl ASTVisitor<E>) -> Result<Action, E> {
        act!(visitor.visit_before(self)?);

        match self.kind_mut() {
            StmtKind::Nop | StmtKind::Return | StmtKind::DefaultCase 
                | StmtKind::Break | StmtKind::Next => (),
            StmtKind::Expr(expr) | StmtKind::ResultIs(expr) 
                | StmtKind::Case(expr) => act!(expr.traverse(visitor)?),
            StmtKind::Block(stmts) => for stmt in stmts {
                act!(stmt.traverse(visitor)?);
            }
            StmtKind::If(cond, if_branch, else_branch) => {
                act!(cond.traverse(visitor)?);
                act!(if_branch.traverse(visitor)?);
                if let Some(else_branch) = else_branch {
                    act!(else_branch.traverse(visitor)?);
                }
            }
            StmtKind::Unless(cond, body) | StmtKind::SwitchOn(cond, body)
                | StmtKind::While(cond, body) | StmtKind::Until(cond, body) => {
                act!(cond.traverse(visitor)?);
                act!(body.traverse(visitor)?);
            }
            StmtKind::For(iter, init, bound, step, body) => {
                act!(iter.traverse(visitor)?);
                act!(init.traverse(visitor)?);
                if let Some(bound) = bound {
                    act!(bound.traverse(visitor)?);
                }
                if let Some(step) = step {
                    act!(step.traverse(visitor)?);
                }
                act!(body.traverse(visitor)?);
            }
            StmtKind::Match(cond, branches) | StmtKind::Every(cond, branches) => {
                for c in cond {
                    act!(c.traverse(visitor)?);
                }                
                for (patterns, body) in branches {
                    for pattern in patterns {
                        act!(pattern.traverse(visitor)?);
                    }
                    act!(body.traverse(visitor)?);
                }
            }
            StmtKind::Binding(pairs) => {
                for (pattern, expr) in pairs {
                    act!(pattern.traverse(visitor)?);
                    act!(expr.traverse(visitor)?);
                }
            }
        }

        visitor.visit(self)
    }
}

impl Traversable for Expr {
    fn traverse<E>(&mut self, visitor: &mut impl ASTVisitor<E>) -> Result<Action, E> {
        act!(visitor.visit_before(self)?);

        match self.kind_mut() {
            ExprKind::Ident(_) | ExprKind::Atom(_)
                | ExprKind::IntLit(_) | ExprKind::FloatLit(_)
                | ExprKind::CharLit(_) | ExprKind::StringLit(_)
                | ExprKind::True | ExprKind::False => (),
            ExprKind::Abs(expr) | ExprKind::Not(expr)
                | ExprKind::Ref(expr) | ExprKind::Deref(expr)
                | ExprKind::Cast(expr) | ExprKind::ImplicitCast(expr) => act!(expr.traverse(visitor)?),
            ExprKind::Add(lhs, rhs) | ExprKind::Sub(lhs, rhs) 
                | ExprKind::Mul(lhs, rhs) | ExprKind::Div(lhs, rhs) | ExprKind::Mod(lhs, rhs)
                | ExprKind::And(lhs, rhs) | ExprKind::Or(lhs, rhs) | ExprKind::XOr(lhs, rhs)
                | ExprKind::Eq(lhs, rhs) | ExprKind::Ne(lhs, rhs) | ExprKind::Gt(lhs, rhs)
                | ExprKind::Ge(lhs, rhs) | ExprKind::Lt(lhs, rhs) | ExprKind::Le(lhs, rhs)
                | ExprKind::LShift(lhs, rhs) | ExprKind::RShift(lhs, rhs)
                | ExprKind::Index(lhs, rhs) => {
                    act!(lhs.traverse(visitor)?);
                    act!(rhs.traverse(visitor)?);
            }
            ExprKind::Slice(lhs, mhs, rhs) | ExprKind::Conditional(lhs, mhs, rhs) => {
                act!(lhs.traverse(visitor)?);
                act!(mhs.traverse(visitor)?);
                act!(rhs.traverse(visitor)?);
            }
            ExprKind::ValOf(stmt) => act!(stmt.traverse(visitor)?),
            ExprKind::FuncCall(callee, args) => {
                act!(callee.traverse(visitor)?);
                for arg in args {
                    act!(arg.traverse(visitor)?);
                }
            }
            ExprKind::Match(cond, branches) | ExprKind::Every(cond, branches) => {
                for c in cond {
                    act!(c.traverse(visitor)?);
                }
                for (patterns, expr) in branches {
                    for pattern in patterns {
                        act!(pattern.traverse(visitor)?);
                    }
                    act!(expr.traverse(visitor)?);
                }
            }
        }
            
        visitor.visit(self)
    }
}

impl Traversable for Pattern {
    fn traverse<E>(&mut self, visitor: &mut impl ASTVisitor<E>) -> Result<Action, E> {
        act!(visitor.visit_before(self)?);

        match self {
            Self::Any | Self::Remaining | Self::Query(_) => (),
            Self::Term(PatternTerm::Range(lhs, rhs)) => {
                act!(lhs.traverse(visitor)?);
                act!(rhs.traverse(visitor)?);
            }
            Self::Term(PatternTerm::Lt(e) | PatternTerm::Le(e)
                | PatternTerm::Gt(e) | PatternTerm::Ge(e)
                | PatternTerm::Ne(e) | PatternTerm::Eq(e) 
                | PatternTerm::Basic(e)) => act!(e.traverse(visitor)?),
            Self::Or(lhs, rhs) | Self::And(lhs, rhs) => {
                act!(lhs.traverse(visitor)?);
                act!(rhs.traverse(visitor)?);
            }
            Self::List(args) | Self::Variant(_, args) => {
                for arg in args {
                    act!(arg.traverse(visitor)?);
                }
            }
        }

        visitor.visit(self)
    }
}
