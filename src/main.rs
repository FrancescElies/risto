use rodio::{Decoder, OutputStream, Sink, Source};
use std::io::{self, Write};
use std::sync::mpsc::channel;
// use std::time::Duration;

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

#[derive(Debug)]
enum Like {
    Yes,
    No,
    Repeat,
}

fn did_you_like_it() -> Like {
    print!("like it? (y/n): ");
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

fn play(path: PathBuf) {
    // Create an output stream
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let (tx_like, rx_input) = channel();
    let file = File::open(path.clone()).unwrap();
    let th_player = thread::spawn(move || {
        // Play the MP3 file
        let source = Decoder::new(BufReader::new(file)).unwrap();
        // let source = source.take_duration(Duration::from_secs(5));
        sink.append(source);

        loop {
            match rx_input.recv_timeout(Duration::from_secs(5)) {
                Ok(liked) => {
                    match liked {
                        Like::Yes => {
                            println!("liked")
                        }
                        Like::No => {
                            println!("didn't like")
                        }
                        Like::Repeat => {
                            println!("recived repeat");
                            sink.stop();
                            play(path);
                            return;
                        }
                    }
                    break;
                }
                Err(_e) => {
                    // println!("error rx {_e:#?}");
                }
            }
        }

        sink.stop();
    });

    let th_ipnut_reader = thread::spawn(move || match did_you_like_it() {
        x @ Like::Yes => {
            println!("send {x:?}");
            tx_like.send(x).unwrap();
        }
        x @ Like::No => {
            println!("send {x:?}");
            tx_like.send(x).unwrap();
        }
        x @ Like::Repeat => {
            println!("send {x:?}");
            tx_like.send(x).unwrap();
        }
    });

    th_ipnut_reader.join().expect("player thread panicked!");
    th_player.join().expect("player thread panicked!");
}

fn main() {
    for file in mp3_files("/Users/cesc/Music") {
        play(file);
    }
}
