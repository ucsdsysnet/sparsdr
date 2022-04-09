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

use crate::window::Window;
use num_complex::Complex;
use sparsdr_sample_parser::{Parser, WindowKind};
use std::io::{ErrorKind, Read, Result};

/// An adapter that reads bytes from a byte source, parses them, and discards average windows
pub struct SampleReader<R, P> {
    byte_source: R,
    parser: P,
    /// A buffer with a length equal to the length of a sample
    buffer: Vec<u8>,
    last_data_window_time: Option<u32>,
}

impl<R, P> SampleReader<R, P>
where
    R: Read,
    P: Parser,
{
    /// Creates a sample reader
    pub fn new(byte_source: R, parser: P) -> Self {
        let bytes_per_sample = parser.sample_bytes();
        SampleReader {
            byte_source,
            parser,
            buffer: vec![0u8; bytes_per_sample],
            last_data_window_time: None,
        }
    }
}

impl<R, P> Iterator for SampleReader<R, P>
where
    R: Read,
    P: Parser,
{
    type Item = Result<Window>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.byte_source.read_exact(&mut self.buffer) {
                Ok(_) => {
                    match self.parser.parse(&self.buffer) {
                        Ok(Some(window)) => {
                            match window.kind {
                                WindowKind::Average(_averages) => { /* Try another sample */ }
                                WindowKind::Data(bins) => {
                                    // Check window time
                                    if let Some(last_window_time) = &self.last_data_window_time {
                                        assert_ne!(
                                            window.timestamp, *last_window_time,
                                            "In sample reader, current window (time {}) has the same time as \
                                    previous window",
                                            window.timestamp
                                        );
                                    }
                                    self.last_data_window_time = Some(window.timestamp);

                                    break Some(Ok(Window::with_bins(
                                        window.timestamp.into(),
                                        bins.len(),
                                        bins.into_iter().map(convert_complex),
                                    )));
                                }
                            }
                        }
                        Ok(None) => { /* Try another sample */ }
                        Err(_e) => {
                            log::warn!("Compressed format parse error");
                        }
                    }
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break None,
                Err(e) => break Some(Err(e)),
            }
        }
    }
}

/// Converts a 16-bit complex into a 32-bit float complex value
///
/// -32768 maps to -1.0 and 32767 maps to approximately 1.0 .
fn convert_complex(bin: Complex<i16>) -> Complex<f32> {
    Complex::new(map_value(bin.re), map_value(bin.im))
}

fn map_value(value: i16) -> f32 {
    (value as f32) / 32768.0
}
