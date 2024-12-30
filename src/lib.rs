use twox_hash::XxHash64;
use walkdir::WalkDir;

use std::{
    fs::{self, File},
    hash::{BuildHasher, BuildHasherDefault},
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::{Context, Ok, Result};
use chromaprint_native;
use rodio::{Decoder, Source};

#[derive(Debug, Default)]
pub struct Song {
    path: PathBuf,
    acoustid: Option<String>,
}

impl Song {
    pub fn new(path: &Path) -> Result<Self> {
        Ok(Song {
            path: path.canonicalize()?.to_path_buf(),
            acoustid: None,
        })
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

    fn hash(&self) -> Result<u64> {
        let data = fs::read(&self.path)?;
        let hasher: BuildHasherDefault<XxHash64> = Default::default();
        Ok(hasher.hash_one(data))
    }

    pub fn get_acoustid(&mut self) -> Result<String> {
        println!("{} hash={:?}", self.path.display(), self.hash());

        // E.g. if sample_rate is  44100 and has 2 audio channels. It is expected that samples
        // are interleaved: in this case left channel samples are placed at even indices
        // and right channel - at odd ones.
        let (sample_rate, num_channels, samples) = self.get_raw_samples()?;
        let mut ctx = chromaprint_native::Context::new();
        ctx.start(sample_rate.try_into()?, num_channels.try_into()?)?;
        ctx.feed(&samples)?;
        ctx.finish()?;

        let acoustid = ctx.fingerprint()?;
        self.acoustid = Some(acoustid.clone());
        Ok(acoustid)
    }
}

pub fn mp3_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|x| match x {
            std::result::Result::Ok(f) => {
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

// #[test]
// fn mp3_acoustid() {
//     let path = Path::new("/home/cesc/Music/07 Toots & The Maytals - Funky Kingston.mp3");
//     let mut song = Song::new(path);
//     song.get_acoustid().unwrap();
//
//     let acoustid = song.acoustid.unwrap();
//     assert_eq!(acoustid[0], 0x_016db1f6_u32);
//     assert_eq!(acoustid[1], 0x_006db1be_u32);
// }
