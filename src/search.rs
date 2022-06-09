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
            recv(search_files_r) -> _cmd => {
                let result = search_files();

                files_searched_s.send(result).unwrap()},
        }
    }
}

pub fn search_files() -> Vec<FileSearched> {
    std::thread::sleep(std::time::Duration::from_millis(500));
    vec![FileSearched {
        path: PathBuf::from("/tmp/foo"),
        hits: vec![
            Hit {
                search: "scala".to_string(),
                start: 0,
                end: 5,
                preview: "scala is".to_string(),
            },
            Hit {
                search: "scala".to_string(),
                start: 58,
                end: 63,
                preview: "in scala to".to_string(),
            },
            Hit {
                search: "rust".to_string(),
                start: 21,
                end: 25,
                preview: "ut rust is".to_string(),
            },
            Hit {
                search: "rust".to_string(),
                start: 94,
                end: 98,
                preview: "in rust".to_string(),
            },
        ],
    }]
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
