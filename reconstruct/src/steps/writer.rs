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

//! Writing of samples to a destination

use std::io::{Error, Result, Write};

use byteorder::{WriteBytesExt, LE};
use libc::{clock_gettime, timespec};
use num_complex::Complex32;

use crate::blocking::BlockLogger;
use crate::steps::overlap::FlushWindow;
use crate::window::Tag;

/// Writes samples to a destination
#[derive(Debug, Default)]
pub struct Writer {
    /// The index of the next sample to be written
    sample_index: u32,
}

impl Writer {
    /// Creates a new writer
    pub fn new() -> Self {
        Writer::default()
    }

    /// Writes windows to a destination and returns the total number of samples written
    ///
    /// Each sample is written as two little-endian 32-bit floating-point values, first the real
    /// part and then the imaginary part.
    ///
    /// time_log: An optional destination where tagged windows will be logged
    pub fn write_windows<W, I>(
        &mut self,
        mut destination: W,
        windows: I,
        logger: &BlockLogger,
        flush_samples: u32,
        downsample: bool,
        mut time_log: Option<&mut (dyn Write + Send)>,
    ) -> Result<u64>
    where
        W: Write,
        I: IntoIterator<Item = FlushWindow>,
    {
        let mut samples_written = 0;

        for window in windows {
            logger.log_blocking(|| -> Result<()> {
                self.write_samples(&mut destination, window.window.samples(), downsample)?;
                if window.flushed {
                    log::debug!("Flushing with zero samples");
                    // Add some zero samples to make the decoders actually run
                    let sample_bytes = [0u8; 8];
                    for _ in 0..flush_samples {
                        destination.write_all(&sample_bytes)?;
                    }
                    destination.flush()?;
                }
                Ok(())
            })?;
            samples_written += window.window.len() as u64;

            if let Some(ref mut log) = time_log {
                if let Some(tag) = window.window.tag() {
                    // Sample index is the index of the last sample in this window
                    log_window(log, tag, self.sample_index)?;
                }
            }
        }
        Ok(samples_written)
    }

    /// Writes samples to a destination
    ///
    /// Each sample is written as two little-endian 32-bit floating-point values, first the real
    /// part and then the imaginary part.
    fn write_samples<W>(
        &mut self,
        mut destination: W,
        samples: &[Complex32],
        downsample: bool,
    ) -> Result<()>
    where
        W: Write,
    {
        let step = if downsample { 2 } else { 1 };
        for sample in samples.iter().step_by(step) {
            destination.write_f32::<LE>(sample.re)?;
            destination.write_f32::<LE>(sample.im)?;

            self.sample_index = self.sample_index.wrapping_add(1);
        }
        Ok(())
    }
}

/// Logs a window output
fn log_window(destination: &mut (dyn Write + Send), tag: &Tag, sample_index: u32) -> Result<()> {
    let mut now = timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // Use clock_gettime, which should be the same in C++ and Rust
    let status = unsafe { clock_gettime(libc::CLOCK_MONOTONIC, &mut now) };
    if status != 0 {
        return Err(Error::last_os_error());
    }
    // CSV format: tag, sample index, seconds, nanoseconds
    writeln!(
        destination,
        "{},{},{},{}",
        tag, sample_index, now.tv_sec, now.tv_nsec
    )?;
    Ok(())
}
