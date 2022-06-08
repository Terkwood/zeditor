use std::path::{Path, PathBuf};
use zeditor::search::*;

#[test]
fn test_search() {
    let test_data = format!("{}/tests/search.txt", env!("CARGO_MANIFEST_DIR"));
    let path = Path::new(&test_data);
    let actual = search(&path, &vec!["scala".to_string(), "rust".to_string()]);

    let expected = Ok(FileSearched {
        path: PathBuf::from(path),
        searches: vec![],
    });
    assert_eq!(actual, expected);
}
