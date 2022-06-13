use crate::db::Db;
use crate::replace::Replacement;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct SkipContent(pub blake3::Hash, pub Replacement);

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

    pub fn add(&mut self, perm_skip: SkipContent) -> Result<(), rusqlite::Error> {
        self.skips.insert(perm_skip.clone());
        self.db
            .lock()
            .expect("db write perm skip")
            .write_perm_skip(perm_skip)
    }

    pub fn contains(&self, skip: &SkipContent) -> bool {
        self.skips.contains(skip)
    }
}

impl From<crate::search::Hit> for SkipContent {
    fn from(hit: crate::search::Hit) -> Self {
        Self(hit.content_hash, hit.into())
    }
}
