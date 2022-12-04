mod downloader;
mod tagger;

use downloader::{download_track, get_cover};
use tagger::tag_track;
use std::env;
use yar::{
    Album, Config,
};

use clap::{arg, command, ArgMatches};


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

fn main() {
    let matches = &command!()
        .arg(arg!(-f <file_path> "Library structure"))
        .arg(arg!(-o <out_dir> "Directory of the library"))
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
            Ok(_) => match tag_track(&path_out, &album, track, &track_postion, cover.clone()) {
                Ok(_) => println!("wrote tags!!"),
                Err(_) => panic!("could not write tags"),
            },
            Err(_) => (),
        }
    }
}
