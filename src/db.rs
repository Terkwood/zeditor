use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

pub struct Db {
    pub conn: Connection,
}
impl Db {
    fn get_search_replace(&self) -> Result<HashMap<String, String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT search, replace FROM search_replace")?;
        let mut rows = stmt.query([])?;

        let mut out = HashMap::new();
        while let Some(row) = rows.next()? {
            out.insert(row.get(0)?, row.get(1)?);
        }

        Ok(out)
    }
}

pub fn lookup_search_replace() -> Result<HashMap<String, String>> {
    let mut h = HashMap::new();
    h.insert("scala".to_string(), "[[scala]]".to_string());
    h.insert("rust".to_string(), "[[rust]]".to_string());
    Ok(h)
}
