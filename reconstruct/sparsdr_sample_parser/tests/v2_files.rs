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

extern crate num_complex;
extern crate serde;
extern crate serde_json;

use crate::expected_file::{ExpectedBins, ExpectedFile, ExpectedWindow, ExpectedWindowOrError};
use sparsdr_sample_parser::{Parser, V2Parser, Window, WindowKind};
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read};
use std::path::Path;

mod expected_file;

#[test]
fn v2_files() -> Result<(), Box<dyn std::error::Error>> {
    let json_extension = OsString::from("json");

    let mut last_error: Option<Box<dyn std::error::Error>> = None;

    for entry in fs::read_dir("test-data/v2")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension() == Some(&json_extension) {
            eprintln!("Running test with file {:?}", path);
            let file_status = run_test_with_file(&path);

            match file_status {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("Error with test file {:?}: {}", path, e);
                    last_error = Some(e);
                }
            }
        }
    }

    match last_error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

fn run_test_with_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let expected: ExpectedFile = {
        let file = BufReader::new(File::open(path)?);
        serde_json::from_reader(file)?
    };
    // Resolve the compressed file path relative to the expected file
    let resolved_compressed_path = path
        .parent()
        .expect("Expected file path has no parent directory")
        .join(expected.compressed_file);
    let mut compressed_file = BufReader::new(File::open(&resolved_compressed_path)?);
    let mut parser = V2Parser::new(expected.fft_size);

    let mut expected_windows = expected.expected_windows.into_iter();
    loop {
        let mut bytes = [0u8; 4];
        match compressed_file.read_exact(&mut bytes) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        let parse_result = parser.parse(&bytes);
        match parse_result {
            Ok(Some(actual_window)) => {
                let expected_parse_result = expected_windows
                    .next()
                    .ok_or_else(|| "Extra window parsed at end")?;
                match expected_parse_result {
                    ExpectedWindowOrError::Window(expected_window) => {
                        check_windows_equal(&actual_window, &expected_window)?
                    }
                    ExpectedWindowOrError::Error { .. } => {
                        return Err("Expected error but got parsed window".into());
                    }
                }
            }
            Err(parse_error) => {
                let expected_parse_result = expected_windows
                    .next()
                    .ok_or_else(|| "Extra parse error at end")?;
                match expected_parse_result {
                    ExpectedWindowOrError::Window(_) => {
                        return Err(
                            format!("Expected window but got parse error {}", parse_error).into(),
                        )
                    }
                    ExpectedWindowOrError::Error { .. } => { /* OK */ }
                }
            }
            Ok(None) => {}
        }
    }
    match expected_windows.next() {
        Some(_) => return Err("Extra expected window at end, missing from file".into()),
        None => {}
    }

    Ok(())
}

fn check_windows_equal(
    actual_window: &Window,
    expected_window: &ExpectedWindow,
) -> Result<(), Box<dyn std::error::Error>> {
    if actual_window.timestamp != expected_window.time {
        return Err(format!(
            "Timestamp mismatch: expected {}, actual {}",
            expected_window.time, actual_window.timestamp
        )
        .into());
    }
    match (&actual_window.kind, &expected_window.kind) {
        (
            WindowKind::Data(actual_bins),
            ExpectedBins::Fft {
                bins: expected_bins,
            },
        ) => {
            if actual_bins.len() != expected_bins.len() {
                return Err(format!(
                    "FFT window number of bins mismatch: expected {}, actual {}",
                    expected_bins.len(),
                    actual_bins.len()
                )
                .into());
            }
            for (i, (actual_bin, expected_bin)) in
                actual_bins.iter().zip(expected_bins.iter()).enumerate()
            {
                if actual_bin != expected_bin {
                    return Err(format!(
                        "FFT bin mismatch at index {}: expected {}, actual {}",
                        i, expected_bin, actual_bin
                    )
                    .into());
                }
            }
        }
        (
            WindowKind::Average(actual_averages),
            ExpectedBins::Average {
                averages: expected_averages,
            },
        ) => {
            if actual_averages.len() != expected_averages.len() {
                return Err(format!(
                    "Average window number of bins mismatch: expected {}, actual {}",
                    actual_averages.len(),
                    expected_averages.len()
                )
                .into());
            }
            for (i, (actual_average, expected_average)) in actual_averages
                .iter()
                .zip(expected_averages.iter())
                .enumerate()
            {
                if actual_average != expected_average {
                    return Err(format!(
                        "Average value mismatch at index {}: expected {}, actual {}",
                        i, expected_average, actual_average
                    )
                    .into());
                }
            }
        }
        (
            WindowKind::Average(actual_averages),
            ExpectedBins::Fft {
                bins: expected_bins,
            },
        ) => {
            return Err(format!(
                "Expected FFT window {:?}, got averages {:?}",
                expected_bins, actual_averages
            )
            .into())
        }
        (
            WindowKind::Data(actual_bins),
            ExpectedBins::Average {
                averages: expected_averages,
            },
        ) => {
            return Err(format!(
                "Expected average window {:?}, got FFT window {:?}",
                expected_averages, actual_bins
            )
            .into())
        }
    }
    Ok(())
}
