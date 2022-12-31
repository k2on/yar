// Converts a bandcamp URL to an Album definition
use std::{io::{Error, ErrorKind}, collections::HashMap, hash::Hash, thread::ScopedJoinHandle, str::FromStr};
use clap::{arg, command};
use serde::Deserialize;
use yar::{duration_seconds_format, Album, Track, Location, Wave, TrackArtist};
use chrono::{NaiveDate, DateTime};


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BandcampAlbum {
    name: String,
    by_artist: BandcampArtist,
    num_tracks: i8,
    date_published: String,
    image: String,
    track: BandcampAlbumTracks,
}

type BandcampAlbumTracks = BandcampList<BandcampTrack>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BandcampList<T> {
    item_list_element: Vec<BandcampListItem<T>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BandcampListItem<T> {
    position: i8,
    item: T,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BandcampArtist {
    name: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BandcampTrack {
    name: String,
    duration: String,
    by_artist: Option<BandcampArtist>,
    main_entity_of_page: String,
}

type SoundcloudHydration = Vec<SoundcloudHydrationData>;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SoundcloudHydrationData {
    hydratable: String,
    data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct SoundcloudSound {
    artwork_url: String,
    title: String,
    genre: String,
    waveform_url: String,
    user: SoundcloudUser,
    description: String,
    created_at: String,
    duration: i32,
}

#[derive(Debug, Deserialize)]
struct SoundcloudUser {
    username: String,
}

#[derive(Debug, Deserialize)]
struct SoundcloudWave {
    width: i32,
    height: i32,
    samples: Vec<i32>,
}

fn get_bandcamp_album(json: &str) -> Result<BandcampAlbum, Error> {
    let bandcamp_album: BandcampAlbum = serde_json::from_str(json)?;
    Ok(bandcamp_album)
}

fn parse(album: BandcampAlbum) -> Result<Album, Error> {
    let name = album.name;
    let artist = album.by_artist.name;
    let fmt = "%d %b %Y %T %Z";
    let released = NaiveDate::parse_from_str(&album.date_published, fmt).unwrap();
    let genre = String::from("Phonk");
    let cover = album.image;
    let track_count = album.num_tracks;
    let tracks = parse_tracks(album.track);

    let parsed = Album { name, artist, genre, released, cover, tracks, track_count, };
    Ok(parsed)
}

fn parse_tracks(tracks: BandcampAlbumTracks) -> HashMap<String, Track> {
    let mut result = HashMap::new();
    for element in tracks.item_list_element {
        let position = element.position.to_string();
        let track = parse_track(&element.item);
        result.insert(position, track);
    }
    result
}

fn parse_track(track: &BandcampTrack) -> Track {
    let name = track.name.to_owned();
    let duration = parse_duration(&track.duration);
    let artists = parse_artists(&track);
    let location = parse_location(&track);
    let sample = None;
    let lyrics = None;
    let wave = None;

    Track { name, duration, artists, location, sample, lyrics, wave, artist: None }
}

fn parse_duration(duration: &str) -> Option<String> {
    let result = String::from(duration)
        .replace("P", "")
        .replace("S", "")
        .replace("H", ":")
        .replace("M", ":");

    Some(result)
}

fn parse_artists(track: &BandcampTrack) -> Option<Vec<TrackArtist>> {
    match &track.by_artist {
        Some(artists) => {
            let names_str = artists.name.to_owned();
            let names = names_str
                .split(", ")
                .flat_map(|name| -> Vec<String> {
                    String::from(name)
                        .split(" & ")
                        .map(|n| String::from(n))
                        .collect()
                })
                .map(|id| TrackArtist { id, r#for: None })
                .collect();
            Some(names)
        },
        None => None,
    }

}

fn parse_location(track: &BandcampTrack) -> Vec<Location> {
    let url = track.main_entity_of_page.to_owned();
    let at = None;
    let location = Location { url, at };

    vec![location]
}

fn get_album_from_url(url: &str) -> Result<Album, Error> {
    let html = reqwest::blocking::get(url).unwrap().text().unwrap();
    let json = extract_json_from_html(&html)?;
    let bandcamp = get_bandcamp_album(&json)?;
    let parsed = parse(bandcamp)?;
    Ok(parsed)
}

fn extract_json_from_html(html: &str) -> Result<String, Error> {
    let after = html.split("<script type=\"application/ld+json\">").last().unwrap().to_owned();
    let parts: Vec<&str> = after.split("</script>").collect();
    let json = parts[0].trim().to_owned();
    Ok(json)
}

fn soundcloud_extract_json_from_html(html: &str) -> Result<String, Error> {
    let after = html.split("<script>window.__sc_hydration = ").last().unwrap().to_owned();
    let parts: Vec<&str> = after.split(";</script>").collect();
    let json = parts[0].trim().to_owned();
    Ok(json)
}

fn soundcloud_get_json(url: &str) -> Result<String, Error> {
    let html = reqwest::blocking::get(url).unwrap().text().unwrap();
    soundcloud_extract_json_from_html(&html)
}

fn soundcloud_parse_json(json: &str) -> Result<SoundcloudSound, Error> {
    let hydration: SoundcloudHydration = serde_json::from_str(json)?;
    for data in hydration {
        if data.hydratable == "sound" {
            let sound: SoundcloudSound = serde_json::from_value(data.data).unwrap();
            return Ok(sound)
        }
    }
    Err(Error::new(ErrorKind::InvalidData, "Could not parse Soundcloud data"))
}

/// Convert a soundcloud wave type into a Wave type.
fn soundcloud_parse_wave(wave: SoundcloudWave) -> Wave {
    let max = wave.height;
    let length = wave.width;
    let points = wave.samples
        .iter()
        .map(|point| ((*point as f32 / max as f32) * u8::MAX as f32) as u8)
        .collect();
    Wave {
        length,
        points,
    }
}

fn get_wave(url_wave: &str) -> Result<Wave, Error> {
    let wave_json = reqwest::blocking::get(url_wave).unwrap().text().unwrap();
    let wave_sc: SoundcloudWave = serde_json::from_str(&wave_json).unwrap();
    let wave = soundcloud_parse_wave(wave_sc);
    Ok(wave)
}

fn soundcloud_parse_sound(url: &str, sound: SoundcloudSound) -> Result<Album, Error> {
    let name = sound.title;
    let artist = sound.user.username;
    let genre = sound.genre;
    let released = DateTime::parse_from_rfc3339(&sound.created_at).unwrap().date_naive();
    let cover = sound.artwork_url.replace("-large.jpg", "-t500x500.jpg");
    let duration = Some(duration_seconds_format(sound.duration / 1000)?);
    let track_count = 1;
    let mut tracks = HashMap::new();

    let artists = None;
    let location = vec![Location {
        url: String::from(url),
        at: None,
    }];
    let sample = None;
    let lyrics = None;
    let wave = Some(get_wave(&sound.waveform_url).unwrap());

    // println!("{}", sound.description);

    let track = Track {
        name: name.clone(),
        duration,
        artists,
        artist: None,
        location,
        sample,
        lyrics,
        wave,
    };
    tracks.insert(String::from("1"), track);



    let album = Album {
        name,
        artist,
        genre,
        released,
        cover,
        tracks,
        track_count,
    };

    Ok(album)
}

fn soundcloud_get(url: &str) -> Result<Album, Error> {
    let json = soundcloud_get_json(&url)?;
    let sound = soundcloud_parse_json(&json)?;
    soundcloud_parse_sound(url, sound)
}

fn main() -> Result<(), Error> {
    let matches = &command!()
        .arg(arg!(-b <bandcamp> "Bandcamp URL"))
        .arg(arg!(-s <soundcloud> "Soundcloud URL"))
        .get_matches();
    
    let bandcamp = matches
        .get_one::<String>("bandcamp")
        .to_owned();
    
    let soundcloud = matches
        .get_one::<String>("soundcloud")
        .to_owned();

    if let Some(url) = bandcamp {
        let parsed = get_album_from_url(&url)?;

        let writer = std::io::stdout();
        match serde_yaml::to_writer(writer, &parsed) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(ErrorKind::InvalidData, "Could not serialize library"))
        }
        
    } else if let Some(url) = soundcloud {
        let parsed = soundcloud_get(&url)?;

        let writer = std::io::stdout();
        match serde_yaml::to_writer(writer, &parsed) {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(ErrorKind::InvalidData, "Could not serialize library"))
        }
        
    } else {
        Err(Error::new(ErrorKind::InvalidInput, "provide either a soundcloud or bandcamp url"))
    }
    

}
