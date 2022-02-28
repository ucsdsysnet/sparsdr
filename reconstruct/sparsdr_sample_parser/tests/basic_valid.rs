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

//!
//! Simple tests for the v2 parser
//!

extern crate num_complex;
extern crate simplelog;
extern crate sparsdr_sample_parser;

use num_complex::Complex;
use sparsdr_sample_parser::{ParseError, Parser, V2Parser, Window, WindowKind};
use std::sync::Once;

const HEADER_BIT: u32 = 0x80000000;
const HEADER_AVERAGE_BIT: u32 = 0x40000000;
/// The maximum allowed timestamp (30 bits)
const MAX_TIME: u32 = 0x3fffffff;

static LOG_INIT: Once = Once::new();
fn log_init() {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Warn,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
    )
    .unwrap();
}

fn make_complex(re: i16, imag: i16) -> u32 {
    // Real part at higher address (more significant)
    ((re as u16 as u32) << 16) | (imag as u16 as u32)
}

#[test]
fn basic_1_bin() {
    LOG_INIT.call_once(log_init);

    let mut parser = V2Parser::new(1);
    // Mandatory zero at beginning
    assert_eq!(Ok(None), parser.accept(0));
    // Begin FFT window
    assert_eq!(Ok(None), parser.accept(HEADER_BIT | 37));
    // Bin index 0
    assert_eq!(Ok(None), parser.accept(0));
    // Value at index 0
    assert_eq!(Ok(None), parser.accept(make_complex(10, 20)));
    // End of bin group
    assert_eq!(Ok(None), parser.accept(0));
    // Beginning of next FFT window, current window is done
    assert_eq!(
        Ok(Some(Window {
            timestamp: 37,
            kind: WindowKind::Data(vec![Complex::new(10, 20)])
        })),
        parser.accept(HEADER_BIT | 38)
    );
    // Bin index 0
    assert_eq!(Ok(None), parser.accept(0));
    // Value at index 0
    assert_eq!(Ok(None), parser.accept(make_complex(932, -9921)));
    // End of bin group
    assert_eq!(Ok(None), parser.accept(0));
    // Beginning of next average window, current window is done
    assert_eq!(
        Ok(Some(Window {
            timestamp: 38,
            kind: WindowKind::Data(vec![Complex::new(932, -9921)])
        })),
        parser.accept(HEADER_BIT | HEADER_AVERAGE_BIT | 39)
    );
    // Average value
    assert_eq!(Ok(None), parser.accept(10000003));
    // End of averages
    assert_eq!(
        Ok(Some(Window {
            timestamp: 39,
            kind: WindowKind::Average(vec![10000003])
        })),
        parser.accept(0)
    );
}

#[test]
fn basic_2_bins() {
    LOG_INIT.call_once(log_init);

    let mut parser = V2Parser::new(2);
    // Mandatory zero at beginning
    assert_eq!(Ok(None), parser.accept(0));
    // Begin FFT window
    assert_eq!(Ok(None), parser.accept(HEADER_BIT | 37));
    // Bin index 0
    assert_eq!(Ok(None), parser.accept(0));
    // Value at index 0
    assert_eq!(Ok(None), parser.accept(make_complex(10, 20)));
    // Value at index 1
    assert_eq!(Ok(None), parser.accept(make_complex(30, 40)));
    // End of bin group
    assert_eq!(Ok(None), parser.accept(0));
    // Beginning of next FFT window, current window is done
    assert_eq!(
        Ok(Some(Window {
            timestamp: 37,
            kind: WindowKind::Data(vec![Complex::new(10, 20), Complex::new(30, 40)])
        })),
        parser.accept(HEADER_BIT | 38)
    );
    // Bin index 0
    assert_eq!(Ok(None), parser.accept(0));
    // Value at index 0
    assert_eq!(Ok(None), parser.accept(make_complex(50, 60)));
    // End this group and start a new group at index 1
    assert_eq!(Ok(None), parser.accept(0));
    assert_eq!(Ok(None), parser.accept(1));
    // Value at index 1
    assert_eq!(Ok(None), parser.accept(make_complex(70, 80)));
    // End of bin group
    assert_eq!(Ok(None), parser.accept(0));
    // Beginning of next average window, current window is done
    assert_eq!(
        Ok(Some(Window {
            timestamp: 38,
            kind: WindowKind::Data(vec![Complex::new(50, 60), Complex::new(70, 80)])
        })),
        parser.accept(HEADER_BIT | HEADER_AVERAGE_BIT | 39)
    );
    assert_eq!(Ok(None), parser.accept(1200));
    assert_eq!(Ok(None), parser.accept(9000));
    // End of averages
    assert_eq!(
        Ok(Some(Window {
            timestamp: 39,
            kind: WindowKind::Average(vec![1200, 9000])
        })),
        parser.accept(0)
    );
    // Beginning of next FFT window
    assert_eq!(Ok(None), parser.accept(HEADER_BIT | 40));
    // Skip to bin 1
    assert_eq!(Ok(None), parser.accept(1));
    // Value at index 1
    assert_eq!(Ok(None), parser.accept(make_complex(90, 100)));
    // End of group
    assert_eq!(Ok(None), parser.accept(0));
    // Beginning of next FFT window, current window is done
    assert_eq!(
        Ok(Some(Window {
            timestamp: 40,
            kind: WindowKind::Data(vec![Complex::new(0, 0), Complex::new(90, 100)])
        })),
        parser.accept(HEADER_BIT | 41)
    );
    // Start group at index 0
    assert_eq!(Ok(None), parser.accept(0));
    // Value at index 0
    assert_eq!(Ok(None), parser.accept(make_complex(110, 120)));
    // End of group
    assert_eq!(Ok(None), parser.accept(0));
    // Beginning of next FFT window, current window is done (value 0 assumed for bin 1)
    assert_eq!(
        Ok(Some(Window {
            timestamp: 41,
            kind: WindowKind::Data(vec![Complex::new(110, 120), Complex::new(0, 0)])
        })),
        parser.accept(HEADER_BIT | 42)
    );
}

