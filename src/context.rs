use std::collections::HashMap;

use colorize::AnsiColor;

use crate::{terminate, source_file::{SourceFile, SourceFileId}, token::lexer::Lexer, ast::{self, parser::Parser}};

#[derive(Default)]
pub enum BuildKind {
    #[default]
    Executable,
    Object,
    SharedObject
}

impl BuildKind {
    fn ext(&self, os: &str) -> Option<&'static str> {
        match os {
            "linux" | "macos" | "unix" => Some(self.ext_unix()),
            "windows" => Some(self.ext_windows()),
            _ => None
        }
    }

    fn ext_unix(&self) -> &'static str {
        match self {
            Self::Executable => "",
            Self::Object => ".o",
            Self::SharedObject => ".so"
        }
    }

    fn ext_windows(&self) -> &'static str {
        match self {
            Self::Executable => ".exe",
            Self::Object => ".lib",
            Self::SharedObject => ".dll"
        }
    }
}

#[derive(Default)]
pub enum OutputFile {
    Name(String),
    #[default]
    Default
}

impl OutputFile {
    pub fn to_filename(self, build_kind: &BuildKind) -> String {
        match self {
            Self::Name(filename) => filename,
            Self::Default => format!("a{}", build_kind.ext(std::env::consts::OS).expect("invalid operating system"))
        }
    }
}

#[derive(Default)]
pub struct Context {
    program_name: String,
    output_file: OutputFile,

    build_kind: BuildKind,
    tags: Vec<String>,

    source_files: HashMap<SourceFileId, SourceFile>,

    ast: ast::Program
}

impl Context {
    pub fn from_program_name(program_name: String) -> Self {
        let mut ctx = Self::default();
        ctx.program_name = program_name;
        ctx
    }
    
    pub fn set_output_file(&mut self, output_file: String) {
        self.output_file = OutputFile::Name(output_file);
    }

    pub fn program_name(&self) -> &String {
        &self.program_name
    }

    pub fn define_tag(&mut self, tag: String) {
        self.tags.push(tag);
    }

    pub fn set_build_kind(&mut self, build_kind: BuildKind) {
        self.build_kind = build_kind;
    }

    pub fn add_source_files(&mut self, source_files: HashMap<SourceFileId, SourceFile>) {
        self.source_files.extend(source_files);
    }

    pub fn fatal_error(self, err: &str) -> ! {
        eprintln!("{} {} {err}",
            format!("{}:", self.program_name()).bold(),
            format!("fatal error:").bold().red()
        );
        
        terminate();
    }

    pub fn compile(mut self) {
        if self.source_files.is_empty() {
            self.fatal_error("no input files.");
        }
            
        let parsers = self.source_files.values().map(|file| Parser::from(Lexer::from(file)));
        for mut parser in parsers {
            if let Err(err) = parser.parse(&mut self.ast) {
                eprintln!("Parse Error: {err:?}");
                eprintln!("^^^ In {:?}", (**parser).path());
            }
        }
    }
}
