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
//! An FFT implementation that uses rustfft
//!

use std::sync::Arc;

use num_complex::Complex32;
use num_traits::Zero;
use rustfft::{Fft, FftDirection, FftPlanner};

use crate::window::Fft as FftOrder;
use crate::window::{TimeWindow, Window};

/// rustfft FFT implementation
pub struct RustFftFft {
    /// FFT calculator
    fft: Arc<dyn Fft<f32>>,
    /// Scratch space used to hold the output of an FFT operation
    scratch: Vec<Complex32>,
}

impl RustFftFft {
    pub fn new(fft_size: usize) -> Self {
        let fft = FftPlanner::new().plan_fft(fft_size, FftDirection::Inverse);
        let scratch_length = fft.get_outofplace_scratch_len();
        RustFftFft {
            fft,
            scratch: vec![Complex32::zero(); scratch_length],
        }
    }

    /// Runs an out-of-place FFT from one window to another
    pub fn run(&mut self, source: &mut Window, destination: &mut TimeWindow) {
        self.fft.process_outofplace_with_scratch(
            source.bins_mut(),
            destination.samples_mut(),
            &mut self.scratch,
        );
    }
}
