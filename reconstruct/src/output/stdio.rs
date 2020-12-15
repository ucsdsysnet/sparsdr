/*
 * Copyright 2020 The Regents of the University of California
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
//! Output to things that implement std::io::Write
//!

use std::error::Error;
use std::io::Write;

use super::WriteOutput;
use byteorder::{NativeEndian, WriteBytesExt};
use num_complex::Complex32;

/// A simple output that writes native-endian binary complex floats to anything that implements
/// std::io::Write
///
/// The WriteOutput implementation makes several small calls to write() on the enclosed sink,
/// so it is probably a good idea to use a BufWriter around a file or socket.
pub struct StdioOutput<W>(W);

impl<W> StdioOutput<W> {
    /// Creates an output wrapper
    pub fn new(inner: W) -> Self {
        StdioOutput(inner)
    }
}

impl<W> WriteOutput for StdioOutput<W>
where
    W: Write,
{
    fn write_samples(&mut self, samples: &[Complex32]) -> Result<(), Box<dyn Error + Send>> {
        for sample in samples {
            self.0
                .write_f32::<NativeEndian>(sample.re)
                .map_err(box_err)?;
            self.0
                .write_f32::<NativeEndian>(sample.im)
                .map_err(box_err)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Box<dyn Error + Send>> {
        self.0.flush().map_err(box_err)
    }
}

fn box_err(e: std::io::Error) -> Box<dyn Error + Send> {
    Box::new(e)
}
