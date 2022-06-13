use crate::env::ZEDITOR_HOME;
use crate::replace::Replacement;
use crate::skip::SkipContent;
use rusqlite::{params, Connection, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS skip_content (
            hash    BLOB NOT NULL,
            start   INTEGER NOT NULL,
            end     INTEGER NOT NULL,
            search  TEXT NOT NULL,
            PRIMARY KEY (hash, start, end, search)
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

    pub fn write_perm_skip(&self, skip: SkipContent) -> Result<()> {
        self.conn.execute(
            "INSERT INTO skip_content (hash, start, end, search) 
                    VALUES (?1, ?2, ?3, ?4)",
            params![skip.0.as_bytes(), skip.1.start, skip.1.end, skip.1.search],
        )?;
        Ok(())
    }

    pub fn get_skip_contents(&self) -> Result<HashSet<SkipContent>> {
        let mut stmt = self
            .conn
            .prepare("SELECT hash, start, end, search FROM skip_content")?;
        let mut rows = stmt.query([])?;

        let mut out = HashSet::new();
        while let Some(row) = rows.next()? {
            let hash_bytes: [u8; 32] = row.get(0)?;
            let hash: blake3::Hash = hash_bytes.into();
            out.insert(SkipContent(
                hash,
                Replacement {
                    start: row.get(1)?,
                    end: row.get(2)?,
                    search: row.get(3)?,
                },
            ));
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
