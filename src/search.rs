use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct FileSearched {
    pub path: PathBuf,
    pub searches: Vec<Searched>,
}

#[derive(Debug, PartialEq)]
pub struct Searched {
    pub search: String,
    pub replace: String,
    pub hits: Vec<Hit>,
}

#[derive(Debug, PartialEq)]
pub struct Hit {
    pub start: usize,
    pub end: usize,
    pub preview: String,
}

pub fn search(path: &Path, terms: &[String]) -> Result<FileSearched, bool> {
    todo!()
}
