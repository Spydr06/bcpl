use std::{collections::{HashMap, HashSet}, fmt::Debug, hash::Hash};

use crate::source_file::{Location, Located};

use self::{types::{TypeList, TypeIndex}, expr::Expr, stmt::Stmt};

pub(crate) mod types;
pub(crate) mod expr;
pub(crate) mod stmt;

#[derive(Default, Debug)]
pub struct Program {
    sections: HashMap<String, Section>,
    types: TypeList
}

impl Program {
    pub fn add_section(&mut self, ident: &String, loc: Location) -> &mut Section {
        if !self.sections.contains_key(ident) {
            self.sections.insert(ident.clone(), Section::new(ident.clone(), loc));
        }

        self.sections.get_mut(ident).unwrap()
    }
}

#[derive(Debug)]
pub struct Section {
    loc: Location,
    ident: String,

    required: HashSet<Located<String>>,

    declarations: HashMap<String, Box<dyn Decl>>
}

impl Section {
    fn new(ident: String, loc: Location) -> Self {
        Self {
            loc,
            ident,
            required: HashSet::new(),
            declarations: HashMap::new()
        }
    }

    pub fn ident(&self) -> &String {
        &self.ident
    }

    pub fn defines(&self, ident: &String) -> Option<&Box<dyn Decl>> {
        self.declarations.get(ident)
    }

    pub fn add_require(&mut self, require: Located<String>) {
        self.required.insert(require);
    }
}

pub trait Decl: Debug {
    fn location(&self) -> &Location;
    fn ident(&self) -> &String;
    fn is_public(&self) -> bool;
}

pub trait IntoDecl: Sized {
    fn into_decl(self) -> Box<dyn Decl>;
}

#[derive(Debug)]
pub struct BasicDecl {
    loc: Location,
    is_public: bool,
    ident: String,
    value: Expr
}

impl IntoDecl for BasicDecl {
    fn into_decl(self) -> Box<dyn Decl> {
        Box::new(self)
    }
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

#[derive(Debug)]
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

impl Function {
    pub fn new(loc: Location, ident: String, params: Vec<Param>, return_type: TypeIndex, tailcall_recursive: bool, body: FunctionBody) -> Self {
        Self {
            loc,
            is_public: true,
            ident,
            required_params: required_params_of(&params),
            params,
            return_type,
            tailcall_recursive,
            body
        }
    }
}

fn required_params_of(params: &[Param]) -> u32 {
    if let Some((i, _)) = params.iter().enumerate().find(|(_, param)| param.default_value.is_some()) {
        i as u32 - 1
    }
    else {
        params.len() as u32
    }
}

impl IntoDecl for Function {
    fn into_decl(self) -> Box<dyn Decl> {
        Box::new(self)
    }
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

#[derive(Debug, Default)]
pub enum FunctionBody {
    Expr(Expr),
    Stmt(Stmt),
    #[default]
    Nothing
}

#[derive(Debug)]
pub struct Param {
    loc: Location,
    ident: String,
    typ: TypeIndex,
    default_value: Option<Expr>
}

