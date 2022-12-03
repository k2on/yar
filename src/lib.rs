use std::{collections::HashMap, io::Error};

use serde::Deserialize;

const DELIMITER_DURATION: &str = ":";

const SECONDS_HOUR: i32 = 60 * 60;
const SECONDS_MIN: i32 = 60;

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

/// Returns an integer plus one as a string.
///
/// # Examples
///
/// Basic usage
///
/// ```
/// let two = yar::get_next_str("1");
/// assert!(!two.is_err());
/// assert_eq!(two.unwrap(), String::from("2"))
/// ```
pub fn get_next_str(pos: &str) -> Result<String, Error> {
    let pos_int: i8 = pos.parse().unwrap();
    let pos_int_next = pos_int + 1;
    Ok(pos_int_next.to_string())
}

/// Converts a duration string to number of seconds.

/// # Example
///
/// Basic usage
///
/// ```
/// let seconds = yar::duration_seconds_parse("4:20");
///
/// assert!(!seconds.is_err());
/// assert_eq!(seconds.unwrap(), 260);
/// ```
pub fn duration_seconds_parse(duration: &str) -> Result<i32, Error> {
    let mut parts = Vec::from_iter(duration.split(DELIMITER_DURATION));
    parts.reverse();
    let mut result = 0;
    let mut idx = 0;
    for part in parts {
        let part_parsed: i32 = part.parse().unwrap();
        let base: i32 = 60;
        let to_add = part_parsed * base.pow(idx);
        result += to_add;
        idx += 1;
    }
    Ok(result)
}

/// Converts a number of seconds to a duration string.

/// # Example
///
/// Basic usage
///
/// ```
/// let duration = yar::duration_seconds_format(260);
///
/// assert!(!duration.is_err());
/// assert_eq!(duration.unwrap(), "0:4:20");
/// ```
pub fn duration_seconds_format(seconds_total: i32) -> Result<String, Error> {
    let mut sec = seconds_total;
    let hour = sec / SECONDS_HOUR;
    sec -= hour * SECONDS_HOUR;

    let min = sec / SECONDS_MIN;
    sec -= min * SECONDS_MIN;

    Ok(format!("{}:{}:{}", hour, min, sec))
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
