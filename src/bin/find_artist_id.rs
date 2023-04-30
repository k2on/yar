use std::io::Error;

use clap::{arg, command};
use yar::read_library;

fn main() -> Result<(), Error> {
    let matches = &command!()
        .arg(arg!(-i <lib_file> "Path to the library"))
        .arg(arg!(-a <artist> "Artist id on location"))
        .get_matches();

    let path_file = matches
        .get_one::<String>("lib_file")
        .expect("Library file path is not provided")
        .to_owned();

    let artist = matches
        .get_one::<String>("artist")
        .expect("Artist is not provided")
        .to_owned();

    let library = &read_library(path_file);

    if let Some(album) = &library.albums.iter().find(|alb| {
        alb.tracks.iter().any(|(_, track)| 
            if let Some(artists) = &track.artists {
                artists.len() == 1
            } else {
                true
            }
            && track.location.iter().any(|loc| loc.url.contains(&artist))
        )
    }) {
        println!("{}", album.artist);
        Ok(())
    } else {
        Err(Error::new(std::io::ErrorKind::NotFound, "Artist not found"))
    }
}
