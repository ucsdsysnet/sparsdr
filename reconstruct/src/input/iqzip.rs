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
//! IQZip compressed data reading and writing
//!

use std::io::{self, Read, Result};
use std::iter::FilterMap;

use num_complex::Complex32;

use crate::blocking::BlockLogger;
use crate::format::SampleFormat;

mod compressed {
    pub use crate::input::Sample;
}

/// A data or average sample
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum Sample {
    /// A data sample
    Data(DataSample),
    /// An average sample
    Average(AverageSample),
}

impl Sample {
    /// Returns a DataSample if this is a data sample, or None otherwise
    pub fn into_data(self) -> Option<DataSample> {
        match self {
            Sample::Data(data) => Some(data),
            Sample::Average(_) => None,
        }
    }
    /// Returns an AverageSample if this is an average sample, or None otherwise
    pub fn into_average(self) -> Option<AverageSample> {
        match self {
            Sample::Data(_) => None,
            Sample::Average(average) => Some(average),
        }
    }
    /// Returns true if this is an average sample
    pub fn is_average(&self) -> bool {
        match *self {
            Sample::Data(_) => false,
            Sample::Average(_) => true,
        }
    }
    /// Returns the index of this sample
    pub fn index(&self) -> u16 {
        match *self {
            Sample::Data(ref data) => data.index,
            Sample::Average(ref average) => average.index,
        }
    }
    /// Returns the time of this sample
    pub fn time(&self) -> u32 {
        match *self {
            Sample::Data(ref data) => data.time,
            Sample::Average(ref average) => average.time,
        }
    }
}

/// A data sample, containing signal information
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct DataSample {
    /// The time of this sample, in units of 10 nanoseconds
    pub time: u32,
    /// The index in the FFT (0-2047) of this sample
    pub index: u16,
    /// The real part of the amplitude, as a 16-bit signed integer
    pub real: i16,
    /// The imaginary part of the amplitude, as a 16-bit signed integer
    pub imag: i16,
}

/// An average sample
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct AverageSample {
    /// The time of this sample, in units of 10 nanoseconds
    pub time: u32,
    /// The index in the FFT (0-2047) of this sample
    pub index: u16,
    /// The magnitude of the average signal at this index, in the same units as the threshold
    pub magnitude: u32,
}

impl DataSample {
    /// Returns the amplitude of this sample as a complex number
    pub fn complex_amplitude(&self) -> Complex32 {
        Complex32 {
            re: to_float(self.real),
            im: to_float(self.imag),
        }
    }
}

/// An iterator that reads IQZip samples from a byte source
#[derive(Debug)]
pub struct Samples<'b, R> {
    /// The byte source
    source: R,
    /// The blocking logger, if provided
    block_logger: Option<&'b BlockLogger>,
    /// The format to use to read samples
    format: SampleFormat,
}

impl<'b, R> Samples<'b, R>
where
    R: Read,
{
    /// Creates a sample reader that will read bytes from the provided source
    pub fn new(source: R, format: SampleFormat) -> Self {
        Samples {
            source,
            block_logger: None,
            format,
        }
    }

    /// Sets the block logger to record time spent blocking
    pub fn set_block_logger(&mut self, block_logger: &'b BlockLogger) {
        self.block_logger = Some(block_logger)
    }

    /// Reads from self.source and records the time spent blocking
    fn log_read_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
        if let Some(ref block_logger) = self.block_logger {
            let source = &mut self.source;
            block_logger.log_blocking(|| source.read_exact(buffer))
        } else {
            self.source.read_exact(buffer)
        }
    }
}

impl<'b, R> Iterator for Samples<'b, R>
where
    R: Read,
{
    type Item = Result<Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; 8];
        match self.log_read_exact(&mut buffer) {
            Ok(_) => Some(Ok(self.format.parse_sample(&buffer))),
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    // End of stream
                    None
                } else {
                    // Error can't be handled here
                    warn!("Error in read_exact for sample: {}", e);
                    Some(Err(e))
                }
            }
        }
    }
}

/// A FilterMap that converts a Result<Sample> into an Option<Result<Sample>> using a function
/// pointer
type FnFilterMap<'b, R> =
    FilterMap<Samples<'b, R>, fn(Result<Sample>) -> Option<Result<compressed::Sample>>>;

/// An iterator that reads IQZip samples from a byte source and converts them to standard
/// `compressed::Sample` values
#[derive(Debug)]
pub struct CompressedSamples<'b, R>(FnFilterMap<'b, R>);

impl<'b, R> CompressedSamples<'b, R>
where
    R: Read,
{
    /// Creates an iterator that reads compressed samples from a file
    pub fn new(source: R, format: SampleFormat) -> Self {
        CompressedSamples(Samples::new(source, format).filter_map(filter_map_compressed_sample))
    }

    /// Creates an iterator that reads compressed samples from a file and logs blocking operations
    pub fn with_block_logger(
        source: R,
        block_logger: &'b BlockLogger,
        format: SampleFormat,
    ) -> Self {
        let mut samples = Samples::new(source, format);
        samples.set_block_logger(block_logger);
        CompressedSamples(samples.filter_map(filter_map_compressed_sample))
    }
}

impl<'b, R> Iterator for CompressedSamples<'b, R>
where
    R: Read,
{
    type Item = Result<compressed::Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Converts DataSamples to compressed::Samples, discards AverageSamples, and passes errors through
fn filter_map_compressed_sample(sample: Result<Sample>) -> Option<Result<compressed::Sample>> {
    match sample {
        Ok(sample) => match sample.into_data() {
            Some(data_sample) => Some(Ok(compressed::Sample::from(data_sample))),
            None => None,
        },
        Err(e) => Some(Err(e)),
    }
}

/// Converts a 16-bit value into a float
///
/// -32767 maps to approximately -1.0 and 32768 maps to approximately 1.0 .
fn to_float(value: i16) -> f32 {
    const MAX_MAGNITUDE: f32 = 32768.0;
    f32::from(value) / MAX_MAGNITUDE
}

/// Conversion from an IQZip data sample into a standard sample
impl From<DataSample> for compressed::Sample {
    fn from(iqzip_sample: DataSample) -> Self {
        compressed::Sample {
            time: iqzip_sample.time,
            index: iqzip_sample.index,
            amplitude: iqzip_sample.complex_amplitude(),
        }
    }
}
