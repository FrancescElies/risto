use crate::Song;
use anyhow::{anyhow, Context, Result};
use id3::{Tag, TagLike, Version};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Serialize, Deserialize)]
struct Artist {
    name: Option<String>,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Recordings {
    duration: Option<u64>,
    id: String,
    title: Option<String>,
    artists: Option<Vec<Artist>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    results: Vec<SongMatch>,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TrackId(String);

#[derive(Debug, Serialize, Deserialize)]
struct SongMatch {
    id: TrackId,
    score: f64,
    recordings: Vec<Recordings>,
}

pub struct SongData {
    pub title: String,
    pub artist: String,
}

pub fn write_song_data(songfile: impl AsRef<Path>, new: &SongData) -> Result<()> {
    let mut tag = Tag::read_from_path(&songfile).unwrap_or(Tag::new());

    tag.set_artist(new.artist.clone());
    tag.set_title(new.title.clone());

    tag.write_to_path(&songfile, Version::Id3v24)
        .with_context(|| format!("failed to write id3 tag"))
}

pub fn lookup_by_fingerprint(mut song: Song) -> Result<SongData> {
    // API key https://acoustid.org/webservice#lookup
    let api_key = std::env::var("ACOUSTID_API_KEY").with_context(|| {
        "reading env var ACOUSTID_API_KEY, register app at https://acoustid.org/my-applications or use the same client as in examples at https://acoustid.org/webservice"
    })?;

    let acoustid = song
        .get_acoustid()
        .with_context(|| format!("{}", song.path.display()))?;
    let display_short_acoustid = &acoustid.to_string()[0..15];
    let url = "https://api.acoustid.org/v2/lookup";
    let client = reqwest::blocking::Client::new();
    let duration = song.get_duration()?.as_secs().to_string();
    let fingerprint = acoustid.to_string();
    let map = HashMap::from([
        // Test https://acoustid.org/webservice#lookup
        //("fingerprint", "AQABz0qUkZK4oOfhL-CPc4e5C_wW2H2QH9uDL4cvoT8UNQ-eHtsE8cceeFJx-LiiHT-aPzhxoc-Opj_eI5d2hOFyMJRzfDk-QSsu7fBxqZDMHcfxPfDIoPWxv9C1o3yg44d_3Df2GJaUQeeR-cb2HfaPNsdxHj2PJnpwPMN3aPcEMzd-_MeB_Ej4D_CLP8ghHjkJv_jh_UDuQ8xnILwunPg6hF2R8HgzvLhxHVYP_ziJX0eKPnIE1UePMByDJyg7wz_6yELsB8n4oDmDa0Gv40hf6D3CE3_wH6HFaxCPUD9-hNeF5MfWEP3SCGym4-SxnXiGs0mRjEXD6fgl4LmKWrSChzzC33ge9PB3otyJMk-IVC6R8MTNwD9qKQ_CC8kPv4THzEGZS8GPI3x0iGVUxC1hRSizC5VzoamYDi-uR7iKPhGSI82PkiWeB_eHijvsaIWfBCWH5AjjCfVxZ1TQ3CvCTclGnEMfHbnZFA8pjD6KXwd__Cn-Y8e_I9cq6CR-4S9KLXqQcsxxoWh3eMxiHI6TIzyPv0M43YHz4yte-Cv-4D16Hv9F9C9SPUdyGtZRHV-OHEeeGD--BKcjVLOK_NCDXMfx44dzHEiOZ0Z44Rf6DH5R3uiPj4d_PKolJNyRJzyu4_CTD2WOvzjKH9GPb4cUP1Av9EuQd8fGCFee4JlRHi18xQh96NLxkCgfWFKOH6WGeoe4I3za4c5hTscTPEZTES1x8kE-9MQPjT8a8gh5fPgQZtqCFj9MDvp6fDx6NCd07bjx7MLR9AhtnFnQ70GjOcV0opmm4zpY3SOa7HiwdTtyHa6NC4e-HN-OfC5-OP_gLe2QDxfUCz_0w9l65HiPAz9-IaGOUA7-4MZ5CWFOlIfe4yUa6AiZGxf6w0fFxsjTOdC6Itbh4mGD63iPH9-RFy909XAMj7mC5_BvlDyO6kGTZKJxHUd4NDwuZUffw_5RMsde5CWkJAgXnDReNEaP6DTOQ65yaD88HoeX8fge-DSeHo9Qa8cTHc80I-_RoHxx_UHeBxrJw62Q34Kd7MEfpCcu6BLeB1ePw6OO4sOF_sHhmB504WWDZiEu8sKPpkcfCT9xfej0o0lr4T5yNJeOvjmu40w-TDmqHXmYgfFhFy_M7tD1o0cO_B2ms2j-ACEEQgQgAIwzTgAGmBIKIImNQAABwgQATAlhDGCCEIGIIM4BaBgwQBogEBIOESEIA8ARI5xAhxEFmAGAMCKAURKQQpQzRAAkCCBQEAKkQYIYIQQxCixCDADCABMAE0gpJIgyxhEDiCKCCIGAEIgJIQByAhFgGACCACMRQEyBAoxQiHiCBCFOECQFAIgAABR2QAgFjCDMA0AUMIoAIMChQghChASGEGeYEAIAIhgBSErnJPPEGWYAMgw05AhiiGHiBBBGGSCQcQgwRYJwhDDhgCSCSSEIQYwILoyAjAIigBFEUQK8gAYAQ5BCAAjkjCCAEEMZAUQAZQCjCCkpCgFMCCiIcVIAZZgilAQAiSHQECOcQAQIc4QClAHAjDDGkAGAMUoBgyhihgEChFCAAWEIEYwIJYwViAAlHCBIGEIEAEIQAoBwwgwiEBAEEEOoEwBY4wRwxAhBgAcKAESIQAwwIowRFhoBhAE"),
        //("duration", "641"), // song duration
        ("format", "json"), // response format
        ("client", &api_key),
        ("duration", &duration), // song duration
        ("fingerprint", &fingerprint),
        ("meta", "recordings"),
    ]);

    eprintln!(
        "Request: {song} with duration {duration} seconds and acoustid {display_short_acoustid}"
    );

    let bytes = client
        .post(url)
        .form(&map)
        .send()?
        .error_for_status()?
        .bytes()?;
    eprintln!("response bytes: {bytes:?}");
    let json: Post =
        serde_json::from_slice(bytes.as_ref()).with_context(|| "unexpected response")?;

    let mut candidates = json.results;
    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    let most_likely_candidate = candidates
        .first()
        .ok_or(anyhow!(""))?
        .recordings
        .iter()
        .find(|x| x.title.is_some() && x.artists.is_some())
        .ok_or(anyhow!(""))?;

    let title = most_likely_candidate.title.clone().unwrap();
    let artist = most_likely_candidate
        .artists
        .as_ref()
        .unwrap() // won't panic because of find(...)
        .first()
        .ok_or(anyhow!(""))?
        .name
        .clone()
        .ok_or(anyhow!(""))?;

    Ok(SongData { title, artist })
}
