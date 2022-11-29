use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, stdout, Error, ErrorKind, Read, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::str::FromStr;
use std::{env, path};
use std::io::BufReader;
use reqwest;

use serde::Deserialize;
use serde_yaml::{self};

use clap::{arg, command, ArgMatches};

use id3::{frame, Tag, TagLike, Timestamp};

#[derive(Debug, Deserialize)]
struct Album {
    name: String,
    artist: String,
    genre: String,
    duration: String,
    released: i32,
    cover: String,
    tracks: HashMap<String, Track>,
    track_count: i8,
}

#[derive(Debug, Deserialize)]
struct Track {
    name: String,
    artists: Option<Vec<String>>,
    location: Vec<Location>,
    sample: Option<Vec<Sample>>,
    lyrics: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Location {
    url: String,
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

fn download_track(config: &Config, path_out: &str, track: &Track) -> Result<(), Error> {
    let should_download = !std::path::Path::new(path_out).exists() || config.force;
    if !should_download {
        if config.debug {
            println!("Skipping: {}", path_out)
        }
        return Ok(());
    }
    for location in track.location.iter() {
        match download_track_at_location(config, path_out, location) {
            Ok(_) => return Ok(()),
            Err(err) => {
                println!("Error: URL failed");
                println!("{}", err);
            }
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
    cover: Vec<u8>
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
        None => ()
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
        if config.debug { println!("downloading cover") }
        let resp = reqwest::blocking::get(cover_url);

        match resp {
            Ok(response) => {
                let bytes = response.bytes().unwrap().to_vec();
                if config.download_covers { write_cover(&path_cover, &bytes).unwrap() }
                bytes
            }
            Err(_) => {
                panic!("could not download cover")
            }
        }
    } else {
        if config.debug { println!("Skipping Cover: {}", path_cover) }
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
        debug: true,
        audio_fmt: "mp3",
        force: false,
        download_covers: true,
    };

    let path_cover = format!("{}cover.jpg", out_dir);
    let cover = get_cover(config, &path_cover, album);

    for (track_postion, track) in album.tracks.iter() {
        // println!("{}: {}", id, track.name);

        let track_name = &track.name;
        let path_out = &format!(
            "{}{} - {}.{}",
            out_dir, track_postion, track_name, config.audio_fmt
        );

        let result = download_track(config, &path_out, &track);

        match result {
            Ok(_) => match set_tags(&path_out, &album, track, &track_postion, cover.clone()) {
                Ok(_) => println!("wrote tags!!"),
                Err(_) => panic!("could not write tags"),
            },
            Err(_) => (),
        }
    }
}
