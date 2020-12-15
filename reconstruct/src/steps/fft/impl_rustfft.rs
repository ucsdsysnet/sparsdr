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
use rustfft::{FFTplanner, FFT};

use crate::window::{Fft, TimeWindow, Window};

/// rustfft FFT implementation
pub struct RustFftFft {
    /// FFT calculator
    fft: Arc<dyn FFT<f32>>,
    /// Scratch space used to hold the output of an FFT operation
    scratch: Vec<Complex32>,
}

impl RustFftFft {
    pub fn new(fft_size: usize) -> Self {
        RustFftFft {
            fft: FFTplanner::new(true).plan_fft(fft_size),
            scratch: vec![Complex32::zero(); fft_size],
        }
    }
    pub fn run(&mut self, mut source: Window<Fft>) -> TimeWindow {
        self.fft.process(source.bins_mut(), &mut self.scratch);
        let mut time_window = source.into_time_domain();
        time_window.samples_mut().copy_from_slice(&self.scratch);
        time_window
    }
}
