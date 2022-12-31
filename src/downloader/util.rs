use std::io::{Error, ErrorKind};

use crate::{duration_seconds_format, Album, Track};

const DELIMITER_DURATION: &str = ":";

pub fn has_full(path_full: &str, files: &mut Vec<String>) -> bool {
    if files.contains(&path_full.to_string()) {
        return true;
    }
    if std::path::Path::new(&path_full).exists() {
        files.push(path_full.to_string());
        return true;
    }
    false
}

pub fn get_end_time(album: &Album, track: &Track, track_pos_str: &str) -> Result<String, Error> {
    if let Some(duration) = &track.duration {
        // FIXME: this math is wrong
        let start_formatted = get_track_start_time(track)?;
        let start = duration_seconds_parse(&start_formatted)?;
        let duration = duration_seconds_parse(duration)?;
        let end = start + duration;
        return duration_seconds_format(end);
    }
    get_next_track_time(album, track_pos_str)
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
fn get_next_str(pos: &str) -> Result<String, Error> {
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
fn duration_seconds_parse(duration: &str) -> Result<i32, Error> {
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

fn get_next_track_time(album: &Album, track_pos_str: &str) -> Result<String, Error> {
    let next_pos = get_next_str(track_pos_str)?;
    let track = album.tracks.get(&next_pos);
    match track {
        Some(track) => get_track_start_time(track),
        None => Err(Error::new(
            ErrorKind::NotFound,
            format!("No track with pos: {}", next_pos),
        )),
    }
}

fn get_track_start_time(track: &Track) -> Result<String, Error> {
    for location in &track.location {
        if let Some(at) = &location.at {
            return Ok(at.to_string());
        }
    }
    Err(Error::new(
        ErrorKind::InvalidData,
        "Next track has no timestamp",
    ))
}
