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
//! Top-level decompression interface
//!

use std::io::{BufReader, Read, Result};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use sparsdr_sample_parser::Parser;

use crate::band_decompress::BandSetup;
use crate::push_reconstruct::Reconstruct;
use crate::steps::overlap::OverlapMode;

/// Default channel capacity value
const DEFAULT_CHANNEL_CAPACITY: usize = 0;

/// Setup for decompression
pub struct DecompressSetup {
    /// Sample parser
    pub(crate) parser: Box<dyn Parser>,
    /// Bands to decompress
    pub(crate) bands: Vec<BandSetup>,
    /// Number of bins in the FFT used for compression
    pub(crate) compression_fft_size: usize,
    /// The number of bits in the window timestamp counter
    pub(crate) timestamp_bits: u32,
    /// Capacity of input -> FFT/output stage channels
    pub(crate) channel_capacity: usize,
    pub(crate) overlap_mode: OverlapMode,
    /// Stop flag, used to stop compression before the end of the input file
    ///
    /// When this is set to true, all decompression threads will cleanly exit
    pub(crate) stop: Option<Arc<AtomicBool>>,
}

impl DecompressSetup {
    /// Creates a new decompression setup with no bands and default channel capacity
    pub fn new(parser: Box<dyn Parser>, compression_fft_size: usize, timestamp_bits: u32) -> Self {
        DecompressSetup {
            parser,
            bands: Vec::new(),
            compression_fft_size,
            timestamp_bits,
            overlap_mode: OverlapMode::Flush(0),
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
            stop: None,
        }
    }

    /// Sets the capacity of input -> FFT/output stage channels
    pub fn set_channel_capacity(&mut self, channel_capacity: usize) -> &mut Self {
        self.channel_capacity = channel_capacity;
        self
    }

    /// Sets the stop flag, which can be used to interrupt decompression before the end of
    /// the input file
    pub fn set_stop_flag(&mut self, stop: Arc<AtomicBool>) -> &mut Self {
        self.stop = Some(stop);
        self
    }

    /// Adds a band to be decompressed to this setup
    pub fn add_band(&mut self, band: BandSetup) -> &mut Self {
        self.bands.push(band);
        self
    }

    /// Sets the overlap/flush mode
    pub fn set_overlap_mode(&mut self, overlap_mode: OverlapMode) -> &mut Self {
        self.overlap_mode = overlap_mode;
        self
    }
}

/// Decompresses bands using the provided setup and returns information about the decompression
pub fn decompress(setup: DecompressSetup, source: Box<dyn Read + '_>) -> Result<()> {
    // Implement this using the push reconstruct interface
    let sample_bytes = setup.parser.sample_bytes();
    let mut reader = BufReader::new(source);
    let mut reconstruct = Reconstruct::start(setup)?;

    let mut read_buffer = vec![0u8; sample_bytes];
    loop {
        reader.read_exact(&mut read_buffer)?;
        let status = reconstruct.process_samples(&read_buffer);
        if status.is_break() {
            break;
        }
    }

    reconstruct.shutdown();
    Ok(())
}
