mod tagger;
mod downloader;

use std::{collections::HashMap, process::Stdio, fs::{create_dir_all, remove_file}, io::Error};
use downloader::{download_track, get_cover};
use tagger::tag_track;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Library {
    pub albums: Vec<Album>,
}

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
    pub keep_full_files: bool,
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
    if debug {
        return Stdio::inherit();
    }
    Stdio::null()
}

/// Returns a path to an album dir.
///
/// # Example
/// ```
/// let album = &yar::Album {
///     artist: String::from("My Artist"),
///     name: String::from("Album Name"),
///     cover: String::new(),
///     genre: String::new(),
///     released: 0,
///     track_count: 0,
///     tracks: std::collections::HashMap::new(),
/// };
/// let path = yar::get_path_album("./library/", album);
/// assert_eq!(path, "./library/my-artist/album-name/");
/// ```
pub fn get_path_album(path_library: &str, album: &Album) -> String {
    let path_artist = parse_name(&album.artist);
    let album_dir_name = parse_name(&album.name);
    format!("{}{}/{}/", path_library, path_artist, album_dir_name)
}

/// Parses a name (artist or album) for the file system.
///
/// 1. Converts the name to lowercase
/// 2. Removes all special characters
/// 3. Replaces spaces with hythens
///
/// # Example
/// ```
/// let name = yar::parse_name("Don't Play With The Gang");
/// assert_eq!(name, "dont-play-with-the-gang");
/// ```
pub fn parse_name(name: &str) -> String {
    let is_space = |c: char| c == ' ';
    let is_special = |c: char| !c.is_ascii_alphanumeric() && !is_space(c);
    name.to_lowercase()
        .replace(is_special, "")
        .replace(is_space, "-")
}

/// Returns a library struct from a file path.
pub fn read_library(path: String) -> Library {
    let f = std::fs::File::open(path).expect("Could not open file");
    let library: Library = serde_yaml::from_reader(f).expect("could not read vals");
    library
}

pub fn process_library(config: &Config, path_library: &str, library: &Library) -> Result<(), Error> {
    for album in &library.albums {
        process_library_album(config, &path_library, album).unwrap();
    }
    Ok(())
}

fn process_library_album(config: &Config, path_library: &str, album: &Album) -> Result<(), Error> {
    let path_album = get_path_album(&path_library, album);

    println!("{}", &path_album);
    create_dir_all(&path_album).unwrap();

    let path_cover = format!("{}cover.jpg", path_album);
    let cover = get_cover(config, &path_cover, &album);

    let mut full_files = vec![];

    for (track_postion, track) in album.tracks.iter() {
        process_library_album_track(
            config,
            &mut full_files,
            &path_album,
            album,
            cover.clone(),
            &track_postion,
            track,
        )
    }
    clean_up_album(&config, &full_files)
}

fn clean_up_album(config: &Config, full_files: &Vec<String>) -> Result<(), Error> {
    let remove_full_files = !config.keep_full_files;
    for path in full_files {
        if remove_full_files {
            if let Err(err) = remove_full_file(config, path) {
                println!("ERR: Could not remove full file: {}", path);
                println!("{}", err)
            }
        }
    }
    Ok(())
}

fn remove_full_file(config: &Config, path: &str) -> Result<(), Error> {
    if config.debug {
        println!("removing full file: {}", path);
    };
    remove_file(path)
}

fn process_library_album_track(
    config: &Config,
    full_files: &mut Vec<String>,
    path_album: &str,
    album: &Album,
    cover: Vec<u8>,
    track_position: &str,
    track: &Track,
) {
    println!("{}: {}", track_position, track.name);

    let track_name = &track.name;
    let path_out = &format!(
        "{}{} - {}.{}",
        &path_album, track_position, track_name, config.audio_fmt
    );

    let result = download_track(
        config,
        full_files,
        &path_album,
        &path_out,
        &album,
        &track,
        track_position,
    );

    match result {
        Ok(_) => match tag_track(&path_out, &album, track, &track_position, cover.clone()) {
            Ok(_) => println!("wrote tags!!"),
            Err(_) => panic!("could not write tags"),
        },
        Err(_) => (),
    }
}