use crate::env::ZEDITOR_HOME;
use rusqlite::{ Connection, Result};
use std::collections::HashMap;
use std::path::{ PathBuf};

const FILENAME: &str = ".zeditor.db";

pub struct Db {
    pub conn: Connection,
}
impl Db {
    pub fn new() -> Result<Self> {
        let conn = Connection::open(path_to_db().as_path())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS search_replace (
                search      TEXT PRIMARY KEY,
                replace     TEXT NOT NULL   
            ) STRICT",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn get_search_replace(&self) -> Result<HashMap<String, String>> {
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

fn path_to_db() -> PathBuf {
    let mut pb = PathBuf::new();
    pb.push(ZEDITOR_HOME);
    pb.push(FILENAME);
    pb
}
