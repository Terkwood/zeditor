use std::path::{Path, PathBuf};
use zeditor::search::*;

#[test]
fn test_search() {
    let test_data = format!("{}/tests/search.txt", env!("CARGO_MANIFEST_DIR"));
    let path = Path::new(&test_data);
    let actual = search(&path, &vec!["scala".to_string(), "rust".to_string()], 3).unwrap();

    let expected = FileSearched {
        path: PathBuf::from(path),
        hits: vec![
            Hit {
                search: "scala".to_string(),
                start: 0,
                end: 5,
                preview: "scala is".to_string(),
            },
            Hit {
                search: "scala".to_string(),
                start: 56,
                end: 61,
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
                start: 92,
                end: 96,
                preview: "in rust".to_string(),
            },
        ],
    };
    assert_eq!(actual, expected);
}
