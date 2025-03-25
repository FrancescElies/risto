use std::io;
use std::path::PathBuf;

pub mod classify_music;
pub mod rename_music_files;

pub fn read_files_from_stdin() -> Vec<PathBuf> {
    let mut lines = vec![];
    for line in io::stdin().lines().map_while(Result::ok) {
        lines.push(PathBuf::from(line));
    }
    lines
}
