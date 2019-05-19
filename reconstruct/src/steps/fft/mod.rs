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

//! The inverse FFT step

mod hanning;

use self::hanning::HANNING_2048;
use crate::window::{Status, TimeWindow, Window};
use crate::NATIVE_FFT_SIZE;

use fftw::array::AlignedVec;
use fftw::plan::{C2CPlan, C2CPlan32};
use fftw::types::{Flag, Sign};
use num_complex::Complex32;

/// An iterator adapter that uses an inverse FFT to convert frequency-domain windows
/// into time-domain windows
pub struct Fft<I> {
    /// Iterator that yields Windows
    inner: I,
    /// FFT implementation
    fft: C2CPlan32,
    /// Aligned FFT input
    fft_input: AlignedVec<Complex32>,
    /// Aligned FFT output
    fft_output: AlignedVec<Complex32>,
    /// Scaling factor to apply to time-domain samples
    scale: f32,
}

impl<I> Fft<I> {
    /// Creates a new FFT step
    pub fn new(inner: I, fft_size: usize) -> Self {
        let fft = C2CPlan32::aligned(&[fft_size], Sign::Backward, Flag::Measure)
            .expect("Failed to create FFT");
        let decimation = NATIVE_FFT_SIZE as usize / fft_size;
        info!("Decimation {}", decimation);
        // Select every decimation-th element for the Hanning window and sum
        let window_sum = HANNING_2048
            .iter()
            .cloned()
            .step_by(decimation)
            .sum::<f32>();
        info!("Window sum {}", window_sum);
        let hop = usize::from(NATIVE_FFT_SIZE) / 2 / decimation;
        let scale = hop as f32 / window_sum / (decimation as f32 * fft_size as f32);
        info!("Scale {}", scale);

        Fft {
            inner,
            fft,
            fft_input: AlignedVec::new(fft_size),
            fft_output: AlignedVec::new(fft_size),
            scale,
        }
    }
}

impl<I> Iterator for Fft<I>
where
    I: Iterator<Item = Status<Window>>,
{
    type Item = Status<TimeWindow>;
    fn next(&mut self) -> Option<Self::Item> {
        let window: Window = try_status!(self.inner.next());
        debug_assert_eq!(
            window.bins().len(),
            self.fft_input.len(),
            "Incorrect window size"
        );

        // Copy bins into aligned input
        self.fft_input.copy_from_slice(window.bins());

        // Run FFT with scratch as output
        self.fft
            .c2c(&mut self.fft_input, &mut self.fft_output)
            .expect("FFT failed");

        // Scale outputs
        for output_value in self.fft_output.iter_mut() {
            *output_value *= self.scale;
        }
        // Convert to time domain and copy FFT output there
        let mut time_window = window.into_time_domain();
        time_window.samples_mut().copy_from_slice(&self.fft_output);

        Some(Status::Ok(time_window))
    }
}
