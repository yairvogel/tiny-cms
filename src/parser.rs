use std::io::{BufReader, Read, BufRead};

use anyhow::{Error, Context};
use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use regex::Regex;

use super::Post;

lazy_static! {
    static ref DASHES: Regex = Regex::new(r"^-{3}-*$").unwrap();
    static ref TITLE: Regex = Regex::new(r"^title: +([\w-]+)$").unwrap();
    static ref DATE: Regex = Regex::new(r"^date published: +(.+)$").unwrap();
}

const DATE_FMT: &str = "%d/%m/%Y %H:%M";


pub fn parse<R: Read>(input: &mut R) -> Result<Post, Error> {
    let mut reader = BufReader::new(input);
    let mut line = read_line(&mut reader)?; 
    if !DASHES.is_match(&line) {
        return Err(unexpected_line("dashes 1", &line));
    }

    line = read_line(&mut reader)?;
    let title = TITLE.captures(&line)
        .ok_or(unexpected_line("title: {any title}", &line))?;
    let title = title.get(1)
        .expect("if matched, we found a capture group")
        .as_str()
        .to_string();

    line = read_line(&mut reader)?;
    let date = DATE.captures(&line)
        .ok_or(unexpected_line("date published: {valid date time}", &line))?
        .get(1)
        .expect("if matched, we found a capture group")
        .as_str();

    let date = NaiveDateTime::parse_from_str(date, DATE_FMT)
        .context(format!("failed to parse date, expected {}, found {}", DATE_FMT, &date))?;

    line = read_line(&mut reader)?; 
    if !DASHES.is_match(&line) {
        return Err(unexpected_line("dashes", &line));
    }
    let mut content = String::new();
    reader.read_to_string(&mut content).context("failed reading content")?;

    return Ok(Post {
        title,
        published: date.and_utc(),
        content
    })
}

fn read_line<R: Read>(lines: &mut BufReader<R>) -> Result<String, Error> {
    let mut buf = String::new();
    match lines.read_line(&mut buf) {
        Err(error) => Err(Error::new(error).context("failed reading line")),
        Ok(0) => Err(Error::msg("unexpected EOF")),
        Ok(_) => {
            buf.pop();
            Ok(buf)
        }
    }
}

fn unexpected_line(expected: &'static str, found: &str) -> Error {
    Error::msg(format!("expected line to be '{}', found '{}'", expected, found))
}
