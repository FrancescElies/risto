use std::time::Duration;

use directories::ProjectDirs;

use crate::{AcoustId, FileHash};

#[derive(Debug, Default)]
pub struct Db {
    tree: Option<sled::Db>,
}

macro_rules! key_duration {
    ($e:expr) => {{
        format!("{}-duration", $e)
    }};
}

macro_rules! key_acoustid {
    ($e:expr) => {{
        format!("{}-acoustid", $e)
    }};
}

impl Db {
    pub fn new() -> Self {
        match ProjectDirs::from("", "music-rater", "risto") {
            Some(appdir) => Self {
                tree: sled::open(appdir.data_dir()).ok(),
            },
            None => Self { tree: None },
        }
    }

    pub fn get_duration(&self, key: &FileHash) -> Option<Duration> {
        let Some(tree) = &self.tree else {
            return None;
        };
        let key = key_duration!(key);
        let res = tree.get(&key).ok()??;
        let res = Duration::from_secs(u64::from_be_bytes(res.as_ref().try_into().ok()?));
        eprintln!("cache-hit for key={key}, {res:?}");
        Some(res)
    }

    pub fn insert_duration(&self, key: &FileHash, duration: Duration) -> Option<Duration> {
        let Some(tree) = &self.tree else {
            return None;
        };
        let key = key_duration!(key);
        let old = tree.insert(key, &duration.as_secs().to_be_bytes()).ok()??;
        Some(Duration::from_secs(u64::from_be_bytes(
            old.as_ref().try_into().ok()?,
        )))
    }

    pub fn get_acoustid(&self, key: &FileHash) -> Option<AcoustId> {
        let Some(tree) = &self.tree else {
            return None;
        };
        let key = key_acoustid!(key);
        let res = tree.get(&key).ok()??;
        let res = std::str::from_utf8(res.as_ref()).unwrap().to_string();
        let short_id: String = res.chars().take(10).collect();
        eprintln!("cache-hit for key={key}, {short_id}...");
        Some(AcoustId(res))
    }

    pub fn insert_acoustid(&self, key: &FileHash, id: AcoustId) -> Option<AcoustId> {
        let Some(tree) = &self.tree else {
            return None;
        };
        let key = key_acoustid!(key);
        let old = tree.insert(key, id.0.as_str()).ok()??;
        Some(AcoustId(
            std::str::from_utf8(old.as_ref()).unwrap().to_string(),
        ))
    }
}
