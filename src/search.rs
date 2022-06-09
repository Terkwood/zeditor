use cursive::reexports::crossbeam_channel::{select, Receiver, Sender};
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

pub fn run(files_searched_s: Sender<Vec<FileSearched>>, search_files_r: Receiver<SearchFiles>) {
    loop {
        select! {
            recv(search_files_r) -> _ => {
                let result = search_files();

                files_searched_s.send(result).unwrap();
            },
        }
    }
}

const ZEDITOR_HOME: &str = env!("ZEDITOR_HOME");

pub fn search_files() -> Vec<FileSearched> {
    let mut out = vec![];

    use glob::glob;

    for entry in glob(&format!("{}/*.md", ZEDITOR_HOME)).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if let Ok(found) = search(path.as_path(), &vec!["scala", "rust"], 10) {
                    out.push(found);
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }

    out
}

pub fn search(
    path: &Path,
    terms: &[&str],
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
