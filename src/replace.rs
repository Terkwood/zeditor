use crate::search::Hit;
use std::collections::HashMap;
use std::path::PathBuf;

pub async fn replace(
    hits: &[Hit],
    sr_terms: &HashMap<String, String>,
) -> Result<(), std::io::Error> {
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

    let new_test = replace_text(todo!("read from file"), &replacements);
    todo!("tokio async write fs");
}

fn replace_text(text: &str, replacements: &[Replacement]) -> String {
    let rs: Vec<Replacement> = {
        let mut rs: Vec<Replacement> = replacements.iter().cloned().collect();
        rs.sort();
        rs
    };

    let mut out = String::new();
    let mut last: usize = 0;
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
}
