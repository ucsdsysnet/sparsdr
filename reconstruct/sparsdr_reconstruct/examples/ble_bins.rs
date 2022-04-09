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

//!
//! Calculates bin numbers for Bluetooth Low Energy channels
//!
//! Usage: ble_bins channel-index
//!
//! This application assumes that the compressed data were recorded with 100 MHz of bandwidth
//! centered at 2.45 GHz.
//!
//! It assumes that bin 0 is the lowest frequency, bin 1023 is the center frequency, and bin
//! 2047 is the highest frequency
//!

// This is not really an example, but it is too small for its own package.

use std::env;
use std::process;

/// Bandwidth of a BLE channel, hertz
const BLE_BANDWIDTH: f64 = 2e6;
/// Total bandwidth of compressed data
const COMPRESSED_BANDWIDTH: f64 = 100e6;
/// Center frequency of compressed data
const COMPRESSED_CENTER: f64 = 2.45e9;
/// Number of bins
const BINS: u16 = 2048;

fn main() {
    let channel_index = env::args()
        .skip(1)
        .next()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or_else(|| {
            eprintln!("Usage: ble_bins channel-index");
            process::exit(-1);
        });
    let center_frequency = channel_to_frequency(channel_index).unwrap_or_else(|| {
        eprintln!("Invalid channel index");
        process::exit(-1);
    });

    let bin_count = (BLE_BANDWIDTH / COMPRESSED_BANDWIDTH * f64::from(BINS)).ceil() as u16;
    let compressed_low = COMPRESSED_CENTER - COMPRESSED_BANDWIDTH / 2.0;
    let center_bin =
        ((center_frequency - compressed_low) / COMPRESSED_BANDWIDTH * f64::from(BINS)) as u16;

    let low_bin = center_bin - bin_count / 2;
    let high_bin = center_bin + bin_count / 2 + 1;
    assert_eq!(high_bin - low_bin, bin_count);
    println!("{}..{}", low_bin, high_bin);
}

/// Converts a BLE channel index into a frequency in hertz
fn channel_to_frequency(channel_index: u16) -> Option<f64> {
    // Defined in BLE core specification volume 6 part B 1.4.1
    let rf_channel = match channel_index {
        0..=10 => channel_index + 1,
        11..=36 => channel_index + 2,
        37 => 0,
        38 => 12,
        39 => 39,
        _ => return None,
    };
    Some(2.402e9 + 2e6 * f64::from(rf_channel))
}
