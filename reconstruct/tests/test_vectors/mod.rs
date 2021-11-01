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

mod uncompressed;

use byteorder::{ByteOrder, LittleEndian};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, ErrorKind, Read, Result, Seek, SeekFrom, Write};
use std::path::Path;

use num_complex::Complex32;
use sparsdr_reconstruct::window::Window;
use sparsdr_reconstruct::{decompress, BandSetupBuilder, DecompressSetup};

use self::uncompressed::SAMPLE_BYTES;

const COMPRESSED_BANDWIDTH: f32 = 100e6;
const COMPRESSION_FFT_SIZE: usize = 2048;
const TIMESTAMP_BITS: u32 = 32;

/// Creates a decompressor and tests it on some test vectors
///
/// input_path: The path to a compressed file containing the input data to decompress
///
/// expected_path: The path to a decompressed file containing the expected decompressed data
///
/// actual_path: The path where the actual output will be written
pub fn test_with_vectors<P1, P2, P3>(
    input_path: P1,
    expected_path: P2,
    actual_path: P3,
    center_frequency: f32,
    bins: u16,
) where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    P3: AsRef<Path>,
{
    let input_file = File::open(input_path).expect("Failed to open input file");
    let input_file = BufReader::new(input_file);
    let expected_file = File::open(&expected_path).expect("Failed to open expected file");
    let expected_file = BufReader::new(expected_file);
    let mut output_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(&actual_path)
        .expect("Failed to create actual file");
    eprintln!("Saving output to {}", actual_path.as_ref().display());

    {
        // Read samples in the MATLAB format
        let windows_in = MatlabWindows::new(input_file);
        let mut output_file = BufWriter::new(&mut output_file);

        let band_setup = BandSetupBuilder::new(
            Box::new(&mut output_file),
            COMPRESSED_BANDWIDTH,
            COMPRESSION_FFT_SIZE,
            bins,
        )
        .center_frequency(center_frequency);
        let mut setup = DecompressSetup::new(windows_in, COMPRESSION_FFT_SIZE, TIMESTAMP_BITS);
        setup.add_band(band_setup.build());

        let info = decompress(setup).expect("Decompress failed");

        eprintln!("{:?}", info);
    }
    output_file.flush().expect("Failed to flush output");

    // Seek back to the beginning of the output to compare
    output_file
        .seek(SeekFrom::Start(0))
        .expect("Output seek failed");

    // Check file sizes and sample counts
    let expected_size = file_size(expected_path).expect("Failed to get expected file size");
    let output_size = file_size(actual_path).expect("Failed to get expected file size");
    if expected_size != output_size {
        println!(
            "Actual output {} bytes ({} samples) does not match expected {} bytes ({} samples)",
            output_size,
            output_size / SAMPLE_BYTES as u64,
            expected_size,
            expected_size / SAMPLE_BYTES as u64
        );
    }

    compare_output(expected_file, output_file);
}

fn file_size<P>(path: P) -> Result<u64>
where
    P: AsRef<Path>,
{
    fs::metadata(path).map(|metadata| metadata.len())
}

/// Compares the uncompressed samples in two files (or other byte sources)
///
/// Panics of the contents of the files are not equal
fn compare_output<W1, W2>(expected: W1, actual: W2)
where
    W1: Read,
    W2: Read,
{
    let mut expected = uncompressed::Samples::new(expected);
    let mut actual = uncompressed::Samples::new(actual);

    for i in 0usize.. {
        let expected_sample = expected.next();
        let actual_sample = actual.next();

        let (expected_sample, actual_sample) = match (expected_sample, actual_sample) {
            (Some(Ok(expected)), Some(Ok(actual))) => (expected, actual),
            (None, None) => break,
            (Some(Err(e)), _) => {
                panic!("Error reading from expected sample file: {}", e);
            }
            (_, Some(Err(e))) => {
                panic!("Error reading from actual sample file: {}", e);
            }
            (None, Some(_)) => {
                panic!("Expected sample file ended before actual sample file");
            }
            (Some(_), None) => {
                panic!("Actual sample file ended before expected sample file");
            }
        };

        if !sample_approx_equal(&expected_sample, &actual_sample) {
            panic!(
                "At sample index {}, samples not equal: expected {}, actual {}",
                i, expected_sample, actual_sample
            );
        }
    }
}

fn sample_approx_equal(s1: &Complex32, s2: &Complex32) -> bool {
    // Threshold experimentally determined to be close enough. This is as close as the current
    // implementation can get in the worst case.
    const THRESHOLD: f32 = 2.5e-3;
    f32::hypot(s1.re - s2.re, s1.im - s2.im) < THRESHOLD
}

/// Reads compressed data from binary files produced by MATLAB
///
/// Each file contains zero or more chunks of 2048 complex amplitude values.
///
/// Each chunk contains 2048 real values, followed by 2048 imaginary values. Each of these is a
/// 64-bit floating-point number in little-endian byte order
///
/// Chunks are assumed to have sequential time values.
struct MatlabWindows<R> {
    source: R,
    real_values: Vec<f32>,
    imaginary_values: Vec<f32>,
    timestamp: u32,
}

impl<R> MatlabWindows<R> {
    pub fn new(source: R) -> Self {
        MatlabWindows {
            source,
            real_values: Vec::with_capacity(COMPRESSION_FFT_SIZE),
            imaginary_values: Vec::with_capacity(COMPRESSION_FFT_SIZE),
            timestamp: 0,
        }
    }
}

impl<R> Iterator for MatlabWindows<R>
where
    R: Read,
{
    type Item = Result<Window>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0u8; 8];
        loop {
            match self.source.read_exact(&mut buffer) {
                Ok(()) => {
                    let value = LittleEndian::read_f64(&buffer) as f32;
                    if self.real_values.len() == COMPRESSION_FFT_SIZE {
                        // Add imaginary
                        self.imaginary_values.push(value);
                        if self.imaginary_values.len() == COMPRESSION_FFT_SIZE {
                            // Assemble a window from the collected samples
                            let complex_bins = self
                                .real_values
                                .drain(..)
                                .zip(self.imaginary_values.drain(..))
                                .map(|(real, imaginary)| Complex32::new(real, imaginary));
                            let window = Window::with_bins(
                                self.timestamp.into(),
                                COMPRESSION_FFT_SIZE,
                                complex_bins,
                            );
                            self.timestamp = self.timestamp.wrapping_add(1);
                            break Some(Ok(window));
                        }
                    } else {
                        self.real_values.push(value);
                    }
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => break None,
                Err(e) => break Some(Err(e)),
            }
        }
    }
}
