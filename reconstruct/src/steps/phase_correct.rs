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

//! The phase correction step

use std::f32::consts::PI;

use num_complex::Complex32;
use num_traits::One;

use crate::window::{Status, Window};

//
// Details of the optimization used to calculate multiplication values:
//
// Every value in each bin is multiplied by e^(i * pi * fc_bins * window_index).
// When window_index is 0, this value is 1.
// For each new window, this value gets multiplied by e^(i * pi * fc_bins).
//

/// Applies phase correction to frequency-domain windows
struct PhaseCorrect {
    /// e^(i * pi * fc_bins)
    ///
    /// For each new window, correction gets multiplied by this value
    correction_base: Complex32,
    /// The correction to apply to the next window
    correction: Complex32,
}

impl PhaseCorrect {
    /// Creates a phase corrector
    ///
    /// fc_bins: The whole-number part of the frequency offset in bins
    pub fn new(fc_bins: f32) -> Self {
        let correction_base = Complex32::exp(&(Complex32::i() * PI * fc_bins));
        PhaseCorrect {
            correction_base,
            correction: Complex32::one(),
        }
    }
}

impl PhaseCorrect {
    /// Applies the correction to a window
    pub fn correct_window(&mut self, window: &mut Window) {
        for bin in window.bins_mut() {
            *bin *= self.correction;
        }
        // Update correction for next window
        self.correction *= self.correction_base;
    }
}

/// An iterator adapter that applies a phase correction to frequency-domain windows
pub struct PhaseCorrectIter<I> {
    /// Inner iterator of Windows
    inner: I,
    /// Phase correction
    corrector: PhaseCorrect,
}

impl<I> PhaseCorrectIter<I> {
    /// Creates a phase corrector
    ///
    /// fc_bins: The whole-number part of the frequency offset in bins
    pub fn new(inner: I, fc_bins: f32) -> Self {
        PhaseCorrectIter {
            inner,
            corrector: PhaseCorrect::new(fc_bins),
        }
    }
}

impl<I> Iterator for PhaseCorrectIter<I>
where
    I: Iterator<Item = Status<Window>>,
{
    type Item = Status<Window>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut window: Window = try_status!(self.inner.next());

        self.corrector.correct_window(&mut window);

        Some(Status::Ok(window))
    }
}
