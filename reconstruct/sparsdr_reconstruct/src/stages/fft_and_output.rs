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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use num_complex::Complex32;
use num_traits::Zero;

use crate::iter_ext::IterExt;
use crate::push_reconstruct::FftAndOutputSetup;
use crate::steps::frequency_correct::FrequencyCorrect;
use crate::steps::overlap::OverlapMode;

use super::band_receive::BandReceiver;

/// Runs the FFT and output stages using the provided setup
///
/// On success, this returns the total number of samples written.
pub fn run_fft_and_output_stage(setup: FftAndOutputSetup, stop: Arc<AtomicBool>) {
    let fft_size = setup.fft_size;
    // Set up FFT chain
    let fft_chain = BandReceiver::new(&setup.source, setup.timeout)
        .take_while(|_| !stop.load(Ordering::Relaxed))
        .filter_bins(setup.bins, setup.fft_size)
        .shift(setup.fft_size.into())
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
            (corrector, output_setup.destination)
        })
        .collect::<Vec<_>>();

    // Run FFT, correct frequency and write for each output
    for window in fft_chain {
        for (frequency_correct, destination) in output_chains.iter_mut() {
            let mut output_window = window.clone();
            frequency_correct.correct_samples(output_window.window.samples_mut());

            destination.write_samples(output_window.window.samples());
            if output_window.flushed {
                // Add some zero samples to make the decoders actually run
                let zero_sample = [Complex32::zero()];
                for _ in 0..flush_samples {
                    destination.write_samples(&zero_sample);
                }
            }
        }
    }
}
