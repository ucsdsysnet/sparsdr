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

extern crate fftw;
extern crate num_complex;

use num_complex::Complex32;

use fftw::array::AlignedVec;
use fftw::plan::*;
use fftw::types::*;

const FFT_SIZE: usize = 2048;

fn main() {
    let input = [Complex32::new(0.5, 0.0); FFT_SIZE];

    let mut input_scratch = AlignedVec::new(FFT_SIZE);
    input_scratch.copy_from_slice(&input);

    let mut output = AlignedVec::<Complex32>::new(FFT_SIZE);

    let mut fft = C2CPlan32::aligned(&[FFT_SIZE], Sign::Backward, Flag::MEASURE)
        .expect("Failed to create FFT");

    fft.c2c(&mut input_scratch, &mut output)
        .expect("FFT failed");

    // Scale
    for output_value in output.iter_mut() {
        *output_value /= FFT_SIZE as f32;
    }

    println!("Input: ");
    print_values(&input);
    println!("Output: ");
    print_values(&output);
}

fn print_values(values: &[Complex32]) {
    for value in values {
        print!("{} ", *value);
    }
    println!()
}
