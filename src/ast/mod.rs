use std::{collections::{HashMap, HashSet}, fmt::Debug, any::Any};

use crate::source_file::{Location, Located};

use self::{types::{TypeList, TypeIndex}, expr::{Expr, AtomIndex}, stmt::Stmt, pattern::Pattern, visitor::Traversable};

pub(crate) mod types;
pub(crate) mod expr;
pub(crate) mod stmt;
pub(crate) mod pattern;
pub(crate) mod visitor;

#[derive(Default, Debug)]
pub struct Program {
    sections: HashMap<String, Section>,
    types: TypeList,

    next_atom_index: AtomIndex,
    atoms: HashMap<String, AtomIndex>
}

impl Program {
    pub fn add_section(&mut self, section: Section) {
        if !self.sections.contains_key(section.ident()) {
            self.sections.insert(section.ident().clone(), section);
        }
        else {
            todo!()
        }
    }

    pub fn add_atom(&mut self, atom: String) -> AtomIndex {
        if let Some(index) = self.atoms.get(&atom) {
            *index
        }
        else {
            self.atoms.insert(atom, self.next_atom_index);
            self.next_atom_index += 1;
            self.next_atom_index - 1
        }
    }

    pub fn types(&self) -> &TypeList {
        &self.types
    }

    pub fn types_mut(&mut self) -> &mut TypeList {
        &mut self.types
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
    pub fn new(ident: String, loc: Location) -> Self {
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

    pub fn declare(&mut self, decl: Box<dyn Decl>) {
        self.declarations.insert(decl.ident().clone(), decl);
    }
}

pub trait Decl: Debug {
    fn location(&self) -> &Location;
    fn ident(&self) -> &String;
    fn is_public(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

pub trait IntoDecl: Sized {
    fn into_decl(self) -> Box<dyn Decl>;
}

#[macro_export]
macro_rules! match_decl {
    // immutable
    ($decl: ident,) => {
        {}
    };
    ($decl: ident, , _ => $else_body: expr) => {
        { $else_body }
    };
    ($decl: ident, $id: ident as $typ: ty => $body: expr $(, $_id: ident as $_typ: ty => $_body: expr)* $(, _ => $else_body: expr)?) => {
        if let Some($id) = $decl.downcast_ref::<$typ>() {
            $body
        } else {
            match_decl!($decl, $($_id as $_typ => $_body),* $(, _ => $else_body)?)
        }
    };
    ($decl: expr; $($id: ident as $typ: ty => $body: expr),* $(, _ => $else_body: expr)?) => {
        let any_decl = ($decl).as_any();
        match_decl!(any_decl, $($id as $typ => $body),* $(, _ => $else_body)?);
    };
    
    // mutable
    (mut $decl: ident,) => {
        {}
    };
    (mut $decl: ident, , _ => $else_body: expr) => {
        { $else_body }
    };
    (mut $decl: ident, $id: ident as $typ: ty => $body: expr $(, $_id: ident as $_typ: ty => $_body: expr)* $(, _ => $else_body: expr)?) => {
        if let Some($id) = $decl.downcast_mut::<$typ>() {
            $body
        } else {
            match_decl!(mut $decl, $($_id as $_typ => $_body),* $(, _ => $else_body)?)
        }
    };
    (mut $decl: expr; $($id: ident as $typ: ty => $body: expr),* $(, _ => $else_body: expr)?) => {
        let any_decl = ($decl).as_mut_any();
        match_decl!(mut any_decl, $($id as $typ => $body),* $(, _ => $else_body)?);
    };
}

#[derive(Debug)]
pub struct ManifestDecl {
    loc: Location,
    is_public: bool,

    ident: String,

    value: Expr
}

impl Decl for ManifestDecl {
    fn ident(&self) -> &String {
        &self.ident
    }

    fn location(&self) -> &Location {
        &self.loc
    }

    fn is_public(&self) -> bool {
        self.is_public
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct Function {
    loc: Location,
    is_public: bool,

    ident: String,

    params: Vec<Param>,
    required_params: u32,

    return_type: Option<TypeIndex>,
    tailcall_recursive: bool, // recursiveness indicated by the `and` declaration

    body: FunctionBody
}

impl Function {
    pub fn new(loc: Location, ident: String, params: Vec<Param>, return_type: Option<TypeIndex>, tailcall_recursive: bool, body: FunctionBody) -> Self {
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub enum FunctionBody {
    Expr(Expr),
    Stmt(Stmt),
    PatternMatchedExpr(Vec<(Vec<Located<Pattern>>, Expr)>),
    PatternMatchedStmt(Vec<(Vec<Located<Pattern>>, Stmt)>),
}

impl From<BasicFunctionBody> for FunctionBody {
    fn from(value: BasicFunctionBody) -> Self {
        match value {
            BasicFunctionBody::Expr(expr) => Self::Expr(expr),
            BasicFunctionBody::Stmt(stmt) => Self::Stmt(stmt)
        }
    }
}

#[derive(Debug)]
pub enum BasicFunctionBody {
    Expr(Expr),
    Stmt(Stmt)
}

#[derive(Debug)]
pub struct Param {
    loc: Location,
    ident: Located<Pattern>,
    typ: Option<TypeIndex>,
    default_value: Option<Expr>
}

impl Param {
    pub fn new(loc: Location, ident: Located<Pattern>, typ: Option<TypeIndex>, default_value: Option<Expr>) -> Self {
        Self {
            loc,
            ident,
            typ,
            default_value
        }
    }
}

