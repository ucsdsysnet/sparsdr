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

use std::error;
use std::fmt;
use std::io::{self, ErrorKind, Read};
use std::mem;

use byteorder::{ByteOrder, LE};
use num_complex::Complex32;

pub const SAMPLE_BYTES: usize = mem::size_of::<Complex32>();

/// An iterator that reads uncompressed samples from a byte source
pub struct Samples<R> {
    /// The byte source
    bytes: R,
}

impl<R> Samples<R> {
    pub fn new(bytes: R) -> Self {
        Samples { bytes }
    }
}

impl<R> Iterator for Samples<R>
where
    R: Read,
{
    type Item = Result<Complex32>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut bytes = [0u8; SAMPLE_BYTES];

        let mut bytes_read = 0usize;
        while bytes_read != SAMPLE_BYTES {
            match self.bytes.read(&mut bytes[bytes_read..]) {
                Ok(read_count) => {
                    if read_count == 0 {
                        // Probably reached end of file
                        break;
                    }
                    // Update remaining bytes to read
                    bytes_read += read_count;
                }
                Err(e) => {
                    match e.kind() {
                        ErrorKind::Interrupted => {
                            // Try again
                        }
                        ErrorKind::UnexpectedEof => break,
                        _ => {
                            // Some other error
                            return Some(Err(Error::Io(e)));
                        }
                    }
                }
            }
        }
        // Could be at the end, or could be a partial sample
        if bytes_read == 0 {
            // Nothing to read, at end
            return None;
        } else if bytes_read != SAMPLE_BYTES {
            // Read part of a sample, but no more to read
            return Some(Err(Error::PartialSample));
        }

        // Now the whole sample has been read
        let real = LE::read_f32(&bytes[0..4]);
        let imaginary = LE::read_f32(&bytes[4..8]);
        Some(Ok(Complex32::new(real, imaginary)))
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// A partial sample was read
    PartialSample,
    /// An IO error, other than unexpected end of file
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::PartialSample => write!(f, "Read partial sample"),
            Error::Io(ref inner) => write!(f, "IO error {}", inner),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "sample read error"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::PartialSample => None,
            Error::Io(ref inner) => Some(inner),
        }
    }
}
