/*
 * Copyright 2021 The Regents of the University of California
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

//! Reads averages from a compressed file and prints the average of averages for each bin

extern crate sparsdr_reconstruct;

use std::env;
use std::fs::File;
use std::io::BufReader;

use sparsdr_reconstruct::input::iqzip::{Sample, Samples};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args_os().nth(1).expect("No path");
    let file = BufReader::new(File::open(path)?);

    const EMPTY_VEC: Vec<u32> = Vec::new();
    let mut averages: [Vec<u32>; 1024] = [EMPTY_VEC; 1024];

    for sample in Samples::new(file) {
        let sample = sample?;
        match sample {
            Sample::Data(_) => {}
            Sample::Average(average_sample) => {
                averages[usize::from(average_sample.index)].push(average_sample.magnitude);
            }
        }
    }

    let averages_of_averages: Vec<f64> = averages
        .iter()
        .map(|bin_averages| {
            assert!(!bin_averages.is_empty(), "No averages for bin");
            bin_averages.iter().copied().sum::<u32>() as f64 / bin_averages.len() as f64
        })
        .collect();

    // Code
    println!("constexpr std::uint32_t BIN_AVERAGES[1024] = {{");
    for average in averages_of_averages.iter().copied() {
        println!("    {},", average.ceil());
    }
    println!("}};");

    Ok(())
}
