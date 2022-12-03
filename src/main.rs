use reqwest;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::process::{Command, Stdio};

use serde::Deserialize;
use serde_yaml::{self};

use clap::{arg, command, ArgMatches};

use id3::{frame, Tag, TagLike};

#[derive(Debug, Deserialize)]
struct Album {
    name: String,
    artist: String,
    genre: String,
    // duration: String,
    released: i32,
    cover: String,
    tracks: HashMap<String, Track>,
    track_count: i8,
}

#[derive(Debug, Deserialize)]
struct Track {
    name: String,
    duration: Option<String>,
    artists: Option<Vec<String>>,
    location: Vec<Location>,
    sample: Option<Vec<Sample>>,
    lyrics: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Location {
    url: String,
    at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Sample {
    artist: String,
    name: String,
    r#type: String,
    // from: Option<String>,
    // to: Option<String>,
}

struct Config<'a> {
    debug_ytdl: bool,
    debug_ffmpeg: bool,
    debug: bool,
    audio_fmt: &'a str,
    force: bool,
    download_covers: bool,
}

fn read_file(path: String) -> Album {
    let f = std::fs::File::open(path).expect("Could not open file");
    let album: Album = serde_yaml::from_reader(f).expect("could not read vals");
    album
}

fn get_out_dir(matches: &ArgMatches) -> String {
    let out_dir = matches.get_one::<String>("out_dir");

    match out_dir {
        Some(dir) => String::from(dir),
        None => {
            let path_buf = env::current_dir().expect("Could not get the current path");
            let dir_current = path_buf
                .to_str()
                .expect("Could not convert the path to a string");
            format!("{}/", dir_current)
        }
    }
}

fn extract_track_from_file(
    config: &Config,
    path_full: &str,
    path_out: &str,
    start: &str,
    end: Option<String>,
) -> Result<(), Error> {
    let mut args: Vec<String> = vec![
        String::from("-i"),
        String::from(path_full),
        String::from("-ss"),
        String::from(start),
    ];
    if let Some(end) = end {
        args.push(String::from("-to"));
        args.push(end);
    }
    args.push(String::from(path_out));
    // println!("{:?}", args);
    let stdout = if config.debug_ffmpeg {
        Stdio::inherit()
    } else {
        Stdio::null()
    };
    let result = Command::new("ffmpeg").args(args).stdout(stdout).output();

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

fn download_track_at_location(
    config: &Config,
    path_out: &str,
    location: &Location,
) -> Result<(), Error> {
    let url = &location.url;
    let audio_fmt = config.audio_fmt;

    let args = ["-x", url, "--audio-format", audio_fmt, "--output", path_out];
    let stdout = if config.debug_ytdl {
        Stdio::inherit()
    } else {
        Stdio::null()
    };
    let result = Command::new("youtube-dl")
        .args(args)
        .stdout(stdout)
        .output();

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

fn has_full(path_full: &str, files: &mut Vec<String>) -> bool {
    if files.contains(&path_full.to_string()) {
        return true;
    }
    if std::path::Path::new(&path_full).exists() {
        files.push(path_full.to_string());
        return true;
    }
    false
}

/// Returns an integer plus one as a string. 

/// # Examples
///
/// Basic usage
///
/// ```
/// let two: Result(String) = get_next_str("1");
///
/// assert_eq!("2", two);
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
/// let seconds: Result(String) = duration_seconds_parse("4:20");
/// 
/// assert_eq!(300, seconds);
/// ```
fn duration_seconds_parse(duration: &str) -> Result<i32, Error> {
    let parts = duration.split(":");
    let mut result = 0;
    let mut idx = 0;
    for part in parts {
        let part_parsed: i32 = part.parse().unwrap();
        result += part_parsed * (60 ^ idx);
        idx += 1;
    };
    Ok(result)
}

const SECONDS_HOUR: i32 = 60 * 60;
const SECONDS_MIN: i32 = 60;

/// Converts a number of seconds to a duration string.

/// # Example
/// 
/// Basic usage
/// 
/// ```
/// let duration: Result(String) = duration_seconds_format(300);
/// 
/// assert_eq!("4:30", duration);
/// ```
fn duration_seconds_format(seconds_total: i32) -> Result<String, Error> {
    println!("{}", seconds_total);

    let mut sec = seconds_total;
    let hour = sec / SECONDS_HOUR;
    sec -= hour * SECONDS_HOUR;
    
    let min = sec / SECONDS_MIN;
    sec -= min * SECONDS_MIN;

    println!("{}", format!("{}:{}:{}", hour, min, sec));
    Ok(String::from("4:20"))
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

fn get_end_time(album: &Album, track: &Track, track_pos_str: &str) -> Result<String, Error> {
    if let Some(duration) = &track.duration {
        // FIXME: this math is wrong
        let start_formatted = get_track_start_time(track)?;
        let start = duration_seconds_parse(&start_formatted)?;
        let duration = duration_seconds_parse(duration)?;
        let end = start + duration;
        return duration_seconds_format(end)
    }
    get_next_track_time(album, track_pos_str)
}

fn download_full(
    config: &Config,
    full_files: &mut Vec<String>,
    path_full: &str,
    location: &Location,
) -> Result<(), Error> {
    if !has_full(&path_full, full_files) {
        if config.debug {
            println!("Downloading full file: {}", path_full)
        }
        match download_track_at_location(config, path_full, location) {
            Ok(_) => {
                full_files.push(path_full.to_string());
                Ok(())
            }
            Err(err) => Err(err),
        }
    } else {
        if config.debug {
            println!("Already downloaded full file")
        }
        Ok(())
    }
}

fn download_track(
    config: &Config,
    full_files: &mut Vec<String>,
    out_dir: &str,
    path_out: &str,
    album: &Album,
    track: &Track,
    track_pos_str: &str,
) -> Result<(), Error> {
    let should_download = !std::path::Path::new(path_out).exists() || config.force;
    if !should_download {
        if config.debug {
            println!("Skipping: {}", path_out)
        }
        return Ok(());
    }
    for location in track.location.iter() {
        match &location.at {
            Some(start) => {
                let path_full = &format!("{}full.mp3", out_dir);
                match download_full(config, full_files, path_full, location) {
                    Ok(_) => {
                        let end = match get_end_time(album, track, track_pos_str) {
                            Ok(end) => Some(end),
                            Err(_) => None,
                        };
                        return extract_track_from_file(config, path_full, path_out, start, end);
                    }
                    Err(err) => {
                        println!("Error: URL failed");
                        println!("{}", err);
                        return Err(err);
                    }
                }
            }
            None => match download_track_at_location(config, path_out, location) {
                Ok(_) => return Ok(()),
                Err(err) => {
                    println!("Error: URL failed");
                    println!("{}", err);
                }
            },
        }
    }
    Result::Err(Error::new(ErrorKind::NotFound, "All locations failed ;("))
}

fn make_comment_from_sample(sample: &Sample) -> String {
    let sample_type = match sample.r#type {
        _ => "SAMPLE",
    };
    format!("{}: {}, {}", sample_type, sample.artist, sample.name)
}

fn make_comment(track: &Track) -> String {
    let samples = &track.sample;
    match samples {
        Some(samples) => samples
            .iter()
            .map(make_comment_from_sample)
            .collect::<Vec<String>>()
            .join("\n"),
        None => String::new(),
    }
}

// Still not sure how to do this, so this will just join the arist names
fn make_artist(album: &Album, track: &Track) -> String {
    match &track.artists {
        Some(artists) => artists.join(", "),
        None => album.artist.to_string(),
    }
}

fn set_tags(
    path_out: &str,
    album: &Album,
    track: &Track,
    track_pos_str: &str,
    cover: Vec<u8>,
) -> Result<(), Error> {
    let track_name = &track.name;
    let album_name = &album.name;
    let album_artist = &album.artist;
    let album_track_count: u32 = album.track_count.try_into().unwrap();
    let track_pos: u32 = track_pos_str.parse().unwrap();
    let genre = &album.genre;
    let year = album.released;
    let lyrics = &track.lyrics;
    let comment = make_comment(track);
    let artist = make_artist(album, track);

    let mut tag = match Tag::read_from_path(path_out) {
        Ok(tag) => tag,
        Err(_) => Tag::new(),
    };

    tag.set_album(album_name);
    tag.set_title(track_name);
    tag.set_album_artist(album_artist);
    tag.set_track(track_pos);
    tag.set_total_tracks(album_track_count);
    tag.set_genre(genre);
    tag.set_year(year);
    tag.set_artist(artist);

    if !comment.is_empty() {
        tag.add_frame(frame::Comment {
            lang: String::from("EN"),
            description: String::new(),
            text: comment,
        });
    }
    if !cover.is_empty() {
        tag.add_frame(frame::Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: frame::PictureType::Other,
            description: "cover".to_string(),
            data: cover,
        });
    }
    match lyrics {
        Some(lyrics) => {
            tag.add_frame(frame::Lyrics {
                lang: String::from("EN"),
                description: String::new(),
                text: String::from(lyrics),
            });
        }
        None => (),
    };

    match tag.write_to_path(path_out, id3::Version::Id3v24) {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("{}", err);
            Err(Error::new(ErrorKind::Unsupported, "tagging failed"))
        }
    }
}

fn write_cover(path: &str, image_bytes: &Vec<u8>) -> Result<(), Error> {
    let mut file = File::create(path)?;
    file.write_all(image_bytes)?;
    Ok(())
}

fn get_cover(config: &Config, path_cover: &str, album: &Album) -> Vec<u8> {
    let should_download_cover = !std::path::Path::new(path_cover).exists() || config.force;
    if should_download_cover {
        let cover_url = &album.cover;
        if config.debug {
            println!("downloading cover")
        }
        let resp = reqwest::blocking::get(cover_url);

        match resp {
            Ok(response) => {
                let bytes = response.bytes().unwrap().to_vec();
                if config.download_covers {
                    write_cover(&path_cover, &bytes).unwrap()
                }
                bytes
            }
            Err(_) => {
                panic!("could not download cover")
            }
        }
    } else {
        if config.debug {
            println!("Skipping Cover: {}", path_cover)
        }
        vec![]
    }
}

fn main() {
    let matches = &command!()
        .arg(arg!(-f <file_path> "The file to read from"))
        .arg(arg!(-o <out_dir> "The output directory of the album"))
        .get_matches();

    let out_dir = get_out_dir(matches);

    let file_path = matches
        .get_one::<String>("file_path")
        .expect("File path is not provided")
        .to_owned();

    let album = &read_file(file_path);
    let config = &Config {
        debug_ytdl: true,
        debug_ffmpeg: true,
        debug: true,
        audio_fmt: "mp3",
        force: false,
        download_covers: true,
    };

    let path_cover = format!("{}cover.jpg", out_dir);
    let cover = get_cover(config, &path_cover, album);

    let mut full_files = vec![];

    for (track_postion, track) in album.tracks.iter() {
        println!("{}: {}", track_postion, track.name);

        let track_name = &track.name;
        let path_out = &format!(
            "{}{} - {}.{}",
            out_dir, track_postion, track_name, config.audio_fmt
        );

        let result = download_track(
            config,
            &mut full_files,
            &out_dir,
            &path_out,
            &album,
            &track,
            &track_postion,
        );

        match result {
            Ok(_) => match set_tags(&path_out, &album, track, &track_postion, cover.clone()) {
                Ok(_) => println!("wrote tags!!"),
                Err(_) => panic!("could not write tags"),
            },
            Err(_) => (),
        }
    }
}
