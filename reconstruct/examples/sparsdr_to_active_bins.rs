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
//! Reads a SparSDR compressed file and writes a text file containing active bins
//!

extern crate num_traits;
extern crate sparsdr_reconstruct;

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

use num_traits::Zero;
use sparsdr_reconstruct::input::iqzip::CompressedSamples;
use sparsdr_reconstruct::iter_ext::IterExt;
use sparsdr_reconstruct::window::{Logical, Window};

const NATIVE_FFT_SIZE: u16 = 2048;

fn main() {
    let mut args = env::args_os().skip(1);
    let in_path = args.next().expect("Expected input path");
    let out_path = args.next().expect("Expected output path");

    let in_file = File::open(in_path).expect("Failed to open input file");
    let mut out_file =
        BufWriter::new(File::create(out_path).expect("Failed to create output file"));

    let windows = CompressedSamples::new(in_file)
        .group(usize::from(NATIVE_FFT_SIZE))
        .shift_result(NATIVE_FFT_SIZE);

    for window in windows {
        let window: Window<Logical> = window.expect("Sample read failed");
        for bin in window.bins() {
            if bin.is_zero() {
                write!(out_file, "0").unwrap();
            } else {
                write!(out_file, "1").unwrap();
            }
        }
        writeln!(out_file).unwrap();
    }
}
