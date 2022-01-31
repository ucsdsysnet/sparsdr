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

//! Combines the FFT stage and output stage into one, running one or more output
//! stages sequentially
//!
//! # FFT stage
//!
//! The FFT stage receives logical-order windows and produces overlapped time-domain windows. It
//! includes these steps:
//!
//! * Filter bins
//! * Shift (logical to FFT order)
//! * Phase correct
//! * FFT
//! * Overlap
//!
//! If the FFT stage does not receive a window over its source channel within the configured
//! timeout, it flushes any window stored in the overlap process and sends everything to the
//! output stage.
//!
//! The parameters for the FFT stage include a bin range, FFT size, and fc_bins, but not the bin
//! offset.
//!
//! The FFT stage sends its output to one or more output stages.
//!
//! # Output stage
//!
//! The output stage of the decompression process
//!
//! The output stage receives overlapped time-domain windows from the FFT stage. It applies a
//! frequency correction and writes samples to the destination.
//!

use crate::steps::overlap::OverlapMode;
use std::io::{Result, Write};
use std::iter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::bins::BinRange;
use crate::blocking::{BlockLogger, BlockLogs};
use crate::channel_ext::LoggingReceiver;
use crate::iter_ext::IterExt;
use crate::steps::frequency_correct::FrequencyCorrect;
use crate::steps::writer::Writer;
use crate::window::WindowOrTimestamp;

use super::band_receive::BandReceiver;

pub struct FftAndOutputSetup<'w> {
    /// Source of windows
    pub source: LoggingReceiver<WindowOrTimestamp>,
    /// The bins to decompress
    pub bins: BinRange,
    /// The FFT size used for compression
    pub compression_fft_size: usize,
    /// The actual FFT size to use
    pub fft_size: u16,
    /// Floor of the center frequency offset, in bins
    pub fc_bins: f32,
    /// Time to wait for a compressed sample before flushing output
    pub timeout: Duration,
    /// Overlap mode (gaps or flush samples)
    pub overlap: OverlapMode,
    /// The output setups
    pub outputs: Vec<OutputSetup<'w>>,
}

pub struct OutputSetup<'w> {
    /// Fractional part of center frequency offset, in bins
    pub bin_offset: f32,
    /// The destination to write decompressed samples to
    pub destination: Box<dyn Write + Send + 'w>,
    /// Tagged window time log
    pub time_log: Option<Box<dyn Write + Send>>,
}

/// A report on the execution of the FFT and output stage
#[derive(Debug)]
pub struct FftOutputReport {
    /// The total number of decompressed samples written
    pub samples: u64,
    /// Logs of blocking on the input channel
    pub channel_blocks: BlockLogs,
    /// Logs of blocking on the output
    pub output_blocks: BlockLogs,
}

/// Runs the FFT and output stages using the provided setup
///
/// On success, this returns the total number of samples written.
pub fn run_fft_and_output_stage(
    mut setup: FftAndOutputSetup<'_>,
    stop: Arc<AtomicBool>,
) -> Result<FftOutputReport> {
    // Time log headers for each file
    for log in setup
        .outputs
        .iter_mut()
        .filter_map(|setup| setup.time_log.as_mut())
    {
        writeln!(log, "Tag,SampleIndex,Seconds,Nanoseconds")?;
    }

    let fft_size = setup.fft_size;
    // Set up FFT chain
    let fft_chain = BandReceiver::new(&setup.source, setup.timeout)
        .take_while(|_| !stop.load(Ordering::Relaxed))
        .filter_bins(setup.bins, setup.fft_size)
        .shift(setup.fft_size)
        .phase_correct(setup.fc_bins)
        .fft(setup.compression_fft_size, setup.fft_size);

    let fft_chain = match setup.overlap {
        OverlapMode::Gaps => fft_chain.overlap_gaps(usize::from(setup.fft_size)),
        OverlapMode::Flush(_) => fft_chain.overlap_flush(usize::from(setup.fft_size)),
    };
    // Get the number of flush samples to add, or 0 in overlap-gaps mode
    let flush_samples = match setup.overlap {
        OverlapMode::Gaps => 0,
        OverlapMode::Flush(samples) => samples,
    };

    // Set up a frequency corrector for each output
    let mut output_chains = setup
        .outputs
        .into_iter()
        .map(|output_setup| {
            let corrector = FrequencyCorrect::new(output_setup.bin_offset, fft_size);
            (corrector, output_setup.destination, output_setup.time_log)
        })
        .collect::<Vec<_>>();

    // Run FFT, correct frequency and write for each output
    let out_block_logger = BlockLogger::new();
    let mut writer = Writer::new();
    let mut total_samples = 0u64;
    for window in fft_chain {
        for (frequency_correct, destination, time_log) in output_chains.iter_mut() {
            let mut output_window = window.clone();
            frequency_correct.correct_samples(output_window.window.samples_mut());
            // time_log borrowing
            let time_log: Option<&mut (dyn Write + Send)> = match time_log {
                Some(time_log) => Some(&mut *time_log),
                None => None,
            };
            let samples = writer.write_windows(
                destination,
                iter::once(output_window),
                &out_block_logger,
                flush_samples,
                time_log,
            )?;
            total_samples = total_samples.saturating_add(samples);
        }
    }

    Ok(FftOutputReport {
        samples: total_samples,
        channel_blocks: setup.source.logs(),
        output_blocks: out_block_logger.logs(),
    })
}
