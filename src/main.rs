//! Risto helps you clean up your music by asking if you like
//! one song at a time while playing it in the background.

mod cli;
use anyhow::Result;
use cli::classify_music;
use std::path::PathBuf;
use termimad::{
    crossterm::style::{Attribute::Underlined, Color::DarkYellow},
    minimad::TextTemplate,
    MadSkin,
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
            classify_music::keep_asking(&skin, music_dir.as_ref())?;
        }
    };

    Ok(())
}
