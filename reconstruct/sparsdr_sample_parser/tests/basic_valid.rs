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

extern crate num_complex;
extern crate simplelog;
extern crate sparsdr_sample_parser;

use num_complex::Complex;
use sparsdr_sample_parser::{Parser, Window, WindowKind};
use std::sync::Once;

const HEADER_BIT: u32 = 0x80000000;
const HEADER_FFT_BIT: u32 = 0x40000000;
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
    // Imaginary part at higher address (more significant)
    ((imag as u16 as u32) << 16) | (re as u16 as u32)
}

#[test]
fn basic_1_bin() {
    LOG_INIT.call_once(log_init);

    let mut parser = Parser::new(1);
    // Begin FFT window
    assert_eq!(Ok(None), parser.accept(HEADER_BIT | HEADER_FFT_BIT | 37));
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
        parser.accept(HEADER_BIT | HEADER_FFT_BIT | 38)
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
        parser.accept(HEADER_BIT | 39)
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

    let mut parser = Parser::new(2);
    // Begin FFT window
    assert_eq!(Ok(None), parser.accept(HEADER_BIT | HEADER_FFT_BIT | 37));
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
        parser.accept(HEADER_BIT | HEADER_FFT_BIT | 38)
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
        parser.accept(HEADER_BIT | 39)
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
    assert_eq!(Ok(None), parser.accept(HEADER_BIT | HEADER_FFT_BIT | 40));
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
        parser.accept(HEADER_BIT | HEADER_FFT_BIT | 41)
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
        parser.accept(HEADER_BIT | HEADER_FFT_BIT | 42)
    );
}

#[test]
fn basic_8_bins() {
    LOG_INIT.call_once(log_init);
    let mut parser = Parser::new(8);
    // Begin FFT window
    assert_eq!(
        Ok(None),
        parser.accept(HEADER_BIT | HEADER_FFT_BIT | MAX_TIME)
    );
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
        parser.accept(HEADER_BIT | 0)
    );
}
