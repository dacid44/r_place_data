use std::fmt;
use std::fs::File;
use std::io::{BufReader, Lines};
use std::io::prelude::*;
use std::str::FromStr;
use flate2::read::GzDecoder;
use chrono::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

pub struct PlaceData {
    lines: Lines<BufReader<GzDecoder<File>>>,
}

impl PlaceData {
    pub fn new() -> Self {
        Self {
            lines: BufReader::new(GzDecoder::new(
                File::open("./place_data.csv.gzip").expect("No data file")
            )).lines(),
        }
    }
}

impl Iterator for PlaceData {
    type Item = PlaceEntry;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

enum DrawType {
    Pixel(u32, u32),
    Rect(u32, u32, u32, u32),
}

pub struct PlaceEntry {
    timestamp: DateTime<Utc>,
    uid_hash: String,
    color: String,
    location: DrawType,
}

impl FromStr for PlaceEntry {
    type Err = EntryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref ENTRY_RE: Regex = Regex::new(
                r#"^(?'time'.+),(?'uid'.+),(?'color'.+),"(?:(?:(?'x'\d+),(?'y'\d+))|(?:(?'x1'\d+),(?'y1'\d+),(?'x2'\d+),(?'y2'\d+)))\"$"#
            ).unwrap();
        }

        let caps = ENTRY_RE.captures(s).ok_or_else(|| EntryParseError { entry: s.to_string() })?;
        let timestamp = DateTime::parse_from_str(
            caps.name("time").ok_or_else(|| EntryParseError { entry: s.to_string() })?.as_str(),
            "%Y-%m-%d %H:%M:%S%.f %Z",
        ).map_err(|_| EntryParseError { entry: s.to_string() })?.with_timezone(&Utc);
        let uid_hash = caps.name("uid").ok_or_else(|| EntryParseError { entry: s.to_string() })?.as_str().to_string();
        let color = caps.name("color").ok_or_else(|| EntryParseError { entry: s.to_string() })?.as_str().to_string();
        let location = if let Some(x) = caps.name("x") {
            DrawType::Pixel(
                x.as_str().parse().map_err(|_| EntryParseError { entry: s.to_string() })?,
                caps.name("y")
                    .map(|y| y.as_str().parse().ok())
                    .flatten()
                    .ok_or_else(|| EntryParseError { entry: s.to_string() })?,
            )
        } else {
            DrawType::Rect(
                caps.name("x1")
                    .map(|x1| x1.as_str().parse().ok())
                    .flatten()
                    .ok_or_else(|| EntryParseError { entry: s.to_string() })?,
                caps.name("y1")
                    .map(|y1| y1.as_str().parse().ok())
                    .flatten()
                    .ok_or_else(|| EntryParseError { entry: s.to_string() })?,
                caps.name("x2")
                    .map(|x2| x2.as_str().parse().ok())
                    .flatten()
                    .ok_or_else(|| EntryParseError { entry: s.to_string() })?,
                caps.name("y2")
                    .map(|y2| y2.as_str().parse().ok())
                    .flatten()
                    .ok_or_else(|| EntryParseError { entry: s.to_string() })?,
            )
        };
        Ok(Self { timestamp, uid_hash, color, location })
    }
}

#[derive(Debug, Clone)]
pub struct EntryParseError {
    entry: String,
}

impl fmt::Display for EntryParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not parse entry: '{}'", self.entry)
    }
}