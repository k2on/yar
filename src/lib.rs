use std::{collections::HashMap, process::Stdio};

use serde::Deserialize;


#[derive(Debug, Deserialize)]
pub struct Album {
    pub name: String,
    pub artist: String,
    pub genre: String,
    // pub duration: String,
    pub released: i32,
    pub cover: String,
    pub tracks: HashMap<String, Track>,
    pub track_count: i8,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub name: String,
    pub duration: Option<String>,
    pub artists: Option<Vec<String>>,
    pub remix: Option<String>,
    pub artist_cover: Option<String>,
    pub location: Vec<Location>,
    pub sample: Option<Vec<Sample>>,
    pub lyrics: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Location {
    pub url: String,
    pub at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Sample {
    pub artist: String,
    pub name: String,
    pub r#type: String,
    // from: Option<String>,
    // to: Option<String>,
}

pub struct Config<'a> {
    pub debug_ytdl: bool,
    pub debug_ffmpeg: bool,
    pub debug: bool,
    pub audio_fmt: &'a str,
    pub force: bool,
    pub download_covers: bool,
}


/// Creates a string representing the title of the track.
///
/// # Example
///
/// Basic usage
///
/// ```
/// let track = &yar::Track {
///     name: String::from("My Song"),
///     artists: Some(vec![String::from("John"), String::from("Andrew")]),
///     remix: Some(String::from("Dennis")),
///     artist_cover: Some(String::from("Chris")),
///     location: vec![],
///     duration: None,
///     lyrics: None,
///     sample: None
/// };
/// let title = yar::get_track_title(track);
/// assert_eq!(title, "My Song (Dennis Remix) (Chris Cover)");
/// ```
pub fn get_track_title(track: &Track) -> String {
    let name = &track.name;
    let remix = if let Some(artist) = &track.remix {
        format!(" ({} Remix)", artist)
    } else {
        String::new()
    };
    let cover = if let Some(artist) = &track.artist_cover {
        format!(" ({} Cover)", artist)
    } else {
        String::new()
    };

    format!("{}{}{}", name, remix, cover)
}

pub fn get_stdout(debug: bool) -> Stdio {
    if debug { return Stdio::inherit() }
    Stdio::null()
}