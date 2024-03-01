use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    sync::mpsc::channel,
    thread,
    time::Duration,
};
use walkdir::WalkDir;

static MUSIC_PATH: &str = "/home/cesc/Music";

#[derive(Serialize, Deserialize)]
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
                    println!("user said something");
                    sink.stop();
                    return;
                }
                Err(_e) => {
                    // println!("error rx {_e:#?}");
                }
            }
        }

        println!("song end");
        sink.stop();
    });

    let th_ipnut_reader = thread::spawn(move || {
        let like = did_you_like_it().unwrap_or(Like::DontKnow);
        tx_like.send(like).unwrap();
        tx_stop_song.send(true).unwrap();
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
    // load_likes(Path::new("likes.txt"))?;
    for file in mp3_files(MUSIC_PATH) {
        println!("playing {file:?}");
        loop {
            match play(&file) {
                Like::Yes => {
                    println!("❤️ {file:?}");
                    break;
                }
                Like::No => {
                    println!("👻 {file:?}");
                    break;
                }
                Like::DontKnow => {
                    println!("⥁ {file:?}");
                }
            }
        }
    }
    Ok(())
}

fn load_likes(path: &Path) -> Result<Vec<Song>> {
    let file = match File::open(path) {
        Ok(x) => x,
        Err(_) => File::create(path).with_context(|| format!("failed to create {path:?}"))?,
    };
    let reader = BufReader::new(file);
    let files: Vec<Song> = serde_json::from_reader(reader)?;
    Ok(files)
}
