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
#[cfg(feature = "fftw")]
mod impl_fftw;
#[cfg(not(feature = "fftw"))]
mod impl_rustfft;

use self::hanning::HANNING_2048;
use crate::window::{Status, TimeWindow, Window};

#[cfg(feature = "fftw")]
type FftImpl = self::impl_fftw::FftwFft;
#[cfg(not(feature = "fftw"))]
type FftImpl = self::impl_rustfft::RustFftFft;

/// An iterator adapter that uses an inverse FFT to convert frequency-domain windows
/// into time-domain windows
pub struct Fft<I> {
    /// Iterator that yields Windows
    inner: I,
    /// FFT implementation
    fft_impl: FftImpl,
    /// Scaling factor to apply to time-domain samples
    scale: f32,
}

impl<I> Fft<I> {
    /// Creates a new FFT step
    pub fn new(inner: I, fft_size: usize, compression_fft_size: usize) -> Self {
        let fft_impl = FftImpl::new(fft_size);
        let decimation = compression_fft_size / fft_size;
        info!("Decimation {}", decimation);
        // Select every decimation-th element for the Hanning window and sum
        let window_sum = HANNING_2048
            .iter()
            .cloned()
            .step_by(decimation)
            .sum::<f32>();
        info!("Window sum {}", window_sum);
        let hop = compression_fft_size / 2 / decimation;
        let scale = hop as f32 / window_sum / (decimation as f32 * fft_size as f32);
        info!("Scale {}", scale);

        Fft {
            inner,
            fft_impl,
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

        let mut time_window = self.fft_impl.run(window);
        // Scale outputs
        for output_value in time_window.samples_mut() {
            *output_value *= self.scale;
        }

        Some(Status::Ok(time_window))
    }
}
