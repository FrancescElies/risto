use anyhow::{Context, Result};
use risto::{
    acoustid::{self, rename_file_as_artist_dash_title},
    mp3_files, Song,
};
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

    let (newfiles, errors): (Vec<_>, Vec<_>) = mp3_files(path)
        .iter()
        .map(|song| lookup_write_id3_and_rename_file(song))
        // .collect::<Vec<Result<_>>>();
        .partition(Result::is_ok);

    let newfiles: Vec<_> = newfiles.into_iter().map(Result::unwrap).collect();
    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();

    eprintln!("\n# Ok:");
    for newfile in newfiles {
        eprintln!("- {}", newfile.display());
    }
    eprintln!("\n# Errors:");
    for err in errors {
        eprintln!("- {err:?}");
    }

    Ok(())
}

fn lookup_write_id3_and_rename_file(file: &PathBuf) -> Result<PathBuf> {
    eprintln!("\n# File `{}`", file.display());
    let song = Song::new(file).with_context(|| ("❌ open song failed"))?;
    let song_data = acoustid::lookup_by_fingerprint(song)
        .with_context(|| "❌ fingerprint lookup failed: {err}")?;
    let _ = acoustid::write_song_data(file, &song_data).with_context(|| "❌ write id3 failed")?;
    let newfile =
        rename_file_as_artist_dash_title(file).with_context(|| "❌ rename file failed")?;
    Ok(newfile)
}
