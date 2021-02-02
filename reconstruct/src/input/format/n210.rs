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
//! USRP N210 compressed data format
//!

use crate::input::{ReadInput, Sample};
use byteorder::{ByteOrder, LittleEndian};
use num_complex::Complex32;
use std::error::Error;
use std::io::{self, ErrorKind, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Number of bytes used to represent a sample, in the format the USRP sends
pub const SAMPLE_BYTES: usize = 8;
/// FFT size used for compression
pub const BINS: u16 = 2048;
/// Compression sample rate
pub const SAMPLE_RATE: f32 = 100_000_000.0;

/// Reads N210-format samples from a file or other byte stream
pub struct N210SampleReader<R> {
    source: R,
    end_of_file: bool,
    stop: Arc<AtomicBool>,
}

impl<R> N210SampleReader<R>
where
    R: Read,
{
    /// Creates a sample reader
    pub fn new(source: R) -> Self {
        N210SampleReader {
            source,
            end_of_file: false,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    fn read_sample(&mut self) -> SampleStatus {
        let mut buffer = [0u8; SAMPLE_BYTES];
        // Read samples until a data sample is read
        loop {
            let status = self.source.read_exact(&mut buffer);
            log::debug!("read_exact returned {:?}", status);
            // Check errors
            if let Err(e) = status {
                break match e.kind() {
                    // End-of-file (for files) or broken pipe (for pipes) is not considered an error
                    ErrorKind::UnexpectedEof | ErrorKind::BrokenPipe | ErrorKind::Interrupted => {
                        SampleStatus::EndOfFile
                    }
                    _ => SampleStatus::Err(e.into()),
                };
            } else {
                if let N210Sample::Data(data_sample) = parse_sample(&buffer) {
                    break SampleStatus::Ok(data_sample.into());
                }
            }
            if self.stop.load(Ordering::Relaxed) {
                break SampleStatus::EndOfFile;
            }
        }
    }
}

#[derive(Debug)]
enum SampleStatus {
    Ok(Sample),
    Err(io::Error),
    EndOfFile,
}

impl<R> ReadInput for N210SampleReader<R>
where
    R: Read,
{
    fn sample_rate(&self) -> f32 {
        SAMPLE_RATE
    }

    fn bins(&self) -> u16 {
        BINS
    }

    fn set_stop_flag(&mut self, stop: Arc<AtomicBool>) {
        self.stop = stop;
    }

    fn read_samples(&mut self, samples: &mut [Sample]) -> Result<usize, Box<dyn Error>> {
        if self.end_of_file {
            Ok(0)
        } else {
            let mut samples_read = 0;
            for sample in samples {
                match self.read_sample() {
                    SampleStatus::Ok(sample_read) => *sample = sample_read,
                    SampleStatus::Err(e) => return Err(e.into()),
                    SampleStatus::EndOfFile => {
                        self.end_of_file = true;
                        break;
                    }
                };
                samples_read += 1;
            }
            Ok(samples_read)
        }
    }
}

/// Parses 8 bytes into a sample
pub fn parse_sample(bytes: &[u8; SAMPLE_BYTES]) -> N210Sample {
    // Little-endian
    type E = LittleEndian;
    let fft_index = E::read_u16(&bytes[0..2]);
    let time = E::read_u16(&bytes[2..4]);
    // Last 4 bytes may be either real/imaginary signal or average magnitude
    let magnitude = {
        // Magnitude is in two 2-byte chunks. Bytes within each chunk are little endian,
        // but the more significant chunk is first.
        let more_significant = E::read_u16(&bytes[4..6]);
        let less_significant = E::read_u16(&bytes[6..8]);
        u32::from(more_significant) << 16 | u32::from(less_significant)
    };
    let real = E::read_i16(&bytes[4..6]);
    let imag = E::read_i16(&bytes[6..8]);

    let is_average = ((fft_index >> 15) & 1) == 1;
    let index = (fft_index >> 4) & 0x7ff;
    // Reassemble time from MSBs and other bits
    let time = u32::from(time) | (u32::from(fft_index & 0xF) << 16);

    if is_average {
        N210Sample::Average(AverageSample {
            time,
            index,
            magnitude,
        })
    } else {
        N210Sample::Data(DataSample {
            time,
            index,
            real,
            imag,
        })
    }
}

/// A data or average sample
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub enum N210Sample {
    /// A data sample
    Data(DataSample),
    /// An average sample
    Average(AverageSample),
}

impl N210Sample {
    /// Returns a DataSample if this is a data sample, or None otherwise
    pub fn into_data(self) -> Option<DataSample> {
        match self {
            N210Sample::Data(data) => Some(data),
            N210Sample::Average(_) => None,
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

impl DataSample {
    /// Returns the amplitude of this sample as a floating-point complex number
    pub fn complex_amplitude(&self) -> Complex32 {
        Complex32 {
            re: to_float(self.real),
            im: to_float(self.imag),
        }
    }
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

/// Converts a 16-bit value into a float
///
/// -32767 maps to approximately -1.0 and 32768 maps to approximately 1.0 .
fn to_float(value: i16) -> f32 {
    const MAX_MAGNITUDE: f32 = 32768.0;
    f32::from(value) / MAX_MAGNITUDE
}

/// Conversion from an IQZip data sample into a standard sample
impl From<DataSample> for Sample {
    fn from(data_sample: DataSample) -> Self {
        Sample {
            time: data_sample.time,
            index: data_sample.index,
            amplitude: data_sample.complex_amplitude(),
        }
    }
}
