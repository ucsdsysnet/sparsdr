/*
 * Copyright 2021 The Regents of the University of California
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

extern crate byteorder;
extern crate log;
extern crate num_complex;
extern crate num_traits;

mod v1;
mod v2;

pub use self::v2::V2Parser;

use num_complex::Complex;

pub trait Parser {
    /// Returns the number of bytes in the compressed sample stream that represent one sample
    fn sample_bytes(&self) -> usize;

    /// Parses `SAMPLE_BYTES` bytes and returns a complete window if one is available
    fn parse(&mut self, bytes: &[u8]) -> Result<Option<Window>, ParseError>;
}

#[derive(Debug, PartialEq)]
pub struct Window {
    pub timestamp: u32,
    pub kind: WindowKind,
}

#[derive(Debug, PartialEq)]
pub enum WindowKind {
    Average(Vec<u32>),
    Data(Vec<Complex<i16>>),
}

/// An error that occurred while parsing
#[derive(Debug, PartialEq)]
pub struct ParseError(());

impl std::error::Error for ParseError {}

mod fmt_impl {
    use super::ParseError;
    use std::fmt::{Debug, Display, Formatter, Result};

    impl Display for ParseError {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "Incorrect compressed sample format")
        }
    }
}
