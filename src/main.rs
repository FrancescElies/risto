use rodio::{Decoder, OutputStream, Sink};
use std::io::{self, Write};
use std::sync::mpsc::channel;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
enum Like {
    Yes,
    No,
    Repeat,
}

fn did_you_like_it() -> Like {
    print!("like it? yes(y), no(n), repeat(r): ");
    io::stdout().flush().unwrap(); // Ensure the prompt is displayed
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    match input.trim() {
        "y" | "Y" => Like::Yes,
        "r" | "R" => Like::Repeat,
        _ => Like::No,
    }
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
                    sink.stop();
                    // println!("error rx {_e:#?}");
                }
            }
        }

        sink.stop();
    });

    let th_ipnut_reader = thread::spawn(move || {
        let like = did_you_like_it();
        tx_like.send(like).unwrap();
        tx_stop_song.send(true).unwrap();
    });

    let like = match rx_like.recv() {
        Ok(x) => x,
        Err(_) => Like::Repeat,
    };

    th_ipnut_reader.join().expect("player thread panicked!");
    th_player.join().expect("player thread panicked!");

    like
}

fn main() {
    for file in mp3_files("/Users/cesc/Music") {
        println!("playing {file:?}");
        loop {
            match play(&file) {
                Like::Yes => {
                    break;
                }
                Like::No => {
                    break;
                }
                Like::Repeat => {}
            }
        }
    }
}
