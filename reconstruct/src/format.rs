/*
 * Copyright 2020 The Regents of the University of California
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
//! Input sample formats, sample parsing, and associated settings
//!

use crate::input::iqzip::{AverageSample, DataSample, Sample};
use byteorder::{ByteOrder, LittleEndian};
use std::fmt;

/// A sample format that can be used to read samples
#[derive(Clone)]
pub struct SampleFormat {
    /// The function that parses a sample from a slice of 8 bytes
    parse_sample: fn(&[u8]) -> Sample,
    /// FFT size used for compression
    fft_size: u16,
    /// Bandwidth that the full FFT size spans (equal to the sample rate)
    compressed_bandwidth: f32,
}

impl SampleFormat {
    /// Returns a format for use with a USRP N210
    pub fn n210() -> SampleFormat {
        SampleFormat {
            parse_sample: parse_sample_n210,
            fft_size: 2048,
            compressed_bandwidth: 100_000_000.0,
        }
    }
    /// Returns a format for use with an Analog Devices Pluto SDR
    pub fn pluto() -> SampleFormat {
        SampleFormat {
            parse_sample: parse_sample_pluto,
            fft_size: 1024,
            compressed_bandwidth: 61_440_000.0,
        }
    }

    /// Parses a sample from a slice of 8 bytes
    pub fn parse_sample(&mut self, bytes: &[u8]) -> Sample {
        (self.parse_sample)(bytes)
    }
    /// Returns the FFT size used for compression
    pub fn fft_size(&self) -> u16 {
        self.fft_size
    }
    /// Returns the bandwidth (sample rate) used for compression
    pub fn compressed_bandwidth(&self) -> f32 {
        self.compressed_bandwidth
    }
}

impl fmt::Debug for SampleFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SampleFormat")
            .field("parse_sample", &"[function pointer]")
            .field("fft_size", &self.fft_size)
            .field("compressed_bandwidth)", &self.compressed_bandwidth)
            .finish()
    }
}

fn parse_sample_n210(bytes: &[u8]) -> Sample {
    assert_eq!(bytes.len(), 8, "Incorrect sample length");

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

fn parse_sample_pluto(bytes: &[u8]) -> Sample {
    assert_eq!(bytes.len(), 8, "Incorrect sample length");

    // Little-endian
    type E = LittleEndian;
    let fft_index = E::read_u16(&bytes[6..8]);
    let time = E::read_u16(&bytes[4..6]);
    // Last 4 bytes may be either real/imaginary signal or average magnitude
    let magnitude = {
        // Magnitude is in two 2-byte chunks. Bytes within each chunk are little endian,
        // but the more significant chunk is first.
        let more_significant = E::read_u16(&bytes[2..4]);
        let less_significant = E::read_u16(&bytes[0..2]);
        u32::from(more_significant) << 16 | u32::from(less_significant)
    };
    let real = E::read_i16(&bytes[2..4]);
    let imag = E::read_i16(&bytes[0..2]);

    let is_average = ((fft_index >> 15) & 1) == 1;
    let index = (fft_index >> 5) & 0x3ff;
    // Reassemble time from MSBs and other bits
    let time = u32::from(time) | (u32::from(fft_index & 0x1F) << 16);

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
