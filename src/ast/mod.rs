use std::collections::HashMap;

use crate::source_file::{Location, Located};

use self::{types::{TypeList, TypeIndex}, expr::Expr, stmt::Stmt};

pub(crate) mod parser;
pub(crate) mod types;
pub(crate) mod expr;
pub(crate) mod stmt;

#[derive(Default)]
pub struct Program {
    sections: HashMap<String, Section>,
    types: TypeList
}

pub struct Section {
    loc: Location,
    ident: String,

    required: Vec<Located<String>>,

    declarations: Vec<Box<dyn Decl>>
}

pub trait Decl {
    fn location(&self) -> &Location;
    fn ident(&self) -> &String;
    fn is_public(&self) -> bool;
}

pub struct BasicDecl {
    loc: Location,
    is_public: bool,
    ident: String,
    value: Expr
}

impl Decl for BasicDecl {
    fn location(&self) -> &Location {
        &self.loc
    }

    fn ident(&self) -> &String {
        &self.ident
    }

    fn is_public(&self) -> bool {
        self.is_public
    }
}

pub struct Function {
    loc: Location,
    is_public: bool,

    ident: String,

    params: Vec<Param>,
    required_params: u32,

    return_type: TypeIndex,
    tailcall_recursive: bool, // recursiveness indicated by the `and` declaration

    body: FunctionBody
}

impl Decl for Function {
    fn location(&self) -> &Location {
        &self.loc
    }

    fn ident(&self) -> &String {
        &self.ident
    }

    fn is_public(&self) -> bool {
        self.is_public
    }
}

pub enum FunctionBody {
    Expr(Expr),
    Stmt(Stmt)
}

pub struct Param {
    loc: Location,
    ident: String,
    typ: TypeIndex,
    default_value: Option<Expr>
}

