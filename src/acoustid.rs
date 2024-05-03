use std::{fs::File, io::BufReader, path::Path};

use anyhow::{Context, Result};
use rodio::{Decoder, Source};
use rusty_chromaprint::{Configuration, Fingerprinter};

#[derive(Debug)]
pub struct Song {
    samples: Vec<i16>,
    sample_rate: u32,
    channels: u32,
}

fn get_raw_samples(song: &Path) -> Result<Song> {
    let file = BufReader::new(File::open(song).with_context(|| format!("opening song {song:?}"))?);
    let decoder = Decoder::new(BufReader::new(file))
        .with_context(|| format!("couldn't open song {song:?}"))?;

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels().into();
    let samples = decoder.collect(); // TODO: it hangs
    Ok(Song {
        samples,
        sample_rate,
        channels,
    })
}

pub fn compute_acoustid(song: &Path) -> Result<(Song, Vec<u32>)> {
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

    Ok((song, fingerprint.iter().map(|x| *x).collect()))
}

#[test]
fn mp3_acoustid() {
    let path = Path::new("/home/cesc/Music/07 Toots & The Maytals - Funky Kingston.mp3");
    let (song, acoustid) = compute_acoustid(&path).unwrap();
    assert_eq!(song.sample_rate, 44_100);
    assert_eq!(song.channels, 2);
    assert_eq!(acoustid[0], 0x_016db1f6_u32);
    assert_eq!(acoustid[1], 0x_006db1be_u32);
}
