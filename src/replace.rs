use crate::search::Hit;
use std::collections::HashMap;

pub async fn replace(
    hits: &[Hit],
    sr_terms: &HashMap<String, String>,
) -> Result<(), std::io::Error> {
    todo!("group hits by file");
    todo!("do an entire file rewrite at once");
    todo!("use tokio");
}

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone)]
struct Replacement {
    pub start: usize,
    pub end: usize,
    pub term: String,
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
