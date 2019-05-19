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
//! Parsers for reading sample input in various formats
//!

pub mod iqzip;
pub mod matlab;

use num_complex::Complex32;

/// A compressed frequency-domain sample
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Sample {
    /// The time of this sample, in units of 10 nanoseconds
    pub time: u32,
    /// The index in the FFT (0-2047) of this sample
    pub index: u16,
    /// The amplitude of this sample
    pub amplitude: Complex32,
}

impl Sample {
    /// Returns true if this sample should be decompressed with FFT 0 instead of FFT 1
    pub fn is_fft_0(&self) -> bool {
        // Moein knows why this works
        u32::from((self.index >> 10) & 1) == self.time & 1
    }
}
