use crate::db::Db;
use crate::replace::Replacement;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct Skip(pub blake3::Hash, pub Replacement);

pub struct PermSkipMemory {
    db: Arc<Mutex<Db>>,
    skips: HashSet<Skip>,
}

impl PermSkipMemory {
    pub fn new(db: Arc<Mutex<Db>>) -> Self {
        let skips = db
            .lock()
            .expect("db perm")
            .get_perm_skips()
            .expect("load perm skips");
        Self { db, skips }
    }

    pub fn add(&mut self, perm_skip: Skip) -> Result<(), rusqlite::Error> {
        self.skips.insert(perm_skip.clone());
        self.db
            .lock()
            .expect("db write perm skip")
            .write_perm_skip(perm_skip)
    }

    pub fn contains(&self, skip: &Skip) -> bool {
        self.skips.contains(skip)
    }
}

impl From<crate::search::Hit> for Skip {
    fn from(hit: crate::search::Hit) -> Self {
        Self(hit.content_hash, hit.into())
    }
}
