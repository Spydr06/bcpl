mod scope;

use std::sync::{Arc, Mutex, MutexGuard};

use crate::{ast::{self, visitor::{ASTVisitor, Visitor, Traversable}}, source_file::Located};

use self::scope::Scope;

pub enum TypeCheckError {

}

pub struct TypeChecker<'a> {
    scope: Scope<'a>
}

impl<'a> TypeChecker<'a> {
    fn new(scope: Scope<'a>) -> Self {
        Self {
            scope
        }
    }
}

type Error = Located<TypeCheckError>;

unsafe fn get_ref<'a, T>(r: &T) -> &'a T {
    (r as *const T).as_ref().unwrap()
}

pub fn typecheck_ast(ast: Arc<Mutex<ast::Program>>) -> Result<(), Error> {
    let mut ast = ast.lock().unwrap();
    let mut typechecker = TypeChecker::new(Scope::toplevel(unsafe { get_ref(&ast) }));
    println!("{:#?}", typechecker.scope);

    ast.traverse(&mut typechecker).map(|_| ())
}

impl<'a> Visitor<ast::Program, Error> for TypeChecker<'a> {
    fn visit(&mut self, node: &mut ast::Program) -> Result<ast::visitor::Action, Error> {
        Ok(ast::visitor::Action::Continue) 
    }
}

impl<'a> Visitor<ast::Section, Error> for TypeChecker<'a> {
    fn visit(&mut self, node: &mut ast::Section) -> Result<ast::visitor::Action, Error> {
        Ok(ast::visitor::Action::Continue)
    }
}

impl<'a> Visitor<ast::Function, Error> for TypeChecker<'a> {
    fn visit(&mut self, node: &mut ast::Function) -> Result<ast::visitor::Action, Error> {
        Ok(ast::visitor::Action::Continue)
    }
}

impl<'a> Visitor<ast::Param, Error> for TypeChecker<'a> {
    fn visit(&mut self, node: &mut ast::Param) -> Result<ast::visitor::Action, Error> {
        Ok(ast::visitor::Action::Continue) 
    }
}

impl<'a> Visitor<ast::stmt::Stmt, Error> for TypeChecker<'a> {
    fn visit(&mut self, node: &mut ast::stmt::Stmt) -> Result<ast::visitor::Action, Error> {
        Ok(ast::visitor::Action::Continue) 
    } 
}

impl<'a> Visitor<ast::expr::Expr, Error> for TypeChecker<'a> {
    fn visit(&mut self, node: &mut ast::expr::Expr) -> Result<ast::visitor::Action, Error> {
        Ok(ast::visitor::Action::Continue)
    }
}

impl<'a> Visitor<ast::pattern::Pattern, Error> for TypeChecker<'a> {
    fn visit(&mut self, node: &mut ast::pattern::Pattern) -> Result<ast::visitor::Action, Error> {
        Ok(ast::visitor::Action::Continue)
    }
}
