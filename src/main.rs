//! Risto helps you clean up your music by asking if you like
//! one song at a time while playing it in the background.

mod cli;
use anyhow::{Context, Result};
use cli::{classify_music, read_files_from_stdin, rename_music_files};
use risto::mp3_files;
use std::path::{Path, PathBuf};
use termimad::{
    crossterm::style::{Attribute::Underlined, Color::DarkYellow},
    minimad::TextTemplate,
    MadSkin,
};

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "risto")]
#[command(
    about = "Remove music one file at a time or rename files",
    long_about = ""
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
    /// Rename music files with lookup acoustid id3
    RenameFiles {
        /// Path to folder with music
        #[arg(value_name = "PATH")]
        /// file or folder to calculate acoustid or instead a list of files via STDIN
        path: Option<PathBuf>,
    },
}

fn shellexpand_or_read_files_from_stdin(path: Option<&Path>) -> Result<Vec<PathBuf>> {
    match path {
        Some(path) => {
            let path_str = path.to_string_lossy();

            let path = shellexpand::full(&path_str)
                .with_context(|| format!("couldn't expand {}", path.display()))?;
            let path = Path::new(path.as_ref());

            Ok(mp3_files(path))
        }
        None => Ok(read_files_from_stdin()),
    }
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
        Commands::RenameFiles { path } => {
            let files = shellexpand_or_read_files_from_stdin(path.as_deref())?;
            let (newfiles, errors) = rename_music_files::as_title_artist(&files)?;

            eprintln!("\n# Ok:");
            for newfile in newfiles {
                eprintln!("- {}", newfile.display());
            }
            eprintln!("\n# Errors:");
            for err in errors {
                eprintln!("- {err:?}");
            }
        }
    };

    Ok(())
}
