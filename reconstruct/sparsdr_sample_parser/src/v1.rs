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

//! Version 1 parsers

use crate::{ParseError, Parser, Window, WindowKind};
use byteorder::{ByteOrder, LittleEndian};
use num_complex::Complex;

/// Length of a binary sample, bytes
const SAMPLE_LENGTH: usize = 8;

/// A parser that parses version 1 of the compressed sample format, as produced by a USRP N210
pub struct V1N210SampleParser {
    current_window: Option<Window>,
}

impl Parser for V1N210SampleParser {
    fn sample_bytes(&self) -> usize {
        SAMPLE_LENGTH
    }

    fn parse(&mut self, bytes: &[u8]) -> Result<Option<Window>, ParseError> {
        let sample = parse_one_sample(bytes);
        match self.current_window.take() {
            Some(Window { timestamp, kind }) => {
                if timestamp == sample.time() {
                    match (kind, sample) {
                        (WindowKind::Data(mut bins), Sample::Data(sample)) => {
                            let entry = bins
                                .get_mut(usize::from(sample.index))
                                .expect("Data bin index too large");
                            *entry = Complex::new(sample.real, sample.imag);
                            self.current_window = Some(Window {
                                timestamp,
                                kind: WindowKind::Data(bins),
                            });
                            Ok(None)
                        }
                        (WindowKind::Average(mut averages), Sample::Average(sample)) => {
                            let entry = averages
                                .get_mut(usize::from(sample.index))
                                .expect("Average bin index too large");
                            *entry = sample.magnitude;
                            self.current_window = Some(Window {
                                timestamp,
                                kind: WindowKind::Average(averages),
                            });
                            Ok(None)
                        }
                        (kind, sample) => {
                            // Have a data window, but got an average sample with the same timestamp
                            // or have an average window, but got a data sample with the same
                            // timestamp.
                            // This gets put into a new window with the same timestamp.
                            self.current_window = Some(new_window_with_sample(sample));
                            Ok(Some(Window { timestamp, kind }))
                        }
                    }
                } else {
                    // Start a new window
                    self.current_window = Some(new_window_with_sample(sample));
                    // Return the window collected from earlier samples
                    Ok(Some(Window { timestamp, kind }))
                }
            }
            None => {
                self.current_window = Some(new_window_with_sample(sample));
                Ok(None)
            }
        }
    }
}

fn new_window_with_sample(sample: Sample) -> Window {
    todo!()
}

fn parse_one_sample(bytes: &[u8]) -> Sample {
    assert_eq!(bytes.len(), SAMPLE_LENGTH, "Incorrect sample length");

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
        Sample::Average(AverageSample {
            time,
            index,
            magnitude,
        })
    } else {
        Sample::Data(DataSample {
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
enum Sample {
    /// A data sample
    Data(DataSample),
    /// An average sample
    Average(AverageSample),
}

impl Sample {
    /// Returns the index of this sample
    fn index(&self) -> u16 {
        match *self {
            Sample::Data(ref data) => data.index,
            Sample::Average(ref average) => average.index,
        }
    }
    /// Returns the time of this sample
    fn time(&self) -> u32 {
        match *self {
            Sample::Data(ref data) => data.time,
            Sample::Average(ref average) => average.time,
        }
    }
}

/// A data sample, containing signal information
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
struct DataSample {
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
struct AverageSample {
    /// The time of this sample, in units of 10 nanoseconds
    pub time: u32,
    /// The index in the FFT (0-2047) of this sample
    pub index: u16,
    /// The magnitude of the average signal at this index, in the same units as the threshold
    pub magnitude: u32,
}
