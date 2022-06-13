use crate::db::Db;
use crate::replace::Replacement;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

pub fn hash(text: &str) -> [u8; 32] {
    use blake3::hash;

    hash(text.as_bytes()).into()
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct PermSkip(pub [u8; 32], pub Replacement);

pub struct PermSkipMemory {
    db: Arc<Mutex<Db>>,
    skips: HashSet<PermSkip>,
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

    pub fn add(&mut self, perm_skip: PermSkip) -> Result<(), rusqlite::Error> {
        self.skips.insert(perm_skip.clone());
        self.db
            .lock()
            .expect("db write perm skip")
            .write_perm_skip(perm_skip)
    }

    pub fn contains(&self, skip: &PermSkip) -> bool {
        self.skips.contains(skip)
    }
}
