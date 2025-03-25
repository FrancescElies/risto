use anyhow::{Context, Result};
use risto::{
    acoustid::{lookup_by_fingerprint, rename_file_as_artist_dash_title, write_song_data},
    Song,
};
use std::path::PathBuf;

pub fn lookup_write_id3_and_rename_file(file: &PathBuf) -> Result<PathBuf> {
    eprintln!("\n# File `{}`", file.display());
    let song = Song::new(file).with_context(|| ("❌ open song failed"))?;
    let song_data =
        lookup_by_fingerprint(song).with_context(|| "❌ fingerprint lookup failed: {err}")?;
    write_song_data(file, &song_data).with_context(|| "❌ write id3 failed")?;
    let newfile =
        rename_file_as_artist_dash_title(file).with_context(|| "❌ rename file failed")?;
    Ok(newfile)
}
