use crate::db::Db;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct SkipContent {
    pub hash: blake3::Hash,
    pub start: usize,
    pub end: usize,
}

pub struct SkipRepo {
    db: Arc<Mutex<Db>>,
    skips: HashSet<SkipContent>,
}

impl SkipRepo {
    pub fn new(db: Arc<Mutex<Db>>) -> Self {
        let skips = db
            .lock()
            .expect("db perm")
            .get_skip_contents()
            .expect("load perm skips");
        Self { db, skips }
    }

    pub fn add(&mut self, skip: SkipContent) -> Result<(), rusqlite::Error> {
        self.skips.insert(skip.clone());
        self.db
            .lock()
            .expect("db write  skip")
            .write_skip_content(skip)
    }

    pub fn contains(&self, skip: &SkipContent) -> bool {
        self.skips.contains(skip)
    }
}

impl From<crate::search::Hit> for SkipContent {
    fn from(hit: crate::search::Hit) -> Self {
        let hash = hit.content_hash;
        let start = hit.start;
        let end = hit.end;
        Self { hash, start, end }
    }
}
