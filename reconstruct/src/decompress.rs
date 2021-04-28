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

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crossbeam_utils::thread::{self, ScopedJoinHandle};

use crate::band_decompress::BandSetup;
use crate::bins::BinRange;
use crate::component_setup::set_up_stages_combined;
use crate::input::ReadInput;
use crate::stages::fft_and_output::run_fft_and_output_stage;
use crate::stages::input::run_input_stage;

/// Default channel capacity value
const DEFAULT_CHANNEL_CAPACITY: usize = 32;

const DEFAULT_BUFFER_SIZE: usize = 32;

/// Setup for decompression
pub struct DecompressSetup<'d> {
    /// Compressed sample source
    source: Box<dyn ReadInput + 'd>,
    /// Bands to decompress
    bands: Vec<BandSetup<'d>>,
    /// Capacity of input -> FFT/output stage channels
    channel_capacity: usize,
    /// The size of buffers used to read from the USRP and do other steps before the
    /// splitting of samples into bin-specific threads, in units of compression_fft_size
    buffer_size: usize,
    /// The number of FFT bins used to compress the samples
    compression_fft_size: u16,
    /// Stop flag, used to stop compression before the end of the input file
    ///
    /// When this is set to true, all decompression threads will cleanly exit
    stop: Option<Arc<AtomicBool>>,
}

impl<'d> DecompressSetup<'d> {
    /// Creates a new decompression setup with no bands and default channel capacity
    pub fn new(source: Box<dyn ReadInput + 'd>, compression_fft_size: u16) -> Self {
        DecompressSetup {
            source,
            bands: Vec::new(),
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
            buffer_size: DEFAULT_BUFFER_SIZE,
            compression_fft_size,
            stop: None,
        }
    }

    /// Sets the capacity of input -> FFT/output stage channels
    pub fn set_channel_capacity(&mut self, channel_capacity: usize) -> &mut Self {
        self.channel_capacity = channel_capacity;
        self
    }

    /// Sets the size of buffers used to read from the USRP and do other steps before the
    /// splitting of samples into bin-specific threads, in units of compression_fft_size
    pub fn set_buffer_size(&mut self, buffer_size: usize) -> &mut Self {
        self.buffer_size = buffer_size;
        self
    }

    /// Sets the stop flag, which can be used to interrupt decompression before the end of
    /// the input file
    pub fn set_stop_flag(&mut self, stop: Arc<AtomicBool>) -> &mut Self {
        self.stop = Some(stop);
        self
    }

    /// Adds a band to be decompressed to this setup
    pub fn add_band(&mut self, band: BandSetup<'d>) -> &mut Self {
        self.bands.push(band);
        self
    }
}

/// Decompresses bands using the provided setup and returns information about the decompression
pub fn decompress(setup: DecompressSetup<'_>) -> Result<(), Box<dyn std::error::Error + Send>> {
    // If a stop flag was not provided, keep running forever
    let stop = setup
        .stop
        .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));

    // Figure out the stages
    let stages = set_up_stages_combined(
        setup.source,
        setup.bands,
        setup.channel_capacity,
        setup.compression_fft_size,
        setup.buffer_size,
        Arc::clone(&stop),
    );

    // Track number of threads created, including the main thread
    let mut threads = 1usize;
    thread::scope(|scope| {
        // Start a thread for each FFT and output stage
        let fft_and_output_threads: Vec<(
            BinRange,
            ScopedJoinHandle<'_, Result<(), Box<dyn std::error::Error + Send>>>,
        )> = stages
            .fft_and_output
            .into_iter()
            .map(|setup| {
                threads += 1;
                let bins = setup.bins.clone();
                let handle = {
                    scope
                        .builder()
                        .name(format!("Bins {}", setup.bins))
                        .spawn(move |_| run_fft_and_output_stage(setup))
                        .expect("Failed to spawn FFT and output thread")
                };
                (bins, handle)
            })
            .collect();

        // Run the input right here
        run_input_stage(stages.input, stop).expect("Input failure");
        log::debug!("Input exited");

        // Join output threads
        // TODO: Error handling
        for (_bins, thread) in fft_and_output_threads {
            log::debug!("Joining an output thread...");
            thread
                .join()
                .expect("Join failure")
                .expect("FFT/output failure");
        }
    })
    .expect("Unjoined thread panic");

    Ok(())
}
