use std::{
    io::Read,
    ops::{Deref, DerefMut}, 
    fmt::{Formatter, Debug}
};

pub type SourceFileId = u32;

#[derive(Debug)]
pub struct SourceFile {
    id: SourceFileId,
    path: String,
    contents: String,
    line_indices: Vec<usize>,
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
            line_indices: contents.match_indices('\n').map(|(i, _)| i).collect(),
            contents
        })
    }

    pub fn contents(&self) -> &String {
        &self.contents
    }

    pub fn line(&self, line_num: usize) -> Option<&str> {
        Some(&self.contents[
             *self.line_indices.get(line_num - 1)?..
             self.line_indices.get(line_num).copied().unwrap_or(self.contents.len())
        ])
    }

    fn id(&self) -> SourceFileId {
        self.id
    }

    pub fn path(&self) -> &String {
        &self.path
    }
}

#[derive(Clone)]
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

pub trait WithLocation: Sized {
    fn with_location(self, loc: Location) -> Located<Self> {
        Located::with_location(self, loc)
    }
}

