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

mod hann;
#[cfg(feature = "fftw")]
mod impl_fftw;
#[cfg(not(feature = "fftw"))]
mod impl_rustfft;

use std::cmp;

use self::hann::HannWindow;
use crate::window::{Status, TimeWindow, Window};

#[cfg(feature = "fftw")]
type FftImpl = self::impl_fftw::FftwFft;
#[cfg(not(feature = "fftw"))]
type FftImpl = self::impl_rustfft::RustFftFft;

/// Uses an inverse FFT to convert frequency-domain windows
/// into time-domain windows
pub struct Fft {
    /// FFT implementation
    fft_impl: FftImpl,
    /// Scaling factor to apply to time-domain samples
    scale: f32,
}

impl Fft {
    /// Creates a new FFT step
    pub fn new(fft_size: usize, compression_fft_size: usize) -> Self {
        let fft_impl = FftImpl::new(fft_size);
        let decimation = compression_fft_size / fft_size;
        // Select every decimation-th element for the Hanning window and sum
        let window_sum = HannWindow::new(compression_fft_size)
            .step_by(decimation)
            .sum::<f32>();
        let hop = compression_fft_size / 2 / decimation;
        let scale = hop as f32 / window_sum / (decimation as f32 * fft_size as f32);

        Fft { fft_impl, scale }
    }

    /// Runs the FFT and scaling on multiple windows
    ///
    /// Returns the number of windows processed
    pub fn run(&mut self, windows_in: &mut [Window], windows_out: &mut [TimeWindow]) -> usize {
        let count = cmp::min(windows_in.len(), windows_out.len());
        let windows_in = &mut windows_in[..count];
        let windows_out = &mut windows_out[..count];
        // FFT
        for (window_in, window_out) in windows_in.iter_mut().zip(windows_out.iter_mut()) {
            self.fft_impl.run2(window_in, window_out);
            window_out.set_time(window_in.time());
        }
        // Scale
        for window in windows_out {
            for sample in window.samples_mut() {
                *sample *= self.scale;
            }
        }
        count
    }
}

/// An iterator adapter that uses an inverse FFT to convert frequency-domain windows
/// into time-domain windows
pub struct FftIter<I> {
    /// Iterator that yields Windows
    inner: I,
    /// FFT implementation
    fft: Fft,
}

impl<I> FftIter<I> {
    /// Creates a new FFT step
    pub fn new(inner: I, fft_size: usize, compression_fft_size: usize) -> Self {
        FftIter {
            inner,
            fft: Fft::new(fft_size, compression_fft_size),
        }
    }
}

impl<I> Iterator for FftIter<I>
where
    I: Iterator<Item = Status<Window>>,
{
    type Item = Status<TimeWindow>;
    fn next(&mut self) -> Option<Self::Item> {
        let window: Window = try_status!(self.inner.next());

        let mut time_window = self.fft.fft_impl.run(window);
        // Scale outputs
        for output_value in time_window.samples_mut() {
            *output_value *= self.fft.scale;
        }

        Some(Status::Ok(time_window))
    }
}
