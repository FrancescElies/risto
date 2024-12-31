use anyhow::{anyhow, Ok, Result};
use directories::ProjectDirs;

use crate::AcoustId;

#[derive(Debug)]
pub struct Db {
    tree: sled::Db,
}

impl Db {
    pub fn new() -> Result<Self> {
        let appdir = ProjectDirs::from("", "music-rater", "risto")
            .ok_or(anyhow!("failed to get ProjectDirs"))?;

        Ok(Db {
            tree: sled::open(appdir.data_dir())?,
        })
    }

    pub fn get_acoustid(&self, key: &str) -> Result<AcoustId> {
        let res = self.tree.get(key)?.ok_or(anyhow!("missing"))?;
        let val = std::str::from_utf8(res.as_ref()).unwrap();
        Ok(AcoustId(val.to_string()))
    }

    pub fn insert(&self, key: &str, acoustid: AcoustId) -> Result<()> {
        let _old = self.tree.insert(key, acoustid.0.as_str())?;
        Ok(())
    }
}
