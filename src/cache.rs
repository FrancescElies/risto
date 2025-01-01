use std::time::Duration;

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

    pub fn get_duration(&self, key: &str) -> Result<Duration> {
        let key = format!("{key}-duration");
        let res = self.tree.get(&key)?.ok_or(anyhow!("missing"))?;
        eprintln!("cache-hit for key={key}");
        Ok(Duration::from_secs(u64::from_be_bytes(
            res.as_ref().try_into()?,
        )))
    }

    pub fn insert_duration(&self, key: &str, duration: Duration) -> Result<()> {
        let _old = self.tree.insert(key, &duration.as_secs().to_be_bytes())?;
        Ok(())
    }

    pub fn get_acoustid(&self, key: &str) -> Result<AcoustId> {
        let key = format!("{key}-acoustid");
        let res = self.tree.get(&key)?.ok_or(anyhow!("missing"))?;
        eprintln!("cache-hit for key={key}");
        let val = std::str::from_utf8(res.as_ref()).unwrap();
        Ok(AcoustId(val.to_string()))
    }

    pub fn insert_acoustid(&self, key: &str, id: AcoustId) -> Result<()> {
        let key = format!("{key}-acoustid");
        let _old = self.tree.insert(key, id.0.as_str())?;
        Ok(())
    }
}
