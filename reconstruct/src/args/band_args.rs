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

use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// Arguments used to set up a band to be decompressed
#[derive(Debug)]
pub struct BandArgs {
    /// Number of bins to reconstruct (this influences the bins that are selected)
    pub bins: u16,
    /// FFT size for reconstruction (this may be greater or equal to than `bins`)
    pub fft_bins: u16,
    /// Center frequency to decompress
    pub center_frequency: f32,
    /// Path to write to, or None to use standard output
    pub path: Option<PathBuf>,
}

impl FromStr for BandArgs {
    type Err = ParseError;

    /// Parses BandArgs from `bins:frequency[:path]`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');
        let bins: &str = parts.next().ok_or(ParseError::Format)?;
        let fft_bins: &str = parts.next().ok_or(ParseError::Format)?;
        let frequency: &str = parts.next().ok_or(ParseError::Format)?;
        let path: Option<&str> = parts.next();

        let bins = bins
            .parse::<u16>()
            .map_err(|_| ParseError::BinNumber)
            .and_then(validate_bins)?;
        let fft_bins = fft_bins
            .parse::<u16>()
            .map_err(|_| ParseError::BinNumber)
            .and_then(validate_bins)?;
        if fft_bins < bins {
            return Err(ParseError::BinNumberRelationship);
        }

        let frequency = frequency
            .parse::<f32>()
            .map_err(|_| ParseError::CenterFrequency)?;
        let path = path.map(PathBuf::from);

        Ok(BandArgs {
            bins,
            fft_bins,
            center_frequency: frequency,
            path,
        })
    }
}

fn validate_bins(value: u16) -> Result<u16, ParseError> {
    if value > 0 && value % 2 == 0 {
        Ok(value)
    } else {
        Err(ParseError::BinNumber)
    }
}

/// Band argument parse errors
#[derive(Debug)]
pub enum ParseError {
    /// Invalid format
    Format,
    /// Bin number parse failure
    BinNumber,
    /// fft_bins < bins
    BinNumberRelationship,
    /// Center frequency parse failure
    CenterFrequency,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ParseError::Format => {
                write!(f, "Invalid format, expected bins:fft_bins:frequency[:path]")
            }
            ParseError::BinNumber => write!(f, "Invalid number of bins value"),
            ParseError::BinNumberRelationship => {
                write!(f, "FFT bins must be greater than or equal to bins")
            }
            ParseError::CenterFrequency => write!(f, "Invalid center frequency value"),
        }
    }
}

impl Error for ParseError {}
