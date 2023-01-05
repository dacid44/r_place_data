use std::{fmt, thread};
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Lines, BufRead, Write};
use std::iter::Skip;
use std::path::Path;
use std::str::FromStr;
use flate2::read::GzDecoder;
use chrono::prelude::*;
use lazy_static::lazy_static;
use ndarray::{ArrayViewMut, s};
use regex::Regex;

type DataLines = Skip<Lines<BufReader<GzDecoder<File>>>>;
pub type Rect = (u32, u32, u32, u32);

pub struct PlaceData {
    pub lines: DataLines,
}

impl PlaceData {
    pub fn new() -> Self {
        Self {
            lines: get_gzip_reader("./place_data.csv.gzip"),
        }
    }
}

impl Iterator for PlaceData {
    type Item = Result<PlaceEntry, EntryError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next()
            .map(|x| x.map_err(|y| EntryError::IoError(y)))
            .map(|x| x.and_then(|y| y.parse()))
    }
}

pub struct ParallelPlaceData;

impl ParallelPlaceData {
    pub fn pixel_frequencies(print_status: usize) -> ndarray::Array2<usize> {
        let mut threads = (0..12)
            .map(|n| thread::spawn(move ||
                Self::pixel_frequencies_fragment(n, print_status.clone())))
            .collect::<Vec<_>>();

        threads.into_iter().map(|x| x.join().unwrap())
            .reduce(|arr, y| arr + y)
            .unwrap()
    }

    fn pixel_frequencies_fragment(n: usize, print_status: usize) -> ndarray::Array2<usize> {
        if n >= 12 {
            panic!("invalid fragment index")
        }
        let mut pixels = ndarray::Array2::zeros((2000, 2000));
        let mut i: usize = 0;

        for line in get_gzip_reader(format!("./data2/place_data_{}.csv.gzip", n + 1))
            .map(|x| x.map_err(|y| EntryError::IoError(y)))
            .map(|x| x.and_then(|y| y.parse::<PlaceEntry>()))
        {
            if let DrawType::Pixel(x, y) = line.unwrap().location {
                if let Some(p) = pixels.get_mut((y as usize, x as usize)) {
                    *p += 1;
                }
            }

            if print_status != 0 {
                if i % print_status == 0 {
                    println!("thread {}: {}", n, i);
                    std::io::stdout().flush();
                }
                i += 1;
            }
        }

        pixels
    }

    pub fn pixel_frequencies_nom(print_status: usize) -> ndarray::Array2<usize> {
        let mut threads = (0..12)
            .map(|n| thread::spawn(move ||
                Self::pixel_frequencies_nom_fragment(n, print_status.clone())))
            .collect::<Vec<_>>();

        threads.into_iter().map(|x| x.join().unwrap())
            .reduce(|arr, y| arr + y)
            .unwrap()
    }


    fn pixel_frequencies_nom_fragment(n: usize, print_status: usize) -> ndarray::Array2<usize> {
        if n >= 12 {
            panic!("invalid fragment index")
        }
        let mut pixels = ndarray::Array2::zeros((2000, 2000));
        let mut i: usize = 0;

        for line in get_gzip_reader(format!("./data2/place_data_{}.csv.gzip", n + 1))
            .map(|x| x.map_err(|y| EntryError::IoError(y)))
            .map(|x| x.and_then(|y| PlaceEntry::parse_nom(&y)))
        {
            if let DrawType::Pixel(x, y) = line.unwrap().location {
                if let Some(p) = pixels.get_mut((y as usize, x as usize)) {
                    *p += 1;
                }
            }

            if print_status != 0 {
                if i % print_status == 0 {
                    println!("thread {}: {}", n, i);
                    std::io::stdout().flush();
                }
                i += 1;
            }
        }

        pixels
    }

    pub fn find_rects() -> Vec<PlaceEntry> {
        let mut threads = (0..12)
            .map(|n| thread::spawn(move ||
                Self::find_rects_fragment(n)))
            .collect::<Vec<_>>();

        let mut rects: Vec<PlaceEntry> = Vec::new();

        for t in threads.into_iter() {
            rects.extend(t.join().unwrap().into_iter())
        }

        rects
    }

    fn find_rects_fragment(n: usize) -> Vec<PlaceEntry> {
        if n >= 12 {
            panic!("invalid fragment index")
        }

        get_gzip_reader(format!("./data/place_data_{}.csv.gzip", n + 1))
            .map(|x| x.map_err(EntryError::IoError))
            .map(|x| x.and_then(|y| y.parse::<PlaceEntry>()))
            .flatten()
            .filter(|x| matches!(x.location, DrawType::Rect(_, _, _, _)))
            .collect()
    }

    pub fn find_before_rects(rects: &Vec<(Rect, i64)>) -> Vec<(Rect, ndarray::Array2<Option<PlaceEntry>>)> {
        let mut threads = (0..12)
            .map(|n| {
                let r = rects.clone();
                thread::spawn(move ||
                    Self::find_before_rects_fragment(n, r))
            })
            .collect::<Vec<_>>();

        threads.into_iter().map(|x| x.join().unwrap())
            .reduce(|mut r, next| {
                for (i, (rect, pixels)) in next.into_iter().enumerate() {
                    for (old_pixel, new_pixel) in r[i].1.iter_mut().zip(pixels.into_iter()) {
                        if let Some(new) = new_pixel {
                            if matches!(&*old_pixel, Some(old) if new.timestamp > old.timestamp) || old_pixel.is_none() {
                                *old_pixel = Some(new)
                            }
                        }
                    }
                }
                r
            })
            .unwrap()
    }

