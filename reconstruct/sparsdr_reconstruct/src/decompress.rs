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

use std::io::Result;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use sparsdr_sample_parser::Parser;

use crate::band_decompress::BandSetup;
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
pub fn decompress(setup: DecompressSetup) -> Result<()> {
    // // Figure out the stages
    // let stages = set_up_stages_combined(
    //     setup.source,
    //     setup.bands,
    //     setup.compression_fft_size,
    //     setup.timestamp_bits,
    //     setup.channel_capacity,
    //     setup.overlap_mode,
    // );
    //
    // // Measure time
    // let start_time = Instant::now();
    // // Track number of threads created, including the main thread
    // let mut threads = 1usize;
    // // Create a stop flag that's always false if the caller did not provide one
    // let stop = setup
    //     .stop
    //     .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));
    //
    // let (input_report, fft_output_reports): (InputReport, BTreeMap<BinRange, FftOutputReport>) =
    //     thread::scope(|scope| {
    //         // Start a thread for each FFT and output stage
    //         // The threads all need to be started here before running the input stage.
    //         #[allow(clippy::needless_collect)]
    //         let fft_and_output_threads: Vec<(
    //             BinRange,
    //             ScopedJoinHandle<'_, Result<FftOutputReport>>,
    //         )> = stages
    //             .fft_and_output
    //             .into_iter()
    //             .map(|setup| {
    //                 threads += 1;
    //                 let bins = setup.bins.clone();
    //                 let handle = {
    //                     let stop = Arc::clone(&stop);
    //                     scope
    //                         .builder()
    //                         .name(format!("Bins {}", setup.bins))
    //                         .spawn(move |_| run_fft_and_output_stage(setup, stop))
    //                         .expect("Failed to spawn FFT and output thread")
    //                 };
    //                 (bins, handle)
    //             })
    //             .collect();
    //
    //         // Run the input right here
    //         let input_report = run_input_stage(stages.input, stop)?;
    //
    //         // Track the last error from a thread
    //         let mut last_error: Option<Error> = None;
    //
    //         // Join output threads
    //         let fft_output_reports: BTreeMap<BinRange, FftOutputReport> = fft_and_output_threads
    //             .into_iter()
    //             .map(|(bins, thread)| {
    //                 let report: Option<(BinRange, FftOutputReport)> = match thread.join() {
    //                     Ok(Ok(report)) => Some((bins, report)),
    //                     Ok(Err(e)) => {
    //                         // Thread returned an error
    //                         last_error = Some(e);
    //                         // No report
    //                         None
    //                     }
    //                     Err(_) => {
    //                         // Thread panicked
    //                         last_error = Some(Error::new(
    //                             ErrorKind::Other,
    //                             "An output thread has panicked",
    //                         ));
    //                         // No report
    //                         None
    //                     }
    //                 };
    //                 report
    //             })
    //             .flatten()
    //             .collect();
    //
    //         let decompress_status: Result<(InputReport, BTreeMap<BinRange, FftOutputReport>)> =
    //             match last_error {
    //                 Some(e) => Err(e),
    //                 None => Ok((input_report, fft_output_reports)),
    //             };
    //         decompress_status
    //     })
    //     .expect("Unjoined thread panic")?;
    //
    // let end_time = Instant::now();
    // let run_time = end_time.duration_since(start_time);
    //
    // let report = assemble_report(input_report, fft_output_reports, threads, run_time);
    //
    // Ok(report)
    todo!()
}
