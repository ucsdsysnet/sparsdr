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
pub mod n210;

use std::error::Error;

use num_complex::Complex32;

/// Trait for methods of reading compressed samples from a radio or file
pub trait ReadInput {
    /// Returns the sample rate used to receive the signals before compression, in samples/second
    ///
    /// This is also equal to the receive bandwidth in hertz.
    fn sample_rate(&self) -> f32;

    /// Returns the number of bins in the FFT used for compression
    fn bins(&self) -> u16;

    /// Prepares this input source to read samples. This will be called once, shortly before
    /// the first call to read_window().
    ///
    /// The default implementation does nothing.
    fn start(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Reads one or more samples from the source into a buffer
    ///
    /// This function may read fewer samples than the size of the buffer. If the source
    /// has reached an end-of-file condition and no more samples will be available, it should
    /// return Ok(0). Otherwise, it should block until at least one sample has been read.
    ///
    /// On success, this function returns the number of samples read.
    fn read_samples(&mut self, samples: &mut [Sample]) -> Result<usize, Box<dyn Error>>;
}

/// A compressed frequency-domain sample in a format compatible with SparSDR implementations
/// on all supported hardware
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Sample {
    /// A timestamp of the FFT that produced this sample
    ///
    /// The unit of this timestamp is the interval between time-domain samples. For example,
    /// SparSDR on the USRP N210 receives signals at 100 million samples per second, so the timestamp
    /// is in units of 10 nanoseconds.
    ///
    /// Consecutive samples with the same value are assumed to come from the same FFT. This value
    /// may overflow.
    pub time: u32,
    /// The index in the FFT of this sample
    pub index: u16,
    /// The amplitude of this sample
    pub amplitude: Complex32,
}
