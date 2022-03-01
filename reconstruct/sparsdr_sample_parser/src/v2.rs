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
    /// The expected sequence number of the next data window
    expected_sequence: Option<WindowSequence>,
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
            expected_sequence: None,
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
                if sample == 0x0 {
                    // Got zero, advance and check for the header
                    return_value = Ok(None);
                    State::Zero
                } else {
                    // No change
                    return_value = Ok(None);
                    State::Idle
                }
            }
            State::Zero => {
                let header = Header(sample);
                if header.is_valid() {
                    log::debug!("Got header with time {}", header.timestamp());
                    return_value = Ok(None);
                    state_for_header(header, self.fft_size)
                } else {
                    // Still in some partially-cut-off window, go back to idle
                    return_value = Ok(None);
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
                        log::trace!("End of averages, returning window with time {}", timestamp);
                        return_value = Ok(Some(Window {
                            timestamp,
                            kind: WindowKind::Average(bins),
                        }));
                        State::Zero
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

                    self.increment_expected_sequence();
                    log::trace!("Got header, returning window with time {}", timestamp);
                    return_value = Ok(Some(Window {
                        timestamp,
                        kind: WindowKind::Data(bins),
                    }));
                    state_for_header(header, self.fft_size)
                } else {
                    let group_header = BinGroupHeader(sample);

                    // If there is no expected sequence number stored and this window has a sequence
                    // number, use the sequence number from this window
                    if let (None, Some(sequence)) =
                        (self.expected_sequence, group_header.sequence())
                    {
                        self.expected_sequence = Some(sequence);
                    }

                    match self.check_sequence(group_header.sequence()) {
                        Ok(()) => {
                            // Sequence number OK or not expected
                            let index = group_header.bin();
                            if usize::from(index) < bins.len() {
                                log::error!(
                                    "In state Data, got a new group of values for \
                                    index {}, but {} bins have already been received",
                                    index,
                                    bins.len()
                                );
                                return_value = Err(ParseError(()));
                                State::Idle
                            } else if u32::from(index) >= self.fft_size {
                                log::error!(
                                    "In state Data, got a new group of values for \
                                    index {}, which is too large for FFT size {}",
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
                        Err(e) => {
                            // Sequence number mismatch -> go back to idle state and wait for a zero
                            return_value = Err(e);
                            State::Idle
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
                        log::warn!(
                            "In state Data, got a bin value when all bins in this \
                            window have already been received"
                        );
                        return_value = Err(ParseError(()));
                        State::Idle
                    } else {
                        // Extract real and imaginary parts of the sample
                        let complex_sample =
                            Complex::new((sample >> 16) as i16, (sample & 0xffff) as i16);
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

    /// If self.expected_sequence and sequence are both Some, this function checks that they
    /// are equal. If they are not equal, this function returns an error.
    fn check_sequence(&mut self, sequence: Option<WindowSequence>) -> Result<(), ParseError> {
        match (self.expected_sequence, sequence) {
            (Some(expected_sequence), Some(sequence)) => {
                if expected_sequence == sequence {
                    Ok(())
                } else {
                    log::error!(
                        "Sequence number mismatch: expected {}, got {}",
                        expected_sequence,
                        sequence
                    );
                    // Accept any sequence number for the next window
                    self.expected_sequence = None;
                    Err(ParseError(()))
                }
            }
            _ => Ok(()),
        }
    }

    /// If self.expected_sequence is Some, increments its value
    fn increment_expected_sequence(&mut self) {
        self.expected_sequence = self.expected_sequence.map(WindowSequence::increment);
    }
}

fn state_for_header(header: Header, fft_size: u32) -> State {
    let timestamp = header.timestamp();
    log::trace!(
        "state_for_header {:#010x} => timestamp {}",
        header.0,
        timestamp
    );
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
    /// Waiting for a zero sample
    Idle,
    /// Got a zero sample, the next sample may be a header
    Zero,
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

/// A u32 wrapper that provides access to the fields of a bin group
///
/// Fields:
/// * Bit 31: 0
/// * Bit 30: 1 if a sequence number is present
/// * Bits 29:16: Window sequence number (14 bits, increments for every FFT window)
/// * Bits 15:0: Bin number
///
struct BinGroupHeader(u32);

const HAS_SEQUENCE_MASK: u32 = 0x40000000;
const SEQUENCE_MASK: u32 = 0x3fff;
const SEQUENCE_SHIFT: u32 = 16;
const BIN_MASK: u32 = 0x0000_ffff;

impl BinGroupHeader {
    /// Returns the sequence number of this window, if one is present
    fn sequence(&self) -> Option<WindowSequence> {
        if (self.0 & HAS_SEQUENCE_MASK) != 0 {
            Some(WindowSequence(
                ((self.0 >> SEQUENCE_SHIFT) & SEQUENCE_MASK) as u16,
            ))
        } else {
            None
        }
    }
    /// Returns the bin number of this header
    fn bin(&self) -> u16 {
        (self.0 & BIN_MASK) as u16
    }
}

/// A 14-bit sequence number
#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct WindowSequence(u16);

impl WindowSequence {
    /// Increments the sequence number (wrapping around after reaching the maximum value)
    /// and returns the new value
    #[must_use]
    pub fn increment(self) -> Self {
        let new_value = (self.0 + 1) & (SEQUENCE_MASK as u16);
        WindowSequence(new_value)
    }
}

mod fmt_impl {
    use super::{State, WindowSequence};
    use std::fmt::{Debug, Display, Formatter, Result};

    impl Debug for State {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                State::Idle => f.debug_struct("Idle").finish(),
                State::Zero => f.debug_struct("Zero").finish(),
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
    impl Debug for WindowSequence {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            Debug::fmt(&self.0, f)
        }
    }
    impl Display for WindowSequence {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            Display::fmt(&self.0, f)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{BinGroupHeader, WindowSequence};

    #[test]
    fn window_sequence_basic() {
        for bin in 0..1024 {
            let no_sequence = BinGroupHeader(bin);
            assert!(no_sequence.sequence().is_none());
            assert_eq!(bin, u32::from(no_sequence.bin()));

            let max_sequence = BinGroupHeader(0x4000_0000 | (16383 << 16) | bin);
            assert_eq!(max_sequence.sequence(), Some(WindowSequence(16383)));
            assert_eq!(bin, u32::from(max_sequence.bin()));
        }
    }
}
