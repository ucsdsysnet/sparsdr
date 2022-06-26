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

use std::time::Duration;

use crate::push_reconstruct::WriteSamples;

use super::bins::choice::choose_bins;
use super::bins::BinRange;

/// Default timeout before flushing samples to output
pub const TIMEOUT: Duration = Duration::from_millis(500);

/// Setup for decompression of one band
pub struct BandSetup {
    /// The bins to decompress
    pub bins: BinRange,
    /// The actual FFT size to use
    pub fft_size: u16,
    /// Floor of the center frequency offset, in bins
    pub fc_bins: f32,
    /// Fractional part of center frequency offset, in bins
    pub bin_offset: f32,
    /// Time to wait for a compressed sample before flushing output
    pub timeout: Duration,
    /// The destination to write decompressed samples to
    pub destination: Box<dyn WriteSamples + Send + 'static>,
}

impl BandSetup {
    /// Returns the bins to be decompressed for this band
    pub fn bins(&self) -> BinRange {
        self.bins.clone()
    }
    /// Returns the floor of the center frequency offset, in bins (this is a whole number)
    pub fn fc_bins(&self) -> f32 {
        self.fc_bins
    }
}

/// Setup builder for decompression of one band
pub struct BandSetupBuilder {
    /// Bandwidth of the compressed data
    compressed_bandwidth: f32,
    /// Center frequency to decompress, relative to the center of the compressed data
    center_frequency: f32,
    /// The number of FFT bins used to compress the signals
    compression_fft_size: usize,
    /// The number of bins to select
    bins: u16,
    /// The inverse FFT size
    fft_bins: u16,
    /// Time to wait for a compressed sample before flushing output
    timeout: Duration,
    /// The destination to write decompressed samples to
    destination: Box<dyn WriteSamples + Send + 'static>,
}

impl BandSetupBuilder {
    /// Creates a default band setup that will decompress a full 100 MHz spectrum and write
    /// decompressed samples to the provided source
    pub fn new(
        destination: Box<dyn WriteSamples + Send + 'static>,
        compressed_bandwidth: f32,
        compression_fft_size: usize,
        bins: u16,
        fft_bins: u16,
    ) -> Self {
        BandSetupBuilder {
            compressed_bandwidth,
            center_frequency: 0.0,
            compression_fft_size,
            bins,
            fft_bins,
            timeout: TIMEOUT,
            destination,
        }
    }
    /// Sets the center frequency to decompress
    pub fn center_frequency(self, center_frequency: f32) -> Self {
        BandSetupBuilder {
            center_frequency,
            ..self
        }
    }
    /// Sets the number of bins to decompress
    pub fn bins(self, bins: u16) -> Self {
        BandSetupBuilder { bins, ..self }
    }
    /// Sets the time to wait for a compressed sample before flushing output
    pub fn timeout(self, timeout: Duration) -> Self {
        BandSetupBuilder { timeout, ..self }
    }
    /// Sets the bandwidth of the compressed data
    pub fn compressed_bandwidth(self, compressed_bandwidth: f32) -> Self {
        BandSetupBuilder {
            compressed_bandwidth,
            ..self
        }
    }

    /// Builds a setup from this builder
    pub fn build(self) -> BandSetup {
        let fft_size = self.fft_bins;

        let exact_bin_offset =
            self.compression_fft_size as f32 * self.center_frequency / self.compressed_bandwidth;
        // For fc_bins, round towards zero
        let fc_bins = if exact_bin_offset >= 0.0 {
            exact_bin_offset.floor()
        } else {
            exact_bin_offset.ceil()
        };
        let bin_offset = exact_bin_offset.fract();
        log::debug!(
            "Offset {} bins, whole fc_bins {}, fractional bin_offset {} bins",
            exact_bin_offset,
            fc_bins,
            bin_offset
        );

        let bin_range = choose_bins(self.bins, fc_bins as i16, self.compression_fft_size);

        BandSetup {
            bins: bin_range,
            fft_size,
            bin_offset,
            fc_bins,
            timeout: self.timeout,
            destination: self.destination,
        }
    }
}
