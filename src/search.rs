use std::fs::File;
use std::io::Read;
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

pub fn search(
    path: &Path,
    terms: &[String],
    peek_size: usize,
) -> Result<FileSearched, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    let mut hits: Vec<Hit> = vec![];

    file.read_to_string(&mut contents)?;
    for t in terms {
        todo!("search individual term");
        todo!("then push to hits");
    }

    Ok(FileSearched {
        path: PathBuf::from(path),
        hits,
    })
}
