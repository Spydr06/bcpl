use std::sync::{Arc, Mutex, MutexGuard};

use crate::ast;

pub struct TypeChecker<'a> {
    ast: MutexGuard<'a, ast::Program>
}

impl<'a> TypeChecker<'a> {
    fn new(ast: MutexGuard<'a, ast::Program>) -> Self {
        Self {
            ast
        }
    }
}

fn typecheck_ast(ast: Arc<Mutex<ast::Program>>) -> Result<(), ()> {
    let mut typechecker = TypeChecker::new(ast.lock().unwrap());
    Ok(())
}
