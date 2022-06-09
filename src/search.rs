use cursive::reexports::crossbeam_channel::{select, Receiver, Sender};
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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

pub async fn run(
    files_searched_s: Sender<Vec<FileSearched>>,
    search_files_r: Receiver<SearchFiles>,
) {
    loop {
        select! {
            recv(search_files_r) -> _ => {
                let terms = make_regex_vec(vec!["scala", "rust"]);

                let result = search_files(&terms).await;

                files_searched_s.send(result).unwrap();
            },
        }
    }
}

fn make_regex_vec(terms: Vec<&str>) -> Vec<(&str, regex::Regex)> {
    terms
        .iter()
        .map(|t| (*t, make_regex(t)))
        .collect::<Vec<(&str, regex::Regex)>>()
}

const ZEDITOR_HOME: &str = env!("ZEDITOR_HOME");

pub async fn search_files(terms: &Vec<(&str, Regex)>) -> Vec<FileSearched> {
    let mut out = vec![];

    use futures::stream::StreamExt;
    use glob::glob;
    let paths = glob(&format!("{}/*.md", ZEDITOR_HOME)).expect("Failed to read glob pattern");
    let reads = futures::stream::iter(
        paths
            .into_iter()
            .map(|path| async move { search(path.expect("path").as_path(), terms, 10).await }),
    )
    .buffer_unordered(16)
    .collect::<Vec<_>>();

    for r in reads.await {
        out.push(r.expect("search"));
    }

    out
}

pub async fn search(
    path: &Path,
    terms: &Vec<(&str, Regex)>,
    peek_size: usize,
) -> Result<FileSearched, std::io::Error> {
    let mut file = File::open(path).await?;
    let mut contents = String::new();

    file.read_to_string(&mut contents).await?;

    let hits = search_text(&contents, terms, peek_size);

    Ok(FileSearched {
        path: PathBuf::from(path),
        hits: hits?,
    })
}

fn make_regex(term: &str) -> Regex {
    Regex::new(&format!(r"(\s|^)({})(\s|$)", term)).unwrap()
}

fn search_text(
    text: &str,
    regexs: &Vec<(&str, Regex)>,
    peek_size: usize,
) -> Result<Vec<Hit>, std::io::Error> {
    let mut hits: Vec<Hit> = vec![];
    for (term, re) in regexs {
        for hit in re.find_iter(&text) {
            if let Some(subcap) = re.captures(hit.as_str()) {
                if let Some(subexact) = subcap.get(2) {
                    let substart = subexact.start();
                    let subend = subexact.end() - substart;
                    let start = hit.start() + substart;
                    let end = start + subend;
                    hits.push(Hit {
                        search: term.to_string(),
                        start,
                        end,
                        preview: text[start.checked_sub(peek_size).unwrap_or_default()
                            ..std::cmp::min(
                                end.checked_add(peek_size).unwrap_or(text.len()),
                                text.len(),
                            )]
                            .to_string(),
                    })
                }
            }
        }
    }

    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_string_test() {
        const DUMMY: &str = "scala is a lang

but rust is better

i wrote something in scala today

but then i wrote it in rust";

        let actual = search_text(DUMMY, &make_regex_vec(vec!["scala", "rust"]), 3).unwrap();

        let expected = vec![
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
        ];
        assert_eq!(actual, expected);
    }
}
