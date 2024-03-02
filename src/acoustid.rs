use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use chromaprint::Chromaprint;
use rodio::Decoder;

fn get_raw_samples(song: &Path) -> Result<Vec<i16>> {
    let file = File::open(song).with_context(|| format!("couldn't ope son {song:?}"))?;
    let decoder_samples =
        Decoder::new(BufReader::new(file)).with_context(|| format!("couldn't ope son {song:?}"))?;

    let samples = decoder_samples.into_iter().collect();
    println!("{samples:?}");
    Ok(samples)
}

pub fn sim_hash(song: &Path) -> Option<String> {
    let data = get_raw_samples(song).ok()?;
    let mut c = Chromaprint::new();
    c.feed(&data);
    let id = c.fingerprint();
    println!("acoustid {id:?}");
    id
}
