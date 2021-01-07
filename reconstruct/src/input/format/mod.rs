/*
 * Copyright 2021 The Regents of the University of California
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
//! Sample formats
//!

use std::error::Error;

pub mod n210;
pub mod pluto;

/// Things that can parse individual samples
pub trait SampleParser {
    /// Size of FFT (number of bins) used for compression
    const BINS: u16;
    /// Sample rate used to capture samples for compression
    const SAMPLE_RATE: f32;
    /// Number of bytes needed to parse a sample
    const SAMPLE_BYTES: usize;
    /// Type of sample that can be parsed
    type FormatSample;
    /// Parsing error type
    type Error: Error;

    /// Attempts to parse a sample from some bytes
    ///
    /// This function may panic if bytes.len() is not equal to Self::SAMPLE_BYTES.
    fn parse_sample(&mut self, bytes: &[u8]) -> Result<Self::FormatSample, Self::Error>;
}