    fn find_before_rects_fragment(
        n: usize,
        rects: Vec<(Rect, i64)>,
    ) -> Vec<(Rect, ndarray::Array2<Option<PlaceEntry>>)> {
        if n >= 12 {
            panic!("invalid fragment index")
        }

        let mut before_rects: Vec<(Rect, i64, ndarray::Array2<Option<PlaceEntry>>)> = rects.into_iter()
            .map(|(r, ts)| (r, ts, ndarray::Array2::default(((r.2 - r.0 + 1) as usize, (r.3 - r.1 + 1) as usize))))
            .collect();

        for entry in get_gzip_reader(format!("./data/place_data_{}.csv.gzip", n + 1))
            .map(|x| x.map_err(|y| EntryError::IoError(y)))
            .map(|x| x.and_then(|y| y.parse::<PlaceEntry>()))
        {
            let entry = entry.unwrap();
            for (rect, ts, pixels) in before_rects.iter_mut() {
                match entry.location {
                    DrawType::Pixel(x, y) => {
                        if in_rect(rect, (x, y))
                            && pixels[((x - rect.0) as usize, (y - rect.1) as usize)].as_ref().map_or(true, |e| entry.timestamp > e.timestamp)
                            && entry.timestamp.timestamp_millis() < *ts
                        {
                            pixels[((x - rect.0) as usize, (y - rect.1) as usize)] = Some(entry.clone());
                        }
                    }
                    DrawType::Rect(x1, y1, x2, y2) => {
                        // println!("{:?}, {:?}", rect, (x1, y1, x2, y2));
                        // if rects_intersect(rect, &(x1, y1, x2, y2)) {
                        //     pixels.slice_mut(s![
                        //         max(x1 as isize - rect.0 as isize, 0) as usize..=min(x2 as isize - rect.2 as isize, pixels.nrows() as isize) as usize,
                        //         max(y1 as isize - rect.1 as isize, 0) as usize..=min(y2 as isize - rect.3 as isize, pixels.ncols() as isize) as usize
                        //     ]).iter_mut().map(|e| *e = Some(entry.clone())).count();
                        // }
                    }
                }
            }
        }

        before_rects.into_iter()
            .map(|(rect, _, pixels)| (rect, pixels))
            .collect()
    }
}

fn get_gzip_reader<P: AsRef<Path>>(path: P) -> DataLines {
    BufReader::new(GzDecoder::new(File::open(path).expect("No data file")))
        .lines()
        .skip(1)
}

fn in_rect(rect: &Rect, point: (u32, u32)) -> bool {
    point.0 >= rect.0 && point.0 <= rect.2 && point.1 >= rect.1 && point.1 <= rect.3
}

fn rects_intersect(rect1: &Rect, rect2: &Rect) -> bool {
    rect2.0 <= rect1.2
        && rect2.2 >= rect1.0
        && rect2.1 <= rect2.3
        && rect2.3 >= rect1.1
}

#[derive(Debug, Clone)]
pub enum DrawType {
    Rect(u32, u32, u32, u32),
    Pixel(u32, u32),
}

#[derive(Debug, Clone)]
pub struct PlaceEntry {
    pub timestamp: NaiveDateTime,
    pub uid_hash: String,
    pub color: String,
    pub location: DrawType,
}

impl FromStr for PlaceEntry {
    type Err = EntryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref ENTRY_RE: Regex = Regex::new(
                r#"^(?P<time>.+),(?P<uid>.+),(?P<color>.+),"(?:(?:(?P<x>\d+),(?P<y>\d+))|(?:(?P<x1>\d+),(?P<y1>\d+),(?P<x2>\d+),(?P<y2>\d+)))"$"#
            ).unwrap();
        }

        let caps: regex::Captures = ENTRY_RE.captures(s).ok_or_else(|| EntryError::ParseError { entry: s.to_string() })?;
        let timestamp = NaiveDateTime::parse_from_str(
            caps.name("time").unwrap().as_str(),
            "%F %T%.f %Z",
        ).map_err(|x| EntryError::DateTimeParseError(x))?;
        let uid_hash = caps.name("uid").unwrap().as_str().to_string();
        let color = caps.name("color").unwrap().as_str().to_string();
        let location = if let Some(x) = caps.name("x") {
            DrawType::Pixel(
                x.as_str().parse().unwrap(),
                caps.name("y").unwrap().as_str().parse().unwrap(),
            )
        } else {
            DrawType::Rect(
                caps.name("x1").unwrap().as_str().parse().unwrap(),
                caps.name("y1").unwrap().as_str().parse().unwrap(),
                caps.name("x2").unwrap().as_str().parse().unwrap(),
                caps.name("y2").unwrap().as_str().parse().unwrap(),
            )
        };
        Ok(Self { timestamp, uid_hash, color, location })
    }
}

impl PlaceEntry {
    fn parse_nom(s: &str) -> Result<Self, EntryError> {
        crate::parser::parse_entry(s)
            .map(|x| x.1)
            // .unwrap())
            .map_err(|_| EntryError::ParseError { entry: s.to_string() })
    }
}

#[derive(Debug)]
pub enum EntryError {
    ParseError { entry: String },
    DateTimeParseError(chrono::ParseError),
    IoError(std::io::Error),
}

impl fmt::Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError { entry } => {
                write!(f, "could not parse entry: '{}'", entry)
            }
            Self::DateTimeParseError(err) => write!(f, "datetime parse error: {:?}", err),
            Self::IoError(err) => write!(f, "io error: {:?}", err),
        }
    }
}