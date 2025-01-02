use anyhow::{Context, Result};
use risto::{acoustid, mp3_files, Song};
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
        eprintln!("File `{}`", file.display());
        let song = acoustid::lookup_by_fingerprint(Song::new(file)?)?;

        eprintln!(
            "Most likely artist `{}` and tilte `{}`",
            song.artist, song.title
        );
    }

    Ok(())
}
