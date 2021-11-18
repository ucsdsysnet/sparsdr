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

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Result, Write};

use crate::args::CompressedFormat;
use log::debug;
use simplelog::LevelFilter;

use super::args::Args;
use super::args::BandArgs;

/// The setup for a decompression operation
///
/// A Setup is created from the command-line arguments (Args)
#[non_exhaustive]
pub struct Setup {
    /// Source for compressed samples
    pub source: Box<dyn Read + Send>,
    /// Size of the source file in bytes, if known
    pub source_length: Option<u64>,
    /// Log level
    pub log_level: LevelFilter,
    /// Bandwidth used to create the compressed data
    pub compressed_bandwidth: f32,
    /// Size of the FFT used for compression
    pub compression_fft_size: usize,
    /// The number of bits in the window timestamp counter
    pub timestamp_bits: u32,
    /// The compressed sample format
    pub sample_format: CompressedFormat,
    /// Bands to decompress
    pub bands: Vec<BandSetup>,
    /// Flag to enable progress bar
    pub progress_bar: bool,
    /// Capacity of input -> FFT/output stage channels
    pub channel_capacity: usize,
}

/// The setup for decompressing a band
pub struct BandSetup {
    /// Number of bins to select
    pub bins: u16,
    /// Reconstruction FFT size
    pub fft_bins: u16,
    /// Center frequency to decompress
    pub center_frequency: f32,
    /// Destination to write to
    pub destination: Box<dyn Write + Send>,
}

impl Setup {
    pub fn from_args(args: Args) -> Result<Self> {
        let buffer = args.buffer;
        // Open source and get length
        // The order of opening (source, then output files) is important to prevent deadlock
        // when using named pipes.
        let source: Box<dyn Read + Send> = match args.source_path {
            Some(ref path) => {
                debug!("Opening file {} to read compressed samples", path.display());
                if args.buffer {
                    Box::new(BufReader::new(File::open(path)?))
                } else {
                    Box::new(File::open(path)?)
                }
            }
            None => {
                debug!("Reading compressed samples from standard input");
                // Standard input
                if args.buffer {
                    Box::new(BufReader::new(io::stdin()))
                } else {
                    Box::new(io::stdin())
                }
            }
        };
        let source_length = args
            .source_path
            .and_then(|path| fs::metadata(path).ok())
            .map(|data| data.len())
            .filter(|&length| length != 0);

        // Open band output files
        let bands = args
            .bands
            .into_iter()
            .map(|band_args| BandSetup::from_args(band_args, buffer))
            .collect::<Result<Vec<BandSetup>>>()?;

        debug!("Finished opening files");

        Ok(Setup {
            source,
            source_length,
            log_level: args.log_level,
            compressed_bandwidth: args.compressed_bandwidth,
            compression_fft_size: args.compression_fft_size,
            timestamp_bits: args.timestamp_bits,
            sample_format: args.sample_format,
            bands,
            progress_bar: args.progress_bar,
            channel_capacity: args.channel_capacity,
        })
    }
}

impl BandSetup {
    fn from_args(args: BandArgs, buffer: bool) -> Result<Self> {
        // Open destination
        let destination: Box<dyn Write + Send> = match args.path {
            Some(ref path) => {
                debug!("Opening file {} for output", path.display());
                if buffer {
                    Box::new(BufWriter::new(File::create(path)?))
                } else {
                    Box::new(File::create(path)?)
                }
            }
            None => {
                // Standard output
                debug!("Writing output to standard output");
                if buffer {
                    Box::new(BufWriter::new(io::stdout()))
                } else {
                    Box::new(io::stdout())
                }
            }
        };

        Ok(BandSetup {
            bins: args.bins,
            fft_bins: args.fft_bins,
            center_frequency: args.center_frequency,
            destination,
        })
    }
}
