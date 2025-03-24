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

    for file in mp3_files(path).iter() {
        eprintln!("File `{}`", file.display());
        let song = match Song::new(file) {
            Ok(it) => it,
            Err(err) => {
                eprintln!("ERR file: {} {err}", file.display());
                continue;
            }
        };
        let song_data = match acoustid::lookup_by_fingerprint(song) {
            Ok(it) => it,
            Err(err) => {
                eprintln!("ERR fingerprint: {err} for `{}`", file.display());
                continue;
            }
        };
        match acoustid::write_song_data(file, &song_data) {
            Ok(_) => {
                match rename_file_as_artist_dash_title(file) {
                    Ok(newfile) => {
                        println!("Renamed `{}` as `{}`", file.display(), newfile.display());
                    }

                    Err(_) => todo!(),
                };
            }
            Err(e) => {
                eprintln!("{}: {}", file.display(), e);
            }
        };
    }

    Ok(())
}
