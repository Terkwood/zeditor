use crate::db::Db;
use crate::msg::Msg;
use crate::search::Hit;
use cursive::reexports::crossbeam_channel::{select, Receiver, Sender};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Clone)]
pub enum ReplaceCommand {
    ReplaceHits(Vec<Hit>),
    RefreshSearchReplace,
}

#[derive(Copy, Clone)]
pub enum HitsReplaced {
    Success,
    Failure,
}

pub async fn run(
    db: Arc<Mutex<Db>>,
    hits_replaced_s: Sender<HitsReplaced>,
    replace_hits_r: Receiver<Msg<ReplaceCommand>>,
) {
    let mut search_replace = db
        .lock()
        .expect("replace db arc lock")
        .get_search_replace()
        .expect("replace db fetch");

    loop {
        select! {
            recv(replace_hits_r) -> msg => {
                match msg {
                    Ok(Msg::Event(ReplaceCommand::ReplaceHits(hits))) =>{
                        let result = if let Ok(_) = replace(&hits, &search_replace).await {
                            HitsReplaced::Success
                        } else {
                            HitsReplaced::Failure
                        };

                        hits_replaced_s.send(result).expect("send");
                    }

                    Ok(Msg::Event(ReplaceCommand::RefreshSearchReplace)) => {
                        search_replace = db
                            .lock()
                            .expect("replace db arc lock")
                            .get_search_replace()
                            .expect("replace db fetch");
                    }

                    Ok(Msg::Quit) => {
                        break;
                    }

                    Err(e) => eprintln!("{}",e)
                }


            },
        }
    }
}

async fn replace(hits: &[Hit], sr_terms: &HashMap<String, String>) -> Result<(), std::io::Error> {
    // prevent replacing the same position twice
    // see https://github.com/Terkwood/zeditor/issues/36
    let hits_no_duplicate_starts = hits
        .iter()
        .fold(
            HashMap::new(),
            |mut acc: HashMap<(PathBuf, usize), Hit>, hit| {
                if !acc.contains_key(&(hit.path.clone(), hit.start)) {
                    acc.insert((hit.path.clone(), hit.start), hit.clone());
                }

                acc
            },
        )
        .values()
        .cloned()
        .collect::<Vec<_>>();

    let mut hits_by_file: HashMap<PathBuf, Vec<Hit>> = HashMap::new();

    for h in hits_no_duplicate_starts {
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

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Hash)]
pub struct Replacement {
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

impl From<crate::search::Hit> for Replacement {
    fn from(hit: crate::search::Hit) -> Self {
        Self {
            start: hit.start,
            term: hit.search,
            end: hit.end,
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
    for r in rs {
        out.push_str(&text[last..r.start]);
        out.push_str(&r.term);

        last = r.end;
    }

    out.push_str(&text[last..]);

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
        // note that we carefully selected start and end values here,
        // based on the UNICODE characters' byte lengths
        let input = "????????????????????";
        let replacements = vec![Replacement {
            term: "foo".to_string(),
            start: 4,
            end: 8,
        }];

        let expected = "????foo????????????";
        assert_eq!(replace_text(input, &replacements), expected.to_string());
    }
}
