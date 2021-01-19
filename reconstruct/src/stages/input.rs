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
pub struct InputSetup {
    /// Source of compressed samples
    pub source: Box<dyn ReadInput>,
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
                    "A decompression thread has exited unexpectedly",
                )
            })?;
            Ok(true)
        }
    }
}

fn running(stop: &AtomicBool) -> bool {
    stop.load(Ordering::Relaxed).not()
}

/// Repeatedly calls read_samples() on the provided sample source until the provided buffer is
/// full
///
/// If the sample source returns Ok(0) indicating an end of file, the buffer will not be
/// completely filled.
///
/// This function returns the number of samples read into the buffer.
fn read_samples_exact(
    source: &mut dyn ReadInput,
    buffer: &mut [Sample],
    stop: &AtomicBool,
) -> Result<usize, Box<dyn Error>> {
    let mut samples_read = 0;
    while samples_read != buffer.len() && running(stop) {
        let remaining = &mut buffer[samples_read..];
        let samples_read_this_time = source.read_samples(remaining)?;
        if samples_read_this_time == 0 {
            // Reached end of file
            break;
        } else {
            samples_read += samples_read_this_time;
        }
    }

    Ok(samples_read)
}

pub fn run_input_stage(mut setup: InputSetup, stop: Arc<AtomicBool>) -> Result<(), Box<dyn Error>> {
    // Steps
    let mut grouper = Grouper::new(usize::from(setup.fft_size));
    let shift = Shift::new(setup.fft_size);

    // Buffers
    let buffer_size = 64usize;
    let sample_buffer_size = usize::from(setup.fft_size) * buffer_size;
    let mut samples_in = Vec::with_capacity(sample_buffer_size);
    let mut grouped_windows = Vec::with_capacity(buffer_size);
    let mut shifted_windows = Vec::with_capacity(buffer_size);

    while running(&stop) {
        let mut last_run = false;
        // Re-expand buffers
        samples_in.resize(sample_buffer_size, Sample::default());

        // Read samples from the source until the buffer is full
        let samples_read_in = read_samples_exact(&mut *setup.source, &mut samples_in, &stop)?;
        samples_in.truncate(samples_read_in);

        if samples_read_in != sample_buffer_size {
            // Couldn't read a full buffer of samples, which indicates that no more samples will
            // come later. Exit at the end of this loop.
            last_run = true;
        }

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

        if last_run {
            // Get out the final window and process it
            if let Some(final_window) = grouper.take_current() {
                let shifted_final_window = shift.shift_window(final_window);
                for destination in setup.destinations.iter() {
                    destination.send_if_interested(std::slice::from_ref(&shifted_final_window))?;
                }
            }
            break;
        }
    }

    Ok(())

    // // Set up iterator chain
    // // Shift and send to the decompression thread
    // let shift = setup
    //     .samples
    //     .take_while(|_| !stop.load(Ordering::Relaxed))
    //     .group(usize::from(setup.fft_size))
    //     .shift_result(setup.fft_size);
    //
    // // Process windows
    // // Latency measurement hack: detect when the channel changes from active to inactive
    // let mut prev_active = false;
    // for window in shift {
    //     let mut window = window?;
    //     // Give this window a tag
    //     let window_tag = next_tag;
    //     window.set_tag(window_tag);
    //     next_tag = next_tag.next();
    //
    //     // Send to each interested FFT stage
    //     for fft_stage in setup.destinations.iter() {
    //         match fft_stage.send_if_interested(&window) {
    //             Ok(sent) => {
    //                 // Latency measurement hack that works correctly only when there is only one
    //                 // band to decompress: Detect when the bins have become inactive and log that
    //                 if !sent && prev_active {
    //                     if let Some(ref mut log) = setup.input_time_log {
    //                         log_inactive_channel(&mut *log, "BLE 37", window_tag)?;
    //                     }
    //                 }
    //
    //                 prev_active = sent;
    //             }
    //             Err(_) => {
    //                 // Other thread could have exited normally due to the stop flag, so just
    //                 // exit this thread normally
    //                 break;
    //             }
    //         }
    //     }
    // }
    //
    // // Collect block logs
    // let channel_send_blocks = setup
    //     .destinations
    //     .into_iter()
    //     .map(|to_fft| (to_fft.bins, to_fft.tx.logs()))
    //     .collect();
    //
    // Ok(InputReport {
    //     channel_send_blocks,
    // })
}
