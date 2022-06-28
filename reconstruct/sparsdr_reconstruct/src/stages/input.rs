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

use std::io::{Error, ErrorKind, Result};
use std::ops::ControlFlow;

use crossbeam::Sender;
use num_traits::Zero;

use crate::bins::BinRange;
use crate::iter::PushIterator;
use crate::window::{Logical, Window, WindowOrTimestamp};

/// The setup for the input stage
pub struct InputSetup {
    /// Number of FFT bins used for compression
    pub compression_fft_size: usize,
    /// The number of bits used to store the timestamp of each window
    pub timestamp_bits: u32,
    /// Send half of channels used to send windows to all FFT stages
    pub destinations: Vec<ToFft>,
}

/// Information about an FFT stage, and a channel that can be used to send windows there
pub struct ToFft {
    /// The range of bins the FFT stage is interested in
    pub bins: BinRange,
    /// A sender on the channel to the FFT stage
    pub tx: Sender<WindowOrTimestamp>,
}

impl ToFft {
    /// Sends a window to this FFT/output stage, if this stage is interested in the window
    ///
    /// This function returns Err if the FFT/output thread thread has exited. On success,
    /// it returns true if the window was sent.
    pub fn send_if_interested(&self, window: &Window<Logical>) -> Result<bool> {
        // Check if the stage is interested
        let interested = self
            .bins
            .as_usize_range()
            .any(|index| !window.bins()[index].is_zero());
        if interested {
            match self.tx.send(WindowOrTimestamp::Window(window.clone())) {
                Ok(()) => Ok(true),
                Err(_) => {
                    // This can happen when using the stop flag if the other thread stops before
                    // this one. It's not really an error here. If the other thread panicked,
                    // the higher-level code will detect it.
                    Err(Error::new(
                        ErrorKind::Other,
                        "A decompression thread has exited unexpectedly",
                    ))
                }
            }
        } else {
            // Not interested, don't send
            Ok(false)
        }
    }

    pub fn send_first_window_time(&self, time: u64) {
        self.tx
            .send(WindowOrTimestamp::FirstWindowTimestamp(time))
            .expect("Can't send first window time");
    }
}

// pub fn run_input_stage<I>(setup: InputSetup<I>, stop: Arc<AtomicBool>) -> Result<InputReport>
// where
//     I: Iterator<Item = Result<Window>>,
// {
//     // Tag windows that have newly active channels
//     let mut next_tag = Tag::default();
//
//     // Set up iterator chain
//     // Shift and send to the decompression thread
//     let shift = setup
//         .samples
//         .take_while(|_| !stop.load(Ordering::Relaxed))
//         .overflow_correct(setup.timestamp_bits)
//         .shift_result(
//             setup
//                 .compression_fft_size
//                 .try_into()
//                 .expect("FFT size too large"),
//         );
//
//     // Sequence:
//     // Sample parser
//     // Overflow correct
//     // Shift (FFT to logical order)
//
//     let mut prev_window_time: Option<u64> = None;
//     let mut first_window = true;
//
//     // Process windows
//     for window in shift {
//         let mut window = window?;
//
//         if let Some(prev_window_time) = prev_window_time {
//             assert!(
//                 window.time() > prev_window_time,
//                 "Current window (time {}) is not after previous window (time {})",
//                 window.time(),
//                 prev_window_time
//             );
//         }
//         prev_window_time = Some(window.time());
//
//         // Give this window a tag
//         let window_tag = next_tag;
//         window.set_tag(window_tag);
//         next_tag = next_tag.next();
//
//         if first_window {
//             // Send the timestamp to every FFT stage
//             log::debug!("Sending first window timestamp {}", window.time());
//             for fft_stage in &setup.destinations {
//                 fft_stage.send_first_window_time(window.time());
//             }
//             first_window = false;
//         }
//
//         // Send to each interested FFT stage
//         for fft_stage in setup.destinations.iter() {
//             match fft_stage.send_if_interested(&window) {
//                 Ok(_sent) => {}
//                 Err(_) => {
//                     // Other thread could have exited normally due to the stop flag, so just
//                     // exit this thread normally
//                     break;
//                 }
//             }
//         }
//     }
//
//     // Collect block logs
//     let channel_send_blocks = setup
//         .destinations
//         .into_iter()
//         .map(|to_fft| (to_fft.bins, to_fft.tx.logs()))
//         .collect();
//
//     Ok(InputReport {
//         channel_send_blocks,
//     })
// }

/// A push iterator that handles windows (with logical bin order) and sends them to the appropriate
/// reconstruction threads
pub struct ToFfts {
    prev_window_time: Option<u64>,
    setup: InputSetup,
    first_window: bool,
}

impl ToFfts {
    pub fn new(setup: InputSetup) -> Self {
        ToFfts {
            prev_window_time: None,
            setup,
            first_window: true,
        }
    }
}

impl PushIterator<Window<Logical>> for ToFfts {
    type Error = ();

    fn push(&mut self, window: Window<Logical>) -> ControlFlow<Self::Error> {
        if let Some(prev_window_time) = self.prev_window_time {
            assert!(
                window.time() > prev_window_time,
                "Current window (time {}) is not after previous window (time {})",
                window.time(),
                prev_window_time
            );
        }
        self.prev_window_time = Some(window.time());

        if self.first_window {
            // Send the timestamp to every FFT stage
            log::debug!("Sending first window timestamp {}", window.time());
            for fft_stage in &self.setup.destinations {
                fft_stage.send_first_window_time(window.time());
            }
            self.first_window = false;
        }

        // Send to each interested FFT stage
        for fft_stage in self.setup.destinations.iter() {
            match fft_stage.send_if_interested(&window) {
                Ok(_sent) => {}
                Err(_) => {
                    // Other thread could have exited normally due to the stop flag, so just
                    // exit this thread normally
                    return ControlFlow::Break(());
                }
            }
        }
        ControlFlow::Continue(())
    }

    fn flush(&mut self) -> std::result::Result<(), Self::Error> {
        // Nothing to do (all samples get sent immediately)
        Ok(())
    }
}
