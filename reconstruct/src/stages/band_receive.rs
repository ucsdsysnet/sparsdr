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

use crossbeam::channel::RecvTimeoutError;

use crate::channel_ext::LoggingReceiver;
use crate::window::{Logical, Status, Window, WindowOrTimestamp};

/// Receives windows over a channel from the reading/filtering subsystem
/// and detects timeouts
///
/// A BandReceiver is an Iterator. It yields Status::Ok with a window if a window is received, or
/// Status::Timeout if no window is received in time.
pub struct BandReceiver<'r> {
    /// Compressed window receiver
    window_rx: &'r LoggingReceiver<WindowOrTimestamp>,
    /// Timeout duration
    timeout: Duration,
}

impl<'r> BandReceiver<'r> {
    /// Creates a band receiver
    ///
    /// window_rx: A receiver that will receive matching windows
    ///
    /// timeout: The approximate time to wait before sending a Timeout state
    pub fn new(window_rx: &'r LoggingReceiver<WindowOrTimestamp>, timeout: Duration) -> Self {
        BandReceiver { window_rx, timeout }
    }
}

impl<'r> Iterator for BandReceiver<'r> {
    type Item = Status<Window<Logical>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.window_rx.recv_timeout(self.timeout) {
            Ok(WindowOrTimestamp::Window(window)) => {
                // Send the window
                Some(Status::Ok(window))
            }
            Ok(WindowOrTimestamp::FirstWindowTimestamp(timestamp)) => {
                Some(Status::FirstWindowTime(timestamp))
            }
            Err(RecvTimeoutError::Timeout) => {
                // Send a timeout
                Some(Status::Timeout)
            }
            Err(RecvTimeoutError::Disconnected) => {
                // The end
                None
            }
        }
    }
}
