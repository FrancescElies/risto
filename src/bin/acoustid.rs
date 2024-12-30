use anyhow::{Context, Result};
use risto::{mp3_files, Song};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clap::Parser;

/// Classify music stop at any time and continue later on.
///
/// Saves results to likes.json, later on you can process the json e.g. remove files you didn't like.
#[derive(Parser)]
struct Cli {
    /// file or folder to calculate acoustid
    path: PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let path = args.path;
    let path_str = path.to_string_lossy();

    let path = shellexpand::full(&path_str)
        .with_context(|| format!("couldn't expand {}", &path.display()))?;
    let path = Path::new(path.as_ref());

    for file in mp3_files(path).iter() {
        let acoustid = Song::new(file)?
            .get_acoustid()
            .with_context(|| format!("{}", path.display()))?;
        eprintln!("{}... acoustid for {}", &acoustid[0..15], file.display());

        // API docs https://acoustid.org/webservice
        let url = "https://api.acoustid.org/v2/lookup";

        let client = reqwest::blocking::Client::new();
        let mut map = HashMap::new();
        map.insert("format", "json"); // response format
        map.insert("client", "ks84xymUAAY"); // application's API key
                                             //map.insert("meta", "recordings");
                                             //map.insert("trackid", "9ff43b6a-4f16-427c-93c2-92307ca505e0");
        map.insert("duration", "641"); // duration of the whole audio file in seconds
        map.insert("fingerprint", &acoustid); // audio fingerprint

        eprintln!("REQUEST {url}");
        let res = client
            .get(url)
            .query(&map) // 414 URI Too Long
            // TODO: maybe register app in https://acoustid.org/webservice instead of using default
            // client API key?
            .send()?;

        eprintln!("Response {}: {:#?}", res.status(), res.bytes());
    }

    Ok(())
}
