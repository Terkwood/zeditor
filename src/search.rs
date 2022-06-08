use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub struct FileSearched {
    pub path: PathBuf,
    pub hits: Vec<Hit>,
}

#[derive(Debug, PartialEq)]
pub struct Hit {
    pub search: String,
    pub start: usize,
    pub end: usize,
    pub preview: String,
}

pub fn search(path: &Path, terms: &[String], peek_size: usize) -> Result<FileSearched, bool> {
    todo!()
}
