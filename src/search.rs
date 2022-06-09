use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct SearchFiles;

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
        let re = Regex::new(&format!(r"(\s|^)({})(\s|$)", t)).unwrap();
        for hit in re.find_iter(&contents) {
            if let Some(subcap) = re.captures(hit.as_str()) {
                if let Some(subexact) = subcap.get(2) {
                    let substart = subexact.start();
                    let subend = subexact.end() - substart;
                    println!("hit.end() {}, subend {}", hit.end(), subend);
                    let start = hit.start() + substart;
                    let end = start + subend;
                    hits.push(Hit {
                        search: t.to_string(),
                        start,
                        end,
                        preview: contents[start.checked_sub(peek_size).unwrap_or_default()
                            ..std::cmp::min(
                                end.checked_add(peek_size).unwrap_or(contents.len()),
                                contents.len(),
                            )]
                            .to_string(),
                    })
                }
            }
        }
    }

    Ok(FileSearched {
        path: PathBuf::from(path),
        hits,
    })
}
