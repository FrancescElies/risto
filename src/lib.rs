pub mod acoustid;
mod cache;

use cache::Db;
use clap::builder::OsStr;
use twox_hash::XxHash64;
use walkdir::WalkDir;

use std::{
    fmt::Display,
    fs::{self, File},
    hash::{BuildHasher, BuildHasherDefault},
    io::BufReader,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use rodio::{Decoder, Source};

#[derive(Debug, Clone)]
pub struct AcoustId(String);

#[derive(Debug, Clone)]
pub struct FileHash(String);
impl Display for FileHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl AsRef<[u8]> for FileHash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Display for AcoustId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct Song {
    pub path: PathBuf,
    acoustid: Option<AcoustId>,
    cache_acoustid: Db,
}

impl Display for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Song-name {}",
            self.path
                .file_name()
                .unwrap_or(&OsStr::from("<unkown-file>"))
                .to_string_lossy()
        )
    }
}

impl Song {
    pub fn new(path: &Path) -> Result<Self> {
        Ok(Song {
            path: path.canonicalize()?.to_path_buf(),
            acoustid: None,
            cache_acoustid: Db::new(),
        })
    }

    pub fn get_duration(&self) -> Result<Duration> {
        let hash = self.hash()?;
        if let Some(x) = self.cache_acoustid.get_duration(&hash) {
            return Ok(x);
        }
        // HACK: decoding samples to get song length https://github.com/RustAudio/rodio/issues/190
        let (sample_rate, channels, samples) = self.get_raw_samples()?;
        let seconds: u64 = samples.len() as u64 / channels as u64 / sample_rate as u64;
        let res = Duration::from_secs(seconds);
        self.cache_acoustid.insert_duration(&hash, res);
        Ok(res)
    }

    pub fn get_raw_samples(&self) -> Result<(u32, u32, Vec<i16>)> {
        let file = BufReader::new(
            File::open(&self.path)
                .with_context(|| format!("opening song {:?}", self.path.clone()))?,
        );
        let decoder = Decoder::new(BufReader::new(file))
            .with_context(|| format!("couldn't decode {}", self.path.display()))?;

        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels().into();

        Ok((sample_rate, channels, decoder.collect()))
    }

    fn hash(&self) -> Result<FileHash> {
        let data = fs::read(&self.path)?;
        let hasher: BuildHasherDefault<XxHash64> = Default::default();
        Ok(FileHash(hasher.hash_one(data).to_string()))
    }

    pub fn calc_acoustid(&mut self) -> Result<AcoustId> {
        eprintln!("Calculating acoustid for {}", self.path.display(),);

        // E.g. if sample_rate is  44100 and has 2 audio channels. It is expected that samples
        // are interleaved: in this case left channel samples are placed at even indices
        // and right channel - at odd ones.
        let (sample_rate, num_channels, samples) = self.get_raw_samples()?;
        let mut ctx = chromaprint_native::Context::new();
        ctx.start(sample_rate.try_into()?, num_channels.try_into()?)?;
        ctx.feed(&samples)?;
        ctx.finish()?;

        let acoustid = AcoustId(ctx.fingerprint()?);
        Ok(acoustid)
    }

    pub fn get_acoustid(&mut self) -> Result<AcoustId> {
        let hash = self.hash()?;
        let acoustid = self.cache_acoustid.get_acoustid(&hash);

        match acoustid {
            Some(acoustid) => {
                self.acoustid = Some(acoustid.clone());
                Ok(acoustid)
            }
            None => {
                let acoustid = self.calc_acoustid()?;
                let _dont_care = self.cache_acoustid.insert_acoustid(&hash, acoustid.clone());
                Ok(acoustid)
            }
        }
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
