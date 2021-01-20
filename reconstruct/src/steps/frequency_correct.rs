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

//! The frequency correction step

use std::f32::consts::PI;

use num_complex::Complex32;
use num_traits::One;

use crate::window::TimeWindow;

//
// Details of the optimization used to calculate multiplication values:
//
// In the basic implementation, every value in each bin is multiplied by
// e^(i * 2 * pi * (-bin_offset / fft_size) * sample_index).
// When sample_index is 0, this is 1.
// Every time sample_index increases, this value gets multiplied by
// e^(i * 2 * pi * (-bin_offset / fft_size)).
//
// By starting at 1 and multiplying by e^(i * 2 * pi * (-bin_offset / fft_size)) for each new
// sample, the per-sample work decreases from a complex multiply, a complex exponent, and a second
// complex multiply to two complex multiplications.
//

/// Applies a frequency correction to time-domain samples
pub struct FrequencyCorrect {
    /// e^(i * 2 * pi * (-bin_offset / fft_size))
    ///
    /// For each new sample, correction gets multiplied by this
    correction_base: Complex32,
    /// The correction to apply to the next sample
    ///
    /// After each sample is processed, this correction gets multiplied by correction_base.
    correction: Complex32,
}

impl FrequencyCorrect {
    /// Creates a frequency corrector
    pub fn new(bin_offset: f32, fft_size: u16) -> Self {
        let correction_real_base = -bin_offset / f32::from(fft_size);

        let correction_base = Complex32::exp(Complex32::i() * 2.0 * PI * correction_real_base);

        FrequencyCorrect {
            correction_base,
            correction: Complex32::one(),
        }
    }
    /// Applies the frequency correction to a sample
    fn correct_sample(&mut self, sample: &mut Complex32) {
        *sample *= self.correction;
        // Update correction for next sample
        self.correction *= self.correction_base;
    }

    /// Applies the frequency correction to each sample in a slice
    pub fn correct_samples(&mut self, samples: &mut [Complex32]) {
        for sample in samples {
            self.correct_sample(sample);
        }
    }
    /// Applies the frequency correction to each sample in each window
    pub fn correct_windows(&mut self, windows: &mut [TimeWindow]) {
        for window in windows {
            self.correct_samples(window.samples_mut());
        }
    }
}
