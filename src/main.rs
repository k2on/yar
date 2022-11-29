use std::collections::HashMap;
use std::process::{Command, Output, Stdio};

use serde::Deserialize;
use serde_yaml::{self};

#[derive(Debug, Deserialize)]
struct Album {
    name: String,
    artist: String,
    genre: String,
    duration: String,
    released: String,
    cover: String,
    tracks: HashMap<String, Track>,
    track_count: i8,
}

#[derive(Debug, Deserialize)]
struct Track {
    name: String,
    location: Vec<Location>,
    sample: Option<Vec<Sample>>,
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
    from: Option<String>,
    to: Option<String>,
}

struct Config<'a> {
    debug_ytdl: bool,
    audio_fmt: &'a str,
}

fn read_file(path: String) -> Album {
    let f = std::fs::File::open(path).expect("Could not open file");
    let album: Album = serde_yaml::from_reader(f).expect("could not read vals");
    album
}

fn main() {
    let album = read_file("./album.yml".to_string());
    let config = Config {
        debug_ytdl: true,
        audio_fmt: "mp3",
    };

    for (id, track) in album.tracks {
        println!("{}: {}", id, track.name);

        for location in track.location {
            let result = {
                let url = &location.url;
                let audio_fmt = config.audio_fmt;

                let args = ["-x", url, "--audio-format", audio_fmt];
                let stdout = if config.debug_ytdl {
                    Stdio::inherit()
                } else {
                    Stdio::null()
                };
                let result = Command::new("youtube-dl")
                    .args(args)
                    .stdout(stdout)
                    .output();
                
                result

            };
            match result {
                Ok(_) => break,
                Err(_) => (),
            }

        }

    }
}
