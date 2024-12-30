use anyhow::{Context, Result};
use risto::{mp3_files, Song};
use std::path::{Path, PathBuf};

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
        eprintln!("{} for {}", &acoustid, file.display());
        let url = format!(
            "https://api.acoustid.org/v2/lookup?client=ks84xymUAAY&duration=641&fingerprint={}",
            acoustid
        );
        eprintln!("Fetching {url:?}...");
        let res = reqwest::blocking::get(url)?;
        eprintln!("Response: {:#?}", res);
    }

    Ok(())
}
