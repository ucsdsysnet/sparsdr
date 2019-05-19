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

//! The grouping step

pub mod overflow;

use std::io;

use num_complex::Complex32;

use self::overflow::Overflow;
use crate::input::Sample;
use crate::window::Window;

/// Groups FFT samples into timestamped windows and handles time overflow
pub struct Grouper<I> {
    /// Inner iterator that yields Samples
    inner: I,
    /// FFT size
    fft_size: usize,
    /// The current window being assembled
    window: Option<Window>,
    /// Time overflow calculator
    overflow: Overflow,
}

impl<I> Grouper<I> {
    /// Creates a new grouper to create groups of the specified FFT size
    pub fn new(inner: I, fft_size: usize) -> Self {
        Grouper {
            inner,
            fft_size,
            window: None,
            overflow: Overflow::new(),
        }
    }

    /// Creates a window with a size of self.fft_size and the provided time, bin/index, and amplitude
    fn create_window(&self, time: u64, bin: u16, amplitude: Complex32) -> Window {
        let mut window = Window::new(time, self.fft_size);
        window.set_amplitude(bin, amplitude);
        window
    }
}

impl<I> Iterator for Grouper<I>
where
    I: Iterator<Item = io::Result<Sample>>,
{
    type Item = io::Result<Window>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Read a sample
            let sample = match self.inner.next() {
                Some(Ok(sample)) => sample,
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    // No more samples. Return the current window, if any
                    debug!("End of samples, returning window");
                    return self.window.take().map(Ok);
                }
            };
            // Expand sample time and handle overflow
            let sample_time = self.overflow.expand(sample.time);
            // Apply sample
            if let Some(window_time) = self.window.as_ref().map(Window::time) {
                // Check if this sample belongs to a new window
                if sample_time == window_time {
                    // Definitely the same window, unless the counter has overflowed and counted
                    // back up to the same number. Assume that won't happen.
                    // Store the amplitude
                    let window = self.window.as_mut().unwrap();

                    window.set_amplitude(sample.index, sample.amplitude);
                    // Mark the bin as active
                    window
                        .active_bins_mut()
                        .set(usize::from(sample.index), true);
                // Continue reading samples
                } else {
                    // A different window
                    trace!("Starting new window with time {}", sample_time);
                    // Take out the current window to return later
                    let old_window = self.window.take().unwrap();
                    // Set up the new window
                    self.window =
                        Some(self.create_window(sample_time, sample.index, sample.amplitude));
                    // Return the old, now complete, window
                    return Some(Ok(old_window));
                }
            } else {
                // Start the first window
                debug!("Starting first window with time {}", sample_time);
                self.window = Some(self.create_window(sample_time, sample.index, sample.amplitude));
                // Continue reading samples
            }
        }
    }
}
