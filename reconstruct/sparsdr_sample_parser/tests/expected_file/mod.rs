/*
 * Copyright 2022 The Regents of the University of California
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
//! A file with parser configuration and expected parse results
//!

use num_complex::Complex;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ExpectedFile {
    pub compressed_file: PathBuf,
    pub fft_size: u32,
    pub expected_windows: Vec<ExpectedWindowOrError>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ExpectedWindowOrError {
    Window {
        #[serde(flatten)]
        window: ExpectedWindow,
    },
    Error {
        error: EmptyError,
    },
}

#[derive(Debug, Deserialize)]
pub struct ExpectedWindow {
    pub time: u32,
    #[serde(flatten)]
    pub kind: ExpectedBins,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ExpectedBins {
    Fft { bins: Vec<Complex<i16>> },
    Average { averages: Vec<u32> },
}

#[derive(Debug, Deserialize)]
pub struct EmptyError {}
