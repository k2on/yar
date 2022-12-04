use std::io::{Error, ErrorKind};
use id3::{frame, Tag, TagLike};

use yar::{Album, Track, Sample, get_track_title};

pub fn tag_track(
    path_out: &str,
    album: &Album,
    track: &Track,
    track_pos_str: &str,
    cover: Vec<u8>,
) -> Result<(), Error> {
    let track_name = get_track_title(track);
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
