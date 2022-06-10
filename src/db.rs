use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

pub struct Db {
    pub conn: Connection,
}
impl Db {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS search_replace (
                search      TEXT PRIMARY KEY,
                replace     TEXT NOT NULL   
            )",
            [],
        )?;

        let _long_insert = "INSERT INTO search_replace  (search, replace)
        VALUES 
            (?1, ?2), 
            (?3, ?4), 
            (?5, ?6),
            (?7, ?8),
            (?9, ?10),
            (?11, ?12)";

        let _long_params = params![
            "scala",
            "[[scala]]",
            "Scala",
            "[[scala]]",
            "rust",
            "[[rust]]",
            "svelte",
            "[[svelte]]",
            "Godot",
            "[[godot]]"
        ];

        let short_insert = "INSERT INTO search_replace  (search, replace)
        VALUES 
            (?1, ?2), 
            (?3, ?4),
            (?5, ?6)";
        let short_params = params![
            "scala",
            "[[scala]]",
            "rust",
            "[[rust]]",
            "Godot",
            "[[godot]]"
        ];

        conn.execute(short_insert, short_params)?;
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

fn _dummy_search_replace() -> Result<HashMap<String, String>> {
    let mut h = HashMap::new();
    h.insert("scala".to_string(), "[[scala]]".to_string());
    h.insert("rust".to_string(), "[[rust]]".to_string());
    Ok(h)
}
