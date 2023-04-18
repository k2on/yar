mod util;
use util::{get_end_time, has_full};

mod splitter;
use splitter::split_track;

use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::process::Command;
use crate::{get_stdout, Album, Config, Location, Track};

pub fn download_track(
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
                        return split_track(config, path_full, path_out, start, end);
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

fn download_track_at_location(
    config: &Config,
    path_out: &str,
    location: &Location,
) -> Result<(), Error> {
    let args = get_downloader_args(&location.url, config.audio_fmt, path_out);
    let stdout = get_stdout(config.debug_ytdl);
    let result = Command::new("youtube-dl")
        .args(args)
        .stdout(stdout)
        .output();
    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

fn get_downloader_args(url: &str, audio_fmt: &str, path_out: &str) -> Vec<String> {
    vec![
        String::from("-x"),
        String::from(url),
        String::from("-f"),
        String::from(audio_fmt),
        String::from("--output"),
        String::from(path_out),
    ]
}

pub fn get_cover(config: &Config, path_cover: &str, album: &Album) -> Vec<u8> {
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

fn write_cover(path: &str, image_bytes: &Vec<u8>) -> Result<(), Error> {
    let mut file = File::create(path)?;
    file.write_all(image_bytes)?;
    Ok(())
}
