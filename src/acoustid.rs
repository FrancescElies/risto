use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use rodio::{Decoder, Source};
use rusty_chromaprint::{Configuration, Fingerprinter};

#[derive(Debug)]
struct Audio {
    samples: Vec<i16>,
    sample_rate: u32,
    channels: u32,
}

fn get_raw_samples(song: &Path) -> Result<Audio> {
    let file = BufReader::new(File::open(song).with_context(|| format!("opening song {song:?}"))?);
    let decoder = Decoder::new(BufReader::new(file))
        .with_context(|| format!("couldn't open song {song:?}"))?;

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels().into();
    let samples = decoder.collect(); // TODO: it hangs
    Ok(Audio {
        samples,
        sample_rate,
        channels,
    })
}

pub fn sim_hash(song: &Path) -> Result<String> {
    let song = get_raw_samples(song)?;
    let mut printer = Fingerprinter::new(&Configuration::preset_test2());
    // Sampling rate is set to 44100 and stream has 2 audio channels. It is expected that samples
    // are interleaved: in this case left channel samples are placed at even indices
    // and right channel - at odd ones.
    printer
        .start(song.sample_rate, song.channels)
        .with_context(|| {
            format!(
                "couldn't initialize fingerprinter sample_rate={} channel={}",
                song.sample_rate, song.channels
            )
        })?;
    // printer.consume(&[-100, -100, -50, -50, 1000, 1000]);
    printer.consume(&song.samples);
    printer.finish();
    let fingerprint = printer.fingerprint();

    Ok(format!("{:08x?}", fingerprint))
}

#[test]
fn mp3_acoustid() {
    let path = Path::new("/home/cesc/Music/07 Toots & The Maytals - Funky Kingston.mp3");
    let acoustid = sim_hash(&path).unwrap();
    assert!(acoustid.starts_with("[016db1f6, 006db1be"));
}
