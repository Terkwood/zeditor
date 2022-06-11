use crate::db::Db;
use crate::search::Hit;
use cursive::reexports::crossbeam_channel::{select, Receiver, Sender};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub struct ReplaceHits(pub Vec<Hit>);

#[derive(Copy, Clone)]
pub enum HitsReplaced {
    Success,
    Failure,
}

pub async fn run(
    db: Arc<Mutex<Db>>,
    hits_replaced_s: Sender<HitsReplaced>,
    replace_hits_r: Receiver<ReplaceHits>,
) {
    let search_replace = db
        .lock()
        .expect("replace db arc lock")
        .get_search_replace()
        .expect("replace db fetch");

    loop {
        select! {
            recv(replace_hits_r) -> msg => {
                if let Ok(ReplaceHits(hits)) = msg {
                    let result = if let Ok(_) = replace(&hits, &search_replace).await {
                        HitsReplaced::Success
                    } else {
                        HitsReplaced::Failure
                    };

                    hits_replaced_s.send(result).expect("send");
                }

            },
        }
    }
}

async fn replace(hits: &[Hit], sr_terms: &HashMap<String, String>) -> Result<(), std::io::Error> {
    let mut hits_by_file: HashMap<PathBuf, Vec<Hit>> = HashMap::new();

    for h in hits {
        let mut a: Vec<Hit> = hits_by_file.get(&h.path).unwrap_or(&Vec::new()).to_vec();
        a.push(h.clone());
        hits_by_file.insert(h.path.clone(), a.clone());
    }

    use futures::stream::StreamExt;
    let done = futures::stream::iter(
        hits_by_file
            .into_iter()
            .map(|(path, hits)| async move { replace_file(path, &hits, sr_terms).await }),
    )
    .buffer_unordered(16)
    .collect::<Vec<_>>();

    for _ in done.await {}

    Ok(())
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone)]
struct Replacement {
    pub start: usize,
    pub end: usize,
    pub term: String,
}

impl Replacement {
    /// transform search result into a byte-aware replacement
    pub fn from(hit: &Hit, sr_terms: &HashMap<String, String>) -> Option<Self> {
        if let Some(replacement) = sr_terms.get(&hit.search) {
            Some(Self {
                start: hit.start,
                end: hit.end,
                term: replacement.to_string(),
            })
        } else {
            None
        }
    }
}

async fn replace_file(
    path: PathBuf,
    hits: &[Hit],
    sr_terms: &HashMap<String, String>,
) -> Result<(), std::io::Error> {
    let mut replacements = vec![];
    for h in hits {
        if let Some(r) = Replacement::from(h, sr_terms) {
            replacements.push(r);
        }
    }

    let input_text = tokio::fs::read_to_string(&path).await?;

    let output_text = replace_text(&input_text, &replacements);

    // truncate and then completely rewrite file
    let mut file = File::create(path.as_path()).await?;
    file.write_all(&output_text.as_bytes()).await?;
    file.sync_all().await?;

    Ok(())
}

fn replace_text(text: &str, replacements: &[Replacement]) -> String {
    let rs: Vec<Replacement> = {
        let mut rs: Vec<Replacement> = replacements.iter().cloned().collect();
        rs.sort();
        rs
    };

    let mut out = String::new();
    let mut last: usize = 0;

    // we need to take care not to split unicode characters in the
    // middle of their sequences
    // see https://stackoverflow.com/a/51983601/9935916
    let utf8 = text.chars().collect::<Vec<_>>();
    for r in rs {
        out.push_str(&utf8[last..r.start].iter().cloned().collect::<String>());
        out.push_str(&r.term);

        last = r.end;
    }

    out.push_str(&utf8[last..].iter().cloned().collect::<String>());

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_text_test() {
        let input = "the quick brown dog jumped over the tiny jvm";
        let replacements = vec![
            Replacement {
                term: "a".to_string(),
                start: 0,
                end: 3,
            },
            Replacement {
                term: "a".to_string(),
                start: 32,
                end: 35,
            },
            Replacement {
                term: "turtle".to_string(),
                start: 41,
                end: 44,
            },
        ];

        let expected = "a quick brown dog jumped over a tiny turtle";
        assert_eq!(replace_text(input, &replacements), expected.to_string());
    }

    #[test]
    fn utf8_test() {
        let input = "🌎🍌🐶🐮💘";
        let replacements = vec![Replacement {
            term: "foo".to_string(),
            start: 1,
            end: 2,
        }];

        let expected = "🌎foo🐶🐮💘";
        assert_eq!(replace_text(input, &replacements), expected.to_string());
    }
}
