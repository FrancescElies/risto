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
        let mut song = Song::new(file)?;
        let acoustid = song
            .get_acoustid()
            .with_context(|| format!("{}", path.display()))?;
        eprintln!("acoustid: {}...", &acoustid.to_string()[0..15],);

        // API docs https://acoustid.org/webservice
        let url = "https://api.acoustid.org/v2/lookup";

        let client = reqwest::blocking::Client::new();
        let duration = song
            .get_duration()
            .unwrap_or_default()
            .as_secs()
            .to_string();
        let fingerprint = acoustid.to_string();
        let map = HashMap::from([
            ("format", "json"),        // response format
            ("client", "ks84xymUAAY"), // API key
            ("duration", &duration),   // song duration
            ("fingerprint", &fingerprint),
        ]);

        eprintln!("REQUEST {url} {map:#?}");
        let req = client.post(url).json(&map); // 414 URI Too Long
        eprintln!("Request::\n {:#?}", req);
        let res = req.send()?;

        eprintln!("Response:\n {:#?}", res);
        eprintln!("Bytes:\n {:#?}", res.bytes());
    }

    Ok(())
}
