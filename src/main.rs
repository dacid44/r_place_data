use std::fs::File;
use std::collections::HashMap;
use std::io::Write;
use std::time::Instant;
use flate2::Compression;
use flate2::write::GzEncoder;
use crate::data::PlaceData;

mod data;
mod parser;

fn main() {
    let start = Instant::now();
    {
        let testdata = data::ParallelPlaceData::pixel_frequencies_nom(0);
    }
    let delta = start.elapsed();
    println!("Time: {:.3?}", delta);
}
