use regex::Regex;
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
        let re = Regex::new(&format!(r"(\s|^)({})(\s|$)", t)).unwrap();
        for hit in re.find_iter(&contents) {
            let start = hit.start();
            let end = hit.end();
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
        /*
        if let Some(caps) = re.captures(&contents) {
            if let Some(exact) = caps.get(1) {
                let start = exact.start();
                let end = exact.end();
                println

                println!(
                    "{}",
                    &contents[start.checked_sub(peek_size).unwrap_or_default()
                        ..end.checked_add(peek_size).unwrap_or(contents.len())]
                );
            }
        }
        */
    }

    Ok(FileSearched {
        path: PathBuf::from(path),
        hits,
    })
}
