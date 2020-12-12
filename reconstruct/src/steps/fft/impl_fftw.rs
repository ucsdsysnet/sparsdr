/*
 * Copyright 2020 The Regents of the University of California
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
//! An FFT implementation that uses the FFTW library
//!

use fftw::array::AlignedVec;
use fftw::plan::{C2CPlan, C2CPlan32};
use num_complex::Complex32;

use crate::window::{TimeWindow, Window};
use fftw::types::{Flag, Sign};

/// An FFT implementation using FFTW
pub struct FftwFft {
    /// FFT implementation
    fft: C2CPlan32,
    /// Aligned FFT input
    fft_input: AlignedVec<Complex32>,
    /// Aligned FFT output
    fft_output: AlignedVec<Complex32>,
}

impl FftwFft {
    pub fn new(fft_size: usize) -> Self {
        let fft = C2CPlan32::aligned(&[fft_size], Sign::Backward, Flag::MEASURE)
            .expect("FFT setup failed");
        FftwFft {
            fft,
            fft_input: AlignedVec::new(fft_size),
            fft_output: AlignedVec::new(fft_size),
        }
    }
    pub fn run(&mut self, source: Window) -> TimeWindow {
        // Copy bins into aligned input
        self.fft_input.copy_from_slice(source.bins());

        // Run FFT with scratch as output
        self.fft
            .c2c(&mut self.fft_input, &mut self.fft_output)
            .expect("FFT failed");
        // Copy output
        let mut output = source.into_time_domain();
        output.samples_mut().copy_from_slice(&self.fft_output);
        output
    }
}
