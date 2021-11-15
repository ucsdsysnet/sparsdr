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

//! Parser for format version 2

use std::convert::TryInto;
use std::mem;

use crate::{ParseError, Parser, Window, WindowKind};
use num_complex::Complex;
use num_traits::Zero;

/// A compressed sample parser that supports format version 2 (for N210 or Pluto)
#[derive(Debug)]
pub struct V2Parser {
    /// Parsing state
    state: State,
    /// Number of bins in each FFT
    fft_size: u32,
}

impl Parser for V2Parser {
    fn sample_bytes(&self) -> usize {
        4
    }

    fn parse(&mut self, bytes: &[u8]) -> Result<Option<Window>, ParseError> {
        let bytes: [u8; 4] = bytes.try_into().expect("Wrong number of bytes");
        self.accept(u32::from_le_bytes(bytes))
    }
}

impl V2Parser {
    /// Creates a parser that will parse samples compressed with the provided FFT size
    pub fn new(fft_size: u32) -> Self {
        V2Parser {
            state: State::Idle,
            fft_size,
        }
    }

    /// Handles a 32-bit sample from the radio
    ///
    /// If a window is ready, it is returned.
    ///
    /// If an error occurs, this function returns an error and the parser returns to the idle
    /// state, where it is expecting a data header or an average header.
    fn accept(&mut self, sample: u32) -> Result<Option<Window>, ParseError> {
        let state = mem::replace(&mut self.state, State::Idle);
        log::trace!("Sample {:#010x} in state {:?}", sample, state);

        let return_value;
        self.state = match state {
            State::Idle => {
                let header = Header(sample);
                if header.is_valid() {
                    log::debug!("Got header with time {}", header.timestamp());
                    return_value = Ok(None);
                    state_for_header(header, self.fft_size)
                } else if sample == 0x0 {
                    // Spurious zero samples are not errors
                    return_value = Ok(None);
                    State::Idle
                } else {
                    log::error!(
                        "In state Initial, unexpected sample {:#x} (not a data or average header)",
                        sample
                    );
                    return_value = Err(ParseError(()));
                    State::Idle
                }
            }
            State::Average {
                timestamp,
                mut bins,
            } => {
                if bins.len() != self.fft_size as usize {
                    // Not all bins have been received, so this is an average value
                    bins.push(sample);

                    return_value = Ok(None);
                    State::Average { timestamp, bins }
                } else {
                    // All bins have been received, so this should be a zero that marks the end of
                    // the average window
                    if sample == 0 {
                        return_value = Ok(Some(Window {
                            timestamp,
                            kind: WindowKind::Average(bins),
                        }));
                        State::Idle
                    } else {
                        log::error!("In state Average, after receiving all {} averages, got a non-zero sample {:#x}", self.fft_size, sample);
                        return_value = Err(ParseError(()));
                        State::Idle
                    }
                }
            }
            State::Data {
                timestamp,
                mut bins,
                data_state: DataState::OutsideGroup,
            } => {
                // Not in a group of consecutive samples, so this sample is the bin index of the
                // next sample. If the most significant bit is set, this sample is instead the header
                // for the next window.
                let header = Header(sample);
                if header.is_valid() {
                    // Fill in zeros so that the length of bins equals the FFT size
                    let zero_count = self.fft_size as usize - bins.len();
                    for _ in 0..zero_count {
                        bins.push(Complex::zero());
                    }

                    return_value = Ok(Some(Window {
                        timestamp,
                        kind: WindowKind::Data(bins),
                    }));
                    state_for_header(header, self.fft_size)
                } else {
                    let index = sample;
                    if (index as usize) < bins.len() {
                        log::error!(
                            "In state Data, got a new group of values for index {}, but {} bins have already been received",
                            index,
                            bins.len()
                        );
                        return_value = Err(ParseError(()));
                        State::Idle
                    } else if index >= self.fft_size {
                        log::error!(
                            "In state Data, got a new group of values for index {}, which is too large for FFT size {}",
                            index,
                            self.fft_size
                        );
                        return_value = Err(ParseError(()));
                        State::Idle
                    } else {
                        // Fill in the bins with zeros as necessary so that bins.len() == index
                        let zero_count = index as usize - bins.len();
                        for _ in 0..zero_count {
                            bins.push(Complex::zero());
                        }

                        return_value = Ok(None);
                        State::Data {
                            timestamp,
                            bins,
                            data_state: DataState::InGroup,
                        }
                    }
                }
            }
            State::Data {
                timestamp,
                mut bins,
                data_state: DataState::InGroup,
            } => {
                if sample != 0 {
                    // New bin
                    if bins.len() == self.fft_size as usize {
                        log::warn!("In state Data, got a bin value when all bins in this window have already been received");
                        return_value = Err(ParseError(()));
                        State::Idle
                    } else {
                        // Extract real and imaginary parts of the sample
                        let complex_sample =
                            Complex::new((sample & 0xffff) as i16, (sample >> 16) as i16);
                        bins.push(complex_sample);

                        return_value = Ok(None);
                        State::Data {
                            timestamp,
                            bins,
                            data_state: DataState::InGroup,
                        }
                    }
                } else {
                    // Zero sample ends this group
                    return_value = Ok(None);
                    State::Data {
                        timestamp,
                        bins,
                        data_state: DataState::OutsideGroup,
                    }
                }
            }
        };
        return_value
    }
}

fn state_for_header(header: Header, fft_size: u32) -> State {
    let timestamp = header.timestamp();
    if header.is_fft_header() {
        // FFT
        State::Data {
            timestamp,
            bins: Vec::with_capacity(fft_size as usize),
            data_state: DataState::OutsideGroup,
        }
    } else {
        // Average
        State::Average {
            timestamp,
            bins: Vec::with_capacity(fft_size as usize),
        }
    }
}

/// Parser states
enum State {
    /// Waiting for the first sample
    Idle,
    /// Receiving a window of averages
    Average {
        /// The timestamp of these averages, in units of overlapped FFT intervals
        timestamp: u32,
        /// The average for each bin
        bins: Vec<u32>,
    },
    Data {
        timestamp: u32,
        bins: Vec<Complex<i16>>,
        data_state: DataState,
    },
}

/// Parser states while decoding data (FFT) windows
#[derive(Debug)]
enum DataState {
    OutsideGroup,
    InGroup,
}

/// A u32 wrapper that provides access to common header fields
struct Header(u32);

const TIMESTAMP_MASK: u32 = 0x3fff_ffff;

impl Header {
    pub fn is_valid(&self) -> bool {
        ((self.0 >> 31) & 1) == 1
    }
    pub fn is_fft_header(&self) -> bool {
        ((self.0 >> 30) & 1) == 0
    }
    pub fn timestamp(&self) -> u32 {
        // Clear bits 31 and 30
        self.0 & TIMESTAMP_MASK
    }
}

mod fmt_impl {
    use super::State;
    use std::fmt::{Debug, Formatter, Result};

    impl Debug for State {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                State::Idle => f.debug_struct("Idle").finish(),
                State::Average { timestamp, bins } => f
                    .debug_struct("Average")
                    .field("timestamp", timestamp)
                    .field("bin_count", &bins.len())
                    .finish(),
                State::Data {
                    timestamp,
                    bins,
                    data_state,
                } => f
                    .debug_struct("Data")
                    .field("timestamp", timestamp)
                    .field("bin_count", &bins.len())
                    .field("data_state", data_state)
                    .finish(),
            }
        }
    }
}
