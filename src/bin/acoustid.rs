use anyhow::{Context, Result};
use risto::Song;
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
    let path = path.as_ref();
    let path = Path::new(path);
    println!("path {}", path.display());
    let path = path.canonicalize()?;

    if path.is_dir() {
        todo!();
    } else if path.is_file() {
        let mut song = Song::new(&path);
        let acoustid = song.get_acoustid().unwrap();
        println!("{acoustid}");
    } else {
        todo!();
    }

    Ok(())
}
