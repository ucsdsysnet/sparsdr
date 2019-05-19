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
//! Reads complex amplitude values in 2048-bin chunks from a file and writes them to another file
//! in IQZip format
//!
//! The input file should have the format expected by the `decompress::matlab` module.
//!

extern crate num_complex;
extern crate num_traits;
extern crate sparsdr_reconstruct;

use num_complex::Complex32;
use num_traits::Zero;

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter, Result, Seek, SeekFrom};
use std::process;

use sparsdr_reconstruct::input::iqzip::{write_samples, DataSample, Sample};
use sparsdr_reconstruct::input::matlab::Samples;

// Mask time to 24 bits
const TIME_MASK: u32 = 0xffffff;

fn main() {
    let mut args = env::args_os().skip(1);
    let in_path = args.next().unwrap_or_else(|| print_usage_and_exit());
    let out_path = args.next().unwrap_or_else(|| print_usage_and_exit());

    match run(&in_path, &out_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            process::exit(-1);
        }
    }
}

fn run(in_path: &OsStr, out_path: &OsStr) -> Result<()> {
    let mut in_file = BufReader::new(File::open(in_path)?);
    let out_file = BufWriter::new(File::create(out_path)?);

    // First run: Collect maximum value
    let max_value = Samples::new(&mut in_file)
        .map(|read_result| read_result.expect("Sample read failed"))
        .map(|sample| {
            // Check no NaNs
            assert!(
                !sample.amplitude.re.is_nan(),
                "NaN in real part of amplitude"
            );
            assert!(
                !sample.amplitude.im.is_nan(),
                "NaN in imaginary part of amplitude"
            );
            // Return larger of real and imaginary
            if sample.amplitude.im > sample.amplitude.re {
                sample.amplitude.im
            } else {
                sample.amplitude.re
            }
        })
        .fold(
            0.0f32,
            |max, sample_max| {
                if sample_max > max {
                    sample_max
                } else {
                    max
                }
            },
        );
    // Scale down if any value is greater than 1
    let scale = if max_value > 1.0 {
        1.0 / max_value
    } else {
        1.0
    };

    // Seek back to the beginning for the real conversion
    in_file.seek(SeekFrom::Start(0))?;

    // Filter out samples with zero amplitude, and convert to IQZip
    let filtered_samples = Samples::new(in_file)
        .map(|read_result| read_result.expect("Sample read failed"))
        .filter(|sample| sample.amplitude != Complex32::zero())
        .map(|sample| {
            Sample::Data(DataSample {
                time: sample.time & TIME_MASK,
                index: sample.index,
                real: amplitude_to_integer(sample.amplitude.re * scale),
                imag: amplitude_to_integer(sample.amplitude.im * scale),
            })
        });
    write_samples(filtered_samples, out_file)?;

    Ok(())
}

fn amplitude_to_integer(amplitude: f32) -> i16 {
    if amplitude.abs() > 1.0 {
        eprintln!("Warning: Amplitude {} outside expected range", amplitude);
    }
    (amplitude * 32767.0) as i16
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: matlab_to_iqz input-path output-path");
    process::exit(-1);
}
