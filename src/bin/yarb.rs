use std::fs;
use std::io::{Error, ErrorKind};

use yar::{Library, Album};
use clap::{arg, command};

fn main() -> Result<(), Error> {
    let matches = &command!()
        .arg(arg!(-i <path_in> "Path to the directory"))
        .get_matches();

    let path_in = matches
        .get_one::<String>("path_in")
        .expect("Input file path is not provided")
        .to_owned();


    let mut library = Library::new();
    scan_artists(&path_in, &mut library)?;

    let path_repos = format!("{}/repo/", path_in);
    if let Ok(paths) = fs::read_dir(path_repos) {
        for entry in paths {
            let path_repo = entry?;
            if !path_repo.file_type()?.is_dir() { continue; }
            if path_repo.file_name().to_str().unwrap().starts_with('.') { continue; }

            scan_artists(&path_repo.path().to_str().unwrap(), &mut library)?;

        }

    }


    let writer = std::io::stdout();
    match serde_yaml::to_writer(writer, &library) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::new(ErrorKind::InvalidData, "Could not serialize library"))
    }
}

fn scan_artists(path: &str, library: &mut Library) -> Result<(), Error>{
    // let path_artists = &join_paths([path, Path::new("artists/")]).unwrap();
    let path_artists = &format!("{}/artists/", path);
    let paths = fs::read_dir(path_artists).expect(&format!("no path at: {}", path_artists));
    for entry in paths {
        let path_artist = entry?;
        if !path_artist.file_type()?.is_dir() { continue; }
        if path_artist.file_name().to_str().unwrap().starts_with('.') { continue; }
        let artist_albums = fs::read_dir(path_artist.path()).unwrap();
        for entry in artist_albums {
            let file = entry?;
            if file.file_type()?.is_dir() { continue; }
            if file.file_name().to_str().unwrap().starts_with('.') { continue; }
            let f = std::fs::File::open(&file.path()).expect(&format!("Could not open file"));
            let album: Album = serde_yaml::from_reader(f).expect(&format!("could not read vals: {:?}", file.path().display()));

            library.albums.push(album);
        }
    }
    Ok(())
}