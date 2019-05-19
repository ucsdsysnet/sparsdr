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
//! Reads compressed samples from a file, sorts them, and writes them to another file
//!
//! Usage: sort input-file output-file
//!

const USAGE: &str = "Usage: sort input-file [output-file]";

extern crate sparsdr_reconstruct;

use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result};
use std::process;

use sparsdr_reconstruct::input::iqzip::{write_samples, DataSample, Sample, Samples};
use std::cmp::Ordering;

fn run() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let source_path = args.next().unwrap_or_else(|| {
        eprintln!("{}", USAGE);
        process::exit(-1);
    });
    let destination_path = args.next();

    let source_file = BufReader::new(File::open(source_path)?);
    let destination_file = if let Some(destination_path) = destination_path {
        Some(BufWriter::new(File::create(destination_path)?))
    } else {
        None
    };

    let mut samples: Vec<DataSample> = Samples::new(source_file)
        .filter_map(result_into_data)
        .collect::<Result<Vec<_>>>()?;
    // Check for out-of-order or overflow
    // Fold: state is (previous sample time, index of first overflow sample)
    let (_, first_overflow) = samples.iter().enumerate().fold(
        (0u32, samples.len()),
        |(previous_time, mut first_overflow), (i, sample)| {
            if sample.time < previous_time {
                let difference = previous_time - sample.time;
                if difference > 8 {
                    eprintln!(
                        "Overflow: {} to {}, difference {}",
                        previous_time, sample.time, difference
                    );
                    // Update overflow index
                    if i < first_overflow {
                        first_overflow = i;
                    }
                } else {
                    eprintln!(
                        "Out-of-order: {} to {}, difference {}",
                        previous_time, sample.time, difference
                    );
                }
            }
            (sample.time, first_overflow)
        },
    );

    if samples.len() != first_overflow {
        eprintln!(
            "Trimming from {} to {} samples before first overflow",
            samples.len(),
            first_overflow
        );
    }
    let samples = &mut samples[..first_overflow];

    if let Some(destination_file) = destination_file {
        samples.sort_unstable_by(compare_time_and_index);

        // Copy the DataSamples into a new vec of Samples
        let samples = samples.iter().cloned().map(Sample::Data);
        write_samples(samples, destination_file)?;
    }
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => eprintln!("{}", e),
    }
}

fn compare_time_and_index(sample1: &DataSample, sample2: &DataSample) -> Ordering {
    // First try by time
    sample1.time.cmp(&sample2.time).then_with(|| {
        // If times are equal, compare by index / bin
        sample1.index.cmp(&sample2.index)
    })
}

fn result_into_data(sample: Result<Sample>) -> Option<Result<DataSample>> {
    match sample {
        Ok(Sample::Data(data)) => Some(Ok(data)),
        Ok(Sample::Average(_)) => None,
        Err(e) => Some(Err(e)),
    }
}
