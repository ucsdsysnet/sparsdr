/*
 * Copyright 2019 The Regents of the University of California
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

//!
//! Reads compressed samples from a file and displays information about averages
//!
//! Usage: averages compressed-file-or-pipe
//!

extern crate sparsdr_reconstruct;

use std::env;
use std::fs::File;
use std::io::{BufReader, Result};
use std::process;

use sparsdr_reconstruct::input::iqzip::{Sample, Samples};

/// Number of bins
const BINS: usize = 2048;

fn main() -> Result<()> {
    let path = env::args_os().skip(1).next().unwrap_or_else(|| {
        eprintln!("Usage: averages compressed-file-or-pipe");
        process::exit(-1)
    });
    // Get average samples
    let samples = Samples::new(BufReader::new(File::open(path)?))
        .map(Result::unwrap)
        .filter_map(Sample::into_average);

    // Most recent average for each of the 2048 bins
    let mut bin_averages = [0u32; BINS];

    let mut i = 0u16;
    for sample in samples {
        if let Some(entry) = bin_averages.get_mut(usize::from(sample.index)) {
            *entry = sample.magnitude;
        }

        if i % 2048 == 0 {
            // Update output
            eprintln!("First 50 bins: {:?}", &bin_averages[0..50]);
        }
        i = i.wrapping_add(1);
    }

    Ok(())
}
