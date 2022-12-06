use std::io::Error;
use std::process::Command;
use crate::{get_stdout, Config};

pub fn split_track(
    config: &Config,
    path_full: &str,
    path_out: &str,
    start: &str,
    end: Option<String>,
) -> Result<(), Error> {
    let args = get_ffmpeg_args(path_full, start, end, path_out);
    let stdout = get_stdout(config.debug_ffmpeg);
    let result = Command::new("ffmpeg").args(args).stdout(stdout).output();
    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

fn get_ffmpeg_args(
    path_full: &str,
    start: &str,
    end: Option<String>,
    path_out: &str,
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        String::from("-i"),
        String::from(path_full),
        String::from("-vn"),
        String::from("-c:a"),
        String::from("copy"),
        String::from("-ss"),
        String::from(start),
    ];
    if let Some(end) = end {
        args.push(String::from("-to"));
        args.push(end);
    }
    args.push(String::from(path_out));
    args
}
