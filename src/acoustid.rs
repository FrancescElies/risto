use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::{Context, Ok, Result};
use rodio::{Decoder, Source};
use rusty_chromaprint::{Configuration, Fingerprinter};

type AcoustId = Vec<u32>;

#[derive(Debug, Default)]
pub struct Song {
    path: PathBuf,
    acoustid: Option<AcoustId>,
}

impl Song {
    pub fn new(path: &Path) -> Self {
        Song {
            path: path.to_path_buf(),
            acoustid: None,
        }
    }

    pub fn get_raw_samples(&self) -> Result<(u32, u32, Vec<i16>)> {
        let file = BufReader::new(
            File::open(&self.path)
                .with_context(|| format!("opening song {:?}", self.path.clone()))?,
        );
        let decoder = Decoder::new(BufReader::new(file))
            .with_context(|| format!("couldn't open song {self:?}"))?;

        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels().into();

        Ok((sample_rate, channels, decoder.collect()))
    }

    pub fn get_acoustid(&mut self) -> Result<AcoustId> {
        let mut printer = Fingerprinter::new(&Configuration::preset_test2());
        // E.g. if sample_rate is  44100 and has 2 audio channels. It is expected that samples
        // are interleaved: in this case left channel samples are placed at even indices
        // and right channel - at odd ones.
        let (sample_rate, channels, samples) = self.get_raw_samples()?;
        printer.start(sample_rate, channels).with_context(|| {
            format!(
                "couldn't initialize fingerprinter sample_rate={} channel={}",
                sample_rate, channels
            )
        })?;
        // printer.consume(&[-100, -100, -50, -50, 1000, 1000]);
        printer.consume(&samples);
        printer.finish();
        let fingerprint = printer.fingerprint();

        self.acoustid = Some(fingerprint.iter().map(|x| *x).collect());
        Ok(self.acoustid.clone().unwrap())
    }
}

#[test]
fn mp3_acoustid() {
    let path = Path::new("/home/cesc/Music/07 Toots & The Maytals - Funky Kingston.mp3");
    let mut song = Song::new(path);
    song.get_acoustid().unwrap();

    let acoustid = song.acoustid.unwrap();
    assert_eq!(acoustid[0], 0x_016db1f6_u32);
    assert_eq!(acoustid[1], 0x_006db1be_u32);
}
