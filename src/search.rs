use crate::db::Db;
use crate::env::ZEDITOR_HOME;
use crate::msg::Msg;
use cursive::reexports::crossbeam_channel::{select, Receiver, Sender};
use regex::Regex;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

const PEEK_SIZE: usize = 20;

#[derive(Copy, Clone)]
pub enum SearchCommand {
    SearchFiles,
    RefreshRegexs,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Hit {
    pub path: PathBuf,
    pub start: usize,
    pub end: usize,
    pub search: String,
    pub preview: Preview,
    pub content_hash: blake3::Hash,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Preview {
    pub before: String,
    pub after: String,
}

pub async fn run(
    db: Arc<Mutex<Db>>,
    files_searched_s: Sender<Vec<Hit>>,
    search_files_r: Receiver<Msg<SearchCommand>>,
) {
    let mut terms_regexs = regexs_from_db(db.clone());
    loop {
        select! {
            recv(search_files_r) -> msg => {
                match msg {
                    Ok(Msg::Event(SearchCommand::SearchFiles)) => {
                        let hits = search_files(&terms_regexs).await;

                        files_searched_s.send(hits).unwrap();
                    }
                    Ok(Msg::Event(SearchCommand::RefreshRegexs)) => {
                        terms_regexs = regexs_from_db(db.clone());
                    }
                    Ok(Msg::Quit) => {
                        break;
                    }
                    Err(e) => eprintln!("search error: {}", e),
                }

            },
        }
    }
}

pub async fn search_files(terms: &[(String, Regex)]) -> Vec<Hit> {
    let mut out: Vec<Vec<Hit>> = vec![];

    use futures::stream::StreamExt;
    use glob::glob;
    let paths = glob(&format!("{}/*.md", ZEDITOR_HOME)).expect("Failed to read glob pattern");
    let reads = futures::stream::iter(
        paths
            .into_iter()
            .map(|path| async move { search(path.expect("path"), terms, PEEK_SIZE).await }),
    )
    .buffer_unordered(16)
    .collect::<Vec<_>>();

    for r in reads.await {
        out.push(r.expect("search"));
    }

    out.iter().cloned().flatten().collect()
}

pub async fn search(
    path: PathBuf,
    terms: &[(String, Regex)],
    peek_size: usize,
) -> Result<Vec<Hit>, std::io::Error> {
    let mut file = File::open(&path).await?;
    let mut contents = String::new();

    file.read_to_string(&mut contents).await?;

    search_text(path, &contents, terms, peek_size)
}

fn regexs_from_db(db: Arc<Mutex<Db>>) -> Vec<(String, Regex)> {
    let search_replace = db
        .lock()
        .expect("search db arc lock")
        .get_search_replace()
        .expect("search db fetch");

    let terms: Vec<&str> = search_replace
        .keys()
        .into_iter()
        .map(|s| &s as &str)
        .collect();

    make_regex_vec(&terms)
}

fn make_regex(term: &str) -> Regex {
    Regex::new(&format!(r"(\s|^)({})(\s|$)", term)).unwrap()
}

fn search_text(
    path: PathBuf,
    text: &str,
    regexs: &[(String, Regex)],
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
                        path: path.clone(),
                        search: term.to_string(),
                        start,
                        end,
                        preview: Preview::new(text, start, end, peek_size),
                        content_hash: blake3::hash(text.as_bytes()),
                    })
                }
            }
        }
    }

    Ok(hits)
}

/// generate a preview of the area around our search target,
/// looking forward _and_ behind by `peek_size` chars
///
/// will not to slice in the middle of UTF8 chars
impl Preview {
    pub fn new(text: &str, start: usize, end: usize, peek_size: usize) -> Preview {
        let chars_before = &text[0..start].chars().collect::<Vec<_>>();

        let chars_after = &text[end..text.len()].chars().collect::<Vec<_>>();

        let before = chars_before
            .iter()
            .rev()
            .take(peek_size)
            .rev()
            .cloned()
            .collect::<String>();

        let after = chars_after
            .iter()
            .take(peek_size)
            .cloned()
            .collect::<String>();
        Preview { before, after }
    }

    pub fn as_text(&self, search: &str) -> String {
        format!("{}{}{}", self.before, search, self.after)
    }
}

fn make_regex_vec(terms: &[&str]) -> Vec<(String, regex::Regex)> {
    terms
        .iter()
        .map(|t| (t.to_string(), make_regex(t)))
        .collect::<Vec<(String, regex::Regex)>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_string_test() {
        let dummy_path: PathBuf = PathBuf::from("/tmp/foo");
        const DUMMY_TEXT: &str = "scala is a lang

but rust is better

i wrote something in scala today

but then i wrote it in rust";

        let actual = search_text(
            dummy_path.clone(),
            DUMMY_TEXT,
            &make_regex_vec(&["scala", "rust"]),
            3,
        )
        .unwrap();

        let content_hash = blake3::hash(DUMMY_TEXT.as_bytes());

        let expected = vec![
            Hit {
                path: dummy_path.clone(),
                search: "scala".to_string(),
                start: 0,
                end: 5,
                preview: Preview {
                    before: "".to_string(),
                    after: " is".to_string(),
                },
                content_hash: content_hash.clone(),
            },
            Hit {
                path: dummy_path.clone(),
                search: "scala".to_string(),
                start: 58,
                end: 63,
                preview: Preview {
                    before: "in ".to_string(),
                    after: " to".to_string(),
                },
                content_hash: content_hash.clone(),
            },
            Hit {
                path: dummy_path.clone(),
                search: "rust".to_string(),
                start: 21,
                end: 25,
                preview: Preview {
                    before: "ut ".to_string(),
                    after: " is".to_string(),
                },
                content_hash: content_hash.clone(),
            },
            Hit {
                path: dummy_path.clone(),
                search: "rust".to_string(),
                start: 94,
                end: 98,
                preview: Preview {
                    before: "in ".to_string(),
                    after: "".to_string(),
                },
                content_hash: content_hash.clone(),
            },
        ];
        assert_eq!(actual, expected);
    }
}
