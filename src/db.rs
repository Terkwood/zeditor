use crate::env::ZEDITOR_HOME;
use crate::perm_skip::PermSkip;
use crate::replace::Replacement;
use rusqlite::{params, Connection, DatabaseName, Result};
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
            "CREATE TABLE IF NOT EXISTS perm_skip (
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

    pub fn write_perm_skip(&self, skip: PermSkip) -> Result<()> {
        self.conn.execute(
            "INSERT INTO perm_skip (hash, start, end, search) 
                    VALUES (?1, ?2, ?3, ?4)",
            params![skip.0, skip.1.start, skip.1.end, skip.1.search],
        )?;

        let row_id = self.conn.last_insert_rowid();
        let mut blob =
            self.conn
                .blob_open(DatabaseName::Main, "perm_skip", "hash", row_id, false)?;

        blob.write_at(&skip.0, 0)
    }

    pub fn get_perm_skips(&self) -> Result<HashSet<PermSkip>> {
        let mut stmt = self
            .conn
            .prepare("SELECT hash, start, end, search FROM perm_skip")?;
        let mut rows = stmt.query([])?;

        let mut out = HashSet::new();
        while let Some(row) = rows.next()? {
            out.insert(PermSkip(
                row.get(0)?,
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
