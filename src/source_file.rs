use std::{
    io::Read,
    ops::{Deref, DerefMut}, 
    fmt::{Formatter, Debug}, hash::Hash
};

use colorize::AnsiColor;

use crate::error::{IntoCompilerError, CompilerError};

pub type SourceFileId = u32;

#[derive(Debug)]
pub struct SourceFile {
    id: SourceFileId,
    path: String,
    contents: String,
    lines: Vec<String>
}

impl SourceFile {
    pub fn read(path: String, id: SourceFileId) -> std::io::Result<Self> {
        let mut file = std::fs::File::open(path.clone())?;
        
        let mut contents = String::new();
        contents.reserve(file.metadata().unwrap().len() as usize);

        file.read_to_string(&mut contents)?;

        Ok(Self {
            id,
            path,
            lines: contents.split('\n').map(|e| e.to_string()).collect(),
            contents
        })
    }

    pub fn contents(&self) -> &String {
        &self.contents
    }

    pub fn line(&self, line_num: usize) -> Option<&String> {
        self.lines.get(line_num - 1)
    }

    fn id(&self) -> SourceFileId {
        self.id
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn highlight_error(&self, err: Located<impl IntoCompilerError>) {
        let loc = err.location().clone();
        let err: CompilerError = err.unwrap().into();        
        println!("{} {}:{}:{}: {}", err.severity(), self.path(), loc.line, loc.column, err.message());
        print!("{} {} ", format!(" {: >4}", loc.line).bold().b_black(), "|".b_black());

        let line = self.line(loc.line as usize).unwrap();
        let mark_start = loc.column as usize;
        let mark_end = (loc.column + loc.width) as usize;
        println!("{}{}{}", &line[..mark_start], (&line[mark_start..mark_end]).to_owned().bold().b_yellow(), &line[mark_end..]);

        print!("      {} {}{}", "|".b_black(), " ".repeat(mark_start), "~".repeat(loc.width as usize).yellow());

        if let Some(hint) = err.hint() {
            print!(" {} {} {}", "<-".b_black(), "hint:".bold().b_grey(), hint.clone().b_grey());
        }

        println!();

        for additional in err.additional {
            self.highlight_error(additional) 
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Location {
    source_file_id: SourceFileId,
    line: u32,
    column: u32,
    width: u32
}

impl Location {
    pub fn new(source_file: &SourceFile, line: usize, column: usize, width: usize) -> Self {
        Self {
            source_file_id: source_file.id(),
            line: line as u32,
            column: column as u32,
            width: width as u32
        }
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width as u32;
    }

    pub fn file_id(&self) -> SourceFileId {
        self.source_file_id
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<id {}>:{}:{}-{}", self.source_file_id, self.line, self.column, self.column + self.width) 
    }
}

#[derive(Clone)]
pub struct Located<T> {
    inner: T,
    loc: Location
}

impl<T> Located<T> {
    pub fn with_location(inner: T, loc: Location) -> Self {
        Self {
            inner,
            loc
        }
    }

    pub fn location(&self) -> &Location {
        &self.loc
    }

    pub fn map<U>(self, func: impl FnOnce(T) -> U) -> Located<U> {
        Located {
            inner: func(self.inner),
            loc: self.loc
        }
    }

    pub fn unwrap(self) -> T {
        self.inner
    }
}

impl<T> Deref for Located<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Located<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T: Debug> Debug for Located<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {:?} @ <{:?}> }}", self.inner, self.loc)
    }
}

impl<T> Eq for Located<T> where T: PartialEq + Eq {}
impl<T: PartialEq> PartialEq for Located<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Hash> Hash for Located<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

pub trait WithLocation: Sized {
    fn with_location(self, loc: Location) -> Located<Self> {
        Located::with_location(self, loc)
    }
}

impl WithLocation for String {}
