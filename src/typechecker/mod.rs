use std::sync::{Arc, Mutex, MutexGuard};

use crate::{ast::{self, visitor::{ASTVisitor, Visitor, Traversable}}, source_file::Located};

pub enum TypeCheckError {

}

pub struct TypeChecker<'a> {
    ast: &'a mut ast::Program
}

impl<'a> TypeChecker<'a> {
    fn new(ast: &'a mut ast::Program) -> Self {
        Self {
            ast
        }
    }
}

type Error = Located<TypeCheckError>;

pub fn typecheck_ast(ast: Arc<Mutex<ast::Program>>) -> Result<(), Error> {
    let mut ast = ast.lock().unwrap();
    let mut typechecker = TypeChecker::new(unsafe {
        (&ast as &ast::Program as *const _ as *mut ast::Program).as_mut().unwrap()
    });
//    ast.traverse(&mut typechecker).map(|_| ())
    Ok(())
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
