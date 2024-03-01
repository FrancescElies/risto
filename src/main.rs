use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    sync::mpsc::channel,
    thread,
    time::Duration,
};
use walkdir::WalkDir;

use clap::Parser;

/// Classify music stop at any time and continue later on.
///
/// Saves results to likes.json, later on you can process the json e.g. remove files you didn't like.
#[derive(Parser)]
struct Cli {
    /// Path to your music folder
    music_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Songs(Vec<Song>);

#[derive(Debug, Serialize, Deserialize)]
struct Song {
    path: String,
    like: Like,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Like {
    Yes,
    No,
    DontKnow,
}

fn did_you_like_it() -> Result<Like> {
    print!("٭ Like it? yes(y)/love(l), no(n)/hate(h), repeat(r): ");
    io::stdout()
        .flush()
        .with_context(|| "couldn't flush stdout")?; // Ensure the prompt is displayed
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    Ok(match input.trim() {
        "y" | "Y" | "l" | "L" => Like::Yes,
        "n" | "N" => Like::No,
        _ => Like::DontKnow,
    })
}

fn mp3_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|x| match x {
            Ok(f) => {
                if f.file_type().is_file() {
                    Some(f.path().to_owned())
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect()
}

fn play(path: &Path) -> Like {
    // Create an output stream
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let (tx_like, rx_like) = channel();
    let (tx_stop_song, rx_stop_song) = channel();

    let file = File::open(path).unwrap();
    let th_player = thread::spawn(move || {
        // Play the MP3 file
        let source = Decoder::new(BufReader::new(file)).unwrap();
        // let source = source.take_duration(Duration::from_secs(5));
        sink.append(source);

        // while song playing
        while sink.len() != 0 {
            match rx_stop_song.recv_timeout(Duration::from_secs(5)) {
                Ok(_) => {
                    // on user input
                    sink.stop();
                    return;
                }
                Err(_e) => {
                    // println!("error rx {_e:#?}");
                }
            }
        }

        sink.stop();
    });

    let th_ipnut_reader = thread::spawn(move || {
        let like = did_you_like_it().unwrap_or(Like::DontKnow);
        // don't care if send fails
        let _ = tx_like.send(like);
        let _ = tx_stop_song.send(true);
    });

    let like = match rx_like.recv() {
        Ok(x) => x,
        Err(_) => Like::DontKnow,
    };

    th_ipnut_reader.join().expect("player thread panicked!");
    th_player.join().expect("player thread panicked!");

    like
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let music_dr = args.music_dir;

    let likes_path = Path::new("likes.json");
    let mut songs = load_likes(likes_path)?;
    let already_listened_longs: HashSet<String> = songs.iter().map(|x| x.path.clone()).collect();
    for file in mp3_files(music_dr).iter().filter(|x| {
        let keep = !already_listened_longs.contains(x.to_str().unwrap_or(""));
        if !keep {
            println!("👣 skipping {x:?}");
        }
        keep
    }) {
        println!("▶️ playing {file:?}");
        let mut like;
        loop {
            like = play(file);
            match like {
                Like::Yes => {
                    println!("❤️ {file:?}");
                    break;
                }
                Like::No => {
                    println!("🚮 {file:?}");
                    break;
                }
                Like::DontKnow => {
                    println!("⥁ {file:?}");
                    // no break, will keep repeating the song
                }
            }
        }
        songs.push(Song {
            path: file
                .to_str()
                .unwrap_or_else(|| panic!("couldn't convert {file:?} to string"))
                .to_owned(),
            like,
        });
        serde_json::to_writer(
            File::create(likes_path).unwrap_or_else(|_| panic!("couldn't open {likes_path:?}")),
            &songs,
        )?;
    }
    Ok(())
}

fn load_likes(path: &Path) -> Result<Vec<Song>> {
    if !path.exists() {
        let mut f = File::create(path).with_context(|| format!("failed to create {path:?}"))?;
        let _ = f.write(b"[]");
    };
    let file = File::open(path).with_context(|| format!("couldn't load {path:?}"))?;
    let reader = BufReader::new(file);
    let files = serde_json::from_reader(reader)?;
    Ok(files)
}
