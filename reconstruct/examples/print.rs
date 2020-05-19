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
//! Reads a SparSDR compressed file and prints human-readable information, like
//! fpga_compress_with_avg_print.py
//!

extern crate sparsdr_reconstruct;

use std::io::{BufReader, Result};

use sparsdr_reconstruct::format::SampleFormat;
use sparsdr_reconstruct::input::iqzip::{Sample, Samples};
use sparsdr_reconstruct::steps::group::overflow::Overflow;

fn main() -> Result<()> {
    println!(
        "#{:9} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9}",
        "Sample", "Type", "FFT_No", "Index", "Time(ns)", "Real", "Imag"
    );

    let samples = Samples::new(BufReader::new(std::io::stdin()), SampleFormat::n210());

    let mut overflow = Overflow::new();

    for (i, sample) in samples.enumerate() {
        let sample = sample?;
        let i = i + 1;

        let time = overflow.expand(sample.time());
        //        let time = offset.remove_offset(time_overflow_corrected);
        let fft_no = sample.time() & 1;

        match sample {
            Sample::Data(sample) => {
                println!(
                    " {:<9}    FFT sample {:10}  {:10}  {:10}  {:10}  {:10}",
                    i, fft_no, sample.index, time, sample.real, sample.imag
                );
            }
            Sample::Average(sample) => {
                println!(
                    " {:<9}    Average                {:10}  {:10}      {:10}",
                    i, sample.index, time, sample.magnitude
                );
            }
        }
    }

    Ok(())
}
