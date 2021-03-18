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

//! The input stage of the decompression, which reads samples from a source, groups them
//! into windows, and shifts them into logical order

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use sparsdr_bin_mask::BinMask;

use crate::bins::BinRange;
use crate::input::{ReadInput, Sample};
use crate::steps::group::Grouper;
use crate::steps::shift::Shift;
use crate::window::{Logical, Window};
use crossbeam_channel::Sender;
use std::error::Error;
use std::ops::Not;

/// The setup for the input stage
pub struct InputSetup<'input> {
    /// Source of compressed samples
    pub source: Box<dyn ReadInput + 'input>,
    /// Send half of channels used to send windows to all FFT stages
    pub destinations: Vec<ToFft>,
    /// The number of FFT bins used to compress the samples
    pub fft_size: u16,
}

/// Information about an FFT stage, and a channel that can be used to send windows there
pub struct ToFft {
    /// The range of bins the FFT stage is interested in
    pub bins: BinRange,
    /// The mask of bins the FFT stage is interested in (same as bins)
    pub bin_mask: BinMask,
    /// A sender on the channel to the FFT stage
    pub tx: Sender<Vec<Window<Logical>>>,
}

impl ToFft {
    /// Sends a window to this FFT/output stage, if this stage is interested in the window
    ///
    /// This function returns Err if the FFT/output thread thread has exited. On success,
    /// it returns true if any window was sent.
    pub fn send_if_interested(&self, windows: &[Window<Logical>]) -> io::Result<bool> {
        let mut windows_to_send = Vec::new();
        for window in windows {
            // Check if the stage is interested
            if window.active_bins().overlaps(&self.bin_mask) {
                windows_to_send.push(window.clone());
            }
        }

        if windows_to_send.is_empty() {
            Ok(false)
        } else {
            // Send the windows
            self.tx.send(windows_to_send).map_err(|_e| {
                // Couldn't send because the channel has been disconnected
                io::Error::new(
                    io::ErrorKind::Other,
                    "A band reconstruction thread has exited unexpectedly",
                )
            })?;
            Ok(true)
        }
    }
}

fn running(stop: &AtomicBool) -> bool {
    stop.load(Ordering::Relaxed).not()
}

pub fn run_input_stage(
    mut setup: InputSetup<'_>,
    stop: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    // Steps
    let mut grouper = Grouper::new(usize::from(setup.fft_size));
    let shift = Shift::new(setup.fft_size);

    // Buffers
    // buffer_size: The number of windows to read at a time
    let buffer_size = 32usize;
    let sample_buffer_size = usize::from(setup.fft_size) * buffer_size;
    let mut samples_in = Vec::with_capacity(sample_buffer_size);
    let mut grouped_windows = Vec::with_capacity(buffer_size);
    let mut shifted_windows = Vec::with_capacity(buffer_size);

    // Prepare the source
    setup.source.start()?;
    while running(&stop) {
        // Re-expand buffers
        samples_in.resize(sample_buffer_size, Sample::default());

        // Read samples from the source until the buffer is full
        // When only a few signals are received, it may take a long time to fill up the whole
        // buffer. Therefore, only call read once. If the call was interrupted, there probably
        // was a good reason (timeout or signal).
        let samples_read_in = setup.source.read_samples(&mut samples_in)?;
        samples_in.truncate(samples_read_in);

        let flush = samples_read_in != sample_buffer_size;
        // If no more samples are coming, exit at the end of this loop
        let last_run = samples_read_in == 0;

        // Group (repeat until all samples have been consumed)
        let mut samples_grouped = 0;
        while samples_grouped != samples_in.len() {
            grouped_windows.resize(buffer_size, Window::default());
            let remaining_samples_in = &samples_in[samples_grouped..];
            let group_status = grouper.group(&remaining_samples_in, &mut grouped_windows);
            samples_grouped += group_status.samples_consumed;

            // Shift windows
            grouped_windows.truncate(group_status.windows_produced);
            shifted_windows.resize(group_status.windows_produced, Window::default());
            shift.shift_windows(&mut grouped_windows, &mut shifted_windows);

            // Send windows to FFT/output stages that are interested
            for destination in setup.destinations.iter() {
                destination.send_if_interested(&shifted_windows)?;
            }
        }

        if flush {
            // Get out the final window and process it
            if let Some(final_window) = grouper.take_current() {
                let shifted_final_window = shift.shift_window(final_window);
                for destination in setup.destinations.iter() {
                    destination.send_if_interested(std::slice::from_ref(&shifted_final_window))?;
                }
            }
        }
        if last_run {
            break;
        }
    }

    Ok(())
}
