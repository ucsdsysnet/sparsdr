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

use std::error::Error;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError};
use num_complex::Complex32;

use crate::bins::BinRange;
use crate::output::WriteOutput;
use crate::steps::fft::Fft;
use crate::steps::filter_bins::FilterBins;
use crate::steps::frequency_correct::FrequencyCorrect;
use crate::steps::overlap::Overlap;
use crate::steps::phase_correct::PhaseCorrect;
use crate::steps::shift::Shift;
use crate::window::{Logical, TimeWindow, Window};
use std::ops::Not;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct FftAndOutputSetup<'d> {
    /// Source of windows
    pub source: Receiver<Vec<Window<Logical>>>,
    /// The bins to decompress
    pub bins: BinRange,
    /// The actual FFT size to use
    pub fft_size: u16,
    /// The number of FFT bins used to compress the samples
    pub compression_fft_size: u16,
    /// Floor of the center frequency offset, in bins
    pub fc_bins: f32,
    /// Time to wait for a compressed sample before flushing output
    ///
    /// If this is None, the output will never be flushed until there are no more samples.
    pub timeout: Option<Duration>,
    /// The output setups
    pub outputs: Vec<OutputSetup<'d>>,
    /// The stop flag
    pub stop: Arc<AtomicBool>,
}

impl<'d> FftAndOutputSetup<'d> {
    /// Returns false if self.stop is true, returns false otherwise
    pub fn running(&self) -> bool {
        self.stop.load(Ordering::Relaxed).not()
    }

    pub fn receive_windows(&self) -> Result<Vec<Window<Logical>>, RecvTimeoutError> {
        match self.timeout.as_ref() {
            Some(timeout) => self.source.recv_timeout(timeout.clone()),
            None => self.source.recv().map_err(From::from),
        }
    }
}

pub struct OutputSetup<'d> {
    /// Fractional part of center frequency offset, in bins
    pub bin_offset: f32,
    /// The destination to write decompressed samples to
    pub destination: Box<dyn WriteOutput + Send + 'd>,
}

/// Frequency correction and output for one band
struct OutputChain<'d> {
    frequency_correct: FrequencyCorrect,
    /// The destination to write decompressed samples to
    destination: Box<dyn WriteOutput + Send + 'd>,
}

/// Runs the FFT and output stages using the provided setup
pub fn run_fft_and_output_stage(setup: FftAndOutputSetup<'_>) -> Result<(), Box<dyn Error + Send>> {
    let this_thread = std::thread::current();
    let thread_name = this_thread.name().unwrap_or("<unknown>");
    let status = run_fft_and_output_stage_inner(setup);
    match &status {
        Ok(()) => log::info!("Reconstruction thread {:?} clean exit", thread_name),
        Err(e) => log::error!("Reconstruction thread {:?} error: {}", thread_name, e),
    }
    status
}

pub fn run_fft_and_output_stage_inner(
    mut setup: FftAndOutputSetup<'_>,
) -> Result<(), Box<dyn Error + Send>> {
    // Set up steps
    let filter_bins = FilterBins::new(setup.bins.clone(), setup.fft_size);
    let shift = Shift::new(setup.fft_size);
    let mut phase_correct = PhaseCorrect::new(setup.fc_bins);
    let mut fft = Fft::new(
        usize::from(setup.fft_size),
        usize::from(setup.compression_fft_size),
    );
    let mut overlap = Overlap::new();
    let fft_size = setup.fft_size;
    let mut output_chains: Vec<OutputChain<'_>> = setup
        .outputs
        .drain(..)
        .map(|output_setup| OutputChain {
            frequency_correct: FrequencyCorrect::new(output_setup.bin_offset, fft_size),
            destination: output_setup.destination,
        })
        .collect();

    // Allocate buffers
    // Overlap is the only step that needs to be flushed and has an unpredictable input-to-output
    // ratio. All the others are 1:1.

    // Window<Fft>s: Shift -> Phase correct -> FFT
    let mut windows_before_fft: Vec<Window> = Vec::new();
    // TimeWindows: FFT -> Overlap
    let mut windows_before_overlap: Vec<TimeWindow> = Vec::new();
    // TimeWindows: Overlap -> Frequency correct (per-output)
    let mut windows_before_frequency_correct: Vec<TimeWindow> = Vec::new();

    // Run everything
    while setup.running() {
        match setup.receive_windows() {
            Ok(mut windows_before_shift) => {
                // Feed windows in to process
                filter_bins.filter_windows(&mut windows_before_shift);
                windows_before_fft.resize_with(windows_before_shift.len(), || {
                    Window::new(0, usize::from(fft_size))
                });
                shift.shift_windows(&mut windows_before_shift, &mut windows_before_fft);
                phase_correct.correct_windows(&mut windows_before_fft);
                windows_before_overlap.resize_with(windows_before_fft.len(), || {
                    TimeWindow::new(0, vec![Complex32::default(); usize::from(fft_size)])
                });
                fft.run(&mut windows_before_fft, &mut windows_before_overlap);
                // Overlap output size: The maximum number of output windows = number of input windows
                windows_before_frequency_correct.resize_with(windows_before_overlap.len(), || {
                    TimeWindow::new(0, vec![Complex32::default(); usize::from(fft_size)])
                });
                let overlap_result = overlap.run(
                    &windows_before_overlap,
                    &mut windows_before_frequency_correct,
                );
                // Use only the post-overlap windows
                windows_before_frequency_correct.truncate(overlap_result.windows_produced);

                // Send to all output chains
                for output in output_chains.iter_mut() {
                    let mut windows_out = windows_before_frequency_correct.clone();
                    output.frequency_correct.correct_windows(&mut windows_out);
                    for window in windows_out.iter() {
                        output.destination.write_samples(window.samples())?;
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // Flush samples: Overlap and outputs
                if let Some(overlap_flushed) = overlap.flush() {
                    for output in output_chains.iter_mut() {
                        let mut window = overlap_flushed.clone();
                        output
                            .frequency_correct
                            .correct_samples(window.samples_mut());
                        output.destination.write_samples(window.samples())?;
                        output.destination.flush()?;
                    }
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                // Flush samples: Overlap and outputs
                if let Some(overlap_flushed) = overlap.flush() {
                    for output in output_chains.iter_mut() {
                        let mut window = overlap_flushed.clone();
                        output
                            .frequency_correct
                            .correct_samples(window.samples_mut());
                        output.destination.write_samples(window.samples())?;
                        output.destination.flush()?;
                    }
                }
                // No more windows will appear, so exit
                return Ok(());
            }
        };
    }
    Ok(())
}
