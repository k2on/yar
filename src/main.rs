
use std::env;
use yar::{read_library, process_library,Config};

use clap::{arg, command};


fn main() {
    let matches = &command!()
        .arg(arg!(-f <file> "Path to library structure file"))
        .arg(arg!(-d <dir> "Path to directory of the library"))
        .get_matches();

    let path_file = matches
        .get_one::<String>("file")
        .expect("File path is not provided")
        .to_owned();

    let path_library = matches
        .get_one::<String>("dir")
        .expect("Library directory")
        .to_owned();

    let library = &read_library(path_file);
    let config = &Config {
        debug_ytdl: true,
        debug_ffmpeg: true,
        debug: true,
        audio_fmt: "mp3",
        force: false,
        download_covers: true,
        keep_full_files: false,
    };

    process_library(config, &path_library, library).unwrap();

}