#[test]
fn basic_8_bins() {
    LOG_INIT.call_once(log_init);
    let mut parser = V2Parser::new(8);
    // Mandatory zero at beginning
    assert_eq!(Ok(None), parser.accept(0));
    // Begin FFT window
    assert_eq!(Ok(None), parser.accept(HEADER_BIT | MAX_TIME));
    // Two values at indexes 2 and 3
    assert_eq!(Ok(None), parser.accept(2));
    assert_eq!(Ok(None), parser.accept(make_complex(32066, -32768)));
    assert_eq!(Ok(None), parser.accept(make_complex(-9999, 31000)));
    assert_eq!(Ok(None), parser.accept(0));
    // Two values at indexes 5 and 6
    assert_eq!(Ok(None), parser.accept(5));
    assert_eq!(Ok(None), parser.accept(make_complex(120, 121)));
    assert_eq!(Ok(None), parser.accept(make_complex(37, -31000)));
    assert_eq!(Ok(None), parser.accept(0));
    // Beginning of next average window, current window is done
    let expected_bins = vec![
        Complex::new(0, 0),
        Complex::new(0, 0),
        Complex::new(32066, -32768),
        Complex::new(-9999, 31000),
        Complex::new(0, 0),
        Complex::new(120, 121),
        Complex::new(37, -31000),
        Complex::new(0, 0),
    ];
    assert_eq!(
        Ok(Some(Window {
            timestamp: MAX_TIME,
            kind: WindowKind::Data(expected_bins)
        })),
        parser.accept(HEADER_BIT | HEADER_AVERAGE_BIT | 0)
    );
}

/// There's a cut-off end of a window at the beginning of the file, and the real header is not until
/// later. The parser should ignore all samples before the first [zero, header] sequence.
#[test]
fn header_not_at_beginning() {
    LOG_INIT.call_once(log_init);
    let mut parser = V2Parser::new(1024);
    // The last zero is required to make the real header recognized as a header
    let cutoff_window_samples: [u32; 19] = [
        0x00000000, 0x00000194, 0x000e0031, 0xffc8ffb3, 0x0058001c, 0xff420079, 0x0126ffaf,
        0x007b00d8, 0xfefcfca0, 0x005601d4, 0xfb1501f9, 0x08f7fc66, 0xfae601e6, 0x017b031e,
        0xfe62fc29, 0x00b80146, 0x002bffb4, 0xffe60007, 0x00000000,
    ];
    for sample in cutoff_window_samples {
        assert_eq!(Ok(None), parser.accept(sample));
    }
    // The real samples, including the zero at the end
    let real_window_samples: [u32; 7] = [
        0xb1472ba5, 0x0000005c, 0x00300037, 0x0015ffd1, 0x000b0033, 0xffe00040, 0x00000000,
    ];
    for sample in real_window_samples {
        assert_eq!(Ok(None), parser.accept(sample));
    }
    let expected_window = Window {
        timestamp: 826747813,
        kind: WindowKind::Data({
            let mut bins = vec![Complex::default(); 1024];
            bins[92] = Complex::new(0x0030, 0x0037);
            bins[93] = Complex::new(0x0015, 0xffd1_u16 as i16);
            bins[94] = Complex::new(0x000b, 0x0033);
            bins[95] = Complex::new(0xffe0_u16 as i16, 0x0040);
            bins
        }),
    };

    // Parse the header of the next window to produce the current window
    let actual_window = parser.accept(0xb1472ba6).unwrap().unwrap();
    assert_eq!(expected_window.timestamp, actual_window.timestamp);
    match (expected_window.kind, actual_window.kind) {
        (WindowKind::Data(expected_bins), WindowKind::Data(actual_bins)) => {
            assert_bins_equal(&expected_bins, &actual_bins);
        }
        _ => panic!("Non-matching window kinds"),
    }
}

trait TestParserExt {
    fn accept(&mut self, sample: u32) -> Result<Option<Window>, ParseError>;
}
impl TestParserExt for V2Parser {
    fn accept(&mut self, sample: u32) -> Result<Option<Window>, ParseError> {
        self.parse(&sample.to_le_bytes())
    }
}

fn assert_bins_equal(bins1: &[Complex<i16>], bins2: &[Complex<i16>]) {
    assert_eq!(bins1.len(), bins2.len());

    let mut any_mismatch = false;
    for (i, (bin1, bin2)) in bins1.iter().zip(bins2.iter()).enumerate() {
        if bin1 != bin2 {
            println!("Mismatch at index {}: {} != {}", i, bin1, bin2);
            any_mismatch = true;
        }
    }
    assert!(!any_mismatch);
}
