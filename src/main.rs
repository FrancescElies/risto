//! Risto helps you clean up your music by asking if you like
//! one song at a time while playing it in the background.

use anyhow::{Context, Result};
use crossterm::style::Color::DarkYellow;
use risto::mp3_files;
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    sync::mpsc::channel,
    thread,
    time::Duration,
};
use termimad::*;
use termimad::{
    crossterm::style::Attribute::Underlined, mad_print_inline, minimad::TextTemplate, MadSkin,
};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "risto")]
#[command(
    about = "Classify music stop at any time and continue later on.",
    long_about = "Saves results to likes.json, later on you can process the json e.g. remove files you didn't like."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
#[command(version, about, long_about = None)]
enum Commands {
    /// Classify music while listening to it
    #[command(arg_required_else_help = true)]
    Listen {
        /// Path to folder with music
        #[arg(value_name = "PATH")]
        music_dir: Option<PathBuf>,
    },
    /// Removes files marked as not liked
    RemoveDisliked {},
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
    ExtensionNotSupported,
}

fn did_you_like_it(skin: &MadSkin) -> Like {
    ask!(skin, "Do you like it?", ('y') {
        ('y', "**y**es") => { Like::Yes }
        ('n', "**n**o, please   move to trash") => { Like::No }
        ('r', "**r**epeat") => { Like::DontKnow }
    })
}

fn play(skin: &MadSkin, path: &Path) -> Result<Like> {
    // Create an output stream
    let (_stream, stream_handle) =
        OutputStream::try_default().with_context(|| "output stream".to_owned())?;
    let sink = Sink::try_new(&stream_handle).with_context(|| "creating sink".to_owned())?;

    let (tx_like, rx_like) = channel();
    let (tx_stop_song, rx_stop_song) = channel();

    let supported_extensions = ["mp3", "flac", "ogg", "wav", "mp4", "acc"];
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("unkown");
    if !supported_extensions.contains(&ext) {
        return Ok(Like::ExtensionNotSupported);
    }

    let file = File::open(path).with_context(|| format!("couldn't open file {path:?}"))?;
    // let path = path.to_owned();
    let th_player = thread::spawn(move || {
        // Play the MP3 file
        let decoder = match Decoder::new(BufReader::new(file)) {
            Ok(x) => x,
            Err(_) => return,
        };
        // let id = acoustid::sim_hash(&path);
        // let source = source.take_duration(Duration::from_secs(5));
        sink.append(decoder);

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

    let skin = skin.clone();
    let th_ipnut_reader = thread::spawn(move || {
        let like = did_you_like_it(&skin);
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

    Ok(like)
}

fn main() -> Result<()> {
    let mut skin = MadSkin::default();
    skin.bold.set_fg(DarkYellow);
    skin.italic.add_attr(Underlined);
    let text_template = TextTemplate::from("# ${app-name} v${app-version}");
    let mut expander = text_template.expander();
    expander
        .set("app-name", env!("CARGO_PKG_NAME"))
        .set("app-version", env!("CARGO_PKG_VERSION"));
    skin.print_expander(expander);

    let args = Cli::parse();

    match args.command {
        Commands::Listen { music_dir } => {
            classify_music(&skin, music_dir.as_ref())?;
        }
        Commands::RemoveDisliked {} => todo!(),
    };

    Ok(())
}

fn classify_music(skin: &MadSkin, music_dir: Option<&PathBuf>) -> Result<()> {
    let pwd = Path::new(".").to_path_buf();
    let music_dir: &PathBuf = music_dir.unwrap_or(&pwd);
    let likes_path = Path::new("likes.json");
    let mut songs = load_likes(likes_path)?;
    let already_listened_longs: HashSet<String> = songs.iter().map(|x| x.path.clone()).collect();
    for file in mp3_files(music_dir).iter().filter(|x| {
        let keep = !already_listened_longs.contains(x.to_str().unwrap_or(""));
        if !keep {
            mad_print_inline!(skin, "*skipped* $0\n", x.display());
        }
        keep
    }) {
        mad_print_inline!(skin, "**playing** $0\n", file.display());
        let mut like;
        loop {
            like = play(skin, file)?;
            match like {
                Like::Yes => {
                    mad_print_inline!(skin, "*liked*  $0\n", file.display());
                    break;
                }
                Like::No => {
                    mad_print_inline!(skin, "*trash*  $0\n", file.display());
                    trash::delete(file)?;
                    break;
                }
                Like::DontKnow => {
                    mad_print_inline!(skin, "*not sure*  $0\n", file.display());
                    // no break, will keep repeating the song
                }
                Like::ExtensionNotSupported => {
                    mad_print_inline!(skin, "$0 *not supported*, skipped\n", file.display());
                    break;
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
        serde_json::to_writer_pretty(
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
