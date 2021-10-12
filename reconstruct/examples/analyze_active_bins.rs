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

//! Reads samples from a compressed file and analyzes things about bins that are active

extern crate num_traits;
extern crate sparsdr_reconstruct;

use num_traits::Zero;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::ops::Range;

use sparsdr_reconstruct::input::iqzip::CompressedSamples;
use sparsdr_reconstruct::iter_ext::IterExt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let path = args.next().expect("No path");
    let threshold: f32 = args
        .next()
        .map(|arg| arg.to_str().unwrap().parse().unwrap())
        .unwrap_or(0.0);
    println!("Threshold {}", threshold);

    let file = BufReader::new(File::open(path)?);

    // Interested in Bluetooth channel 40 (2.442 GHz)
    let channel_bins: Range<usize> = 347..411;
    // Shift into FFT bin order
    let channel_bins_fft_order: Range<usize> = channel_bins.start + 512..channel_bins.end + 512;

    let mut activation_start_time: Option<u64> = None;
    // Keep track of the number of 512-sample half windows
    let mut activation_durations: Vec<u64> = Vec::new();
    // Keep track of the number of bins in the channel that are active
    let mut active_bin_counts: Vec<usize> = Vec::new();

    for window in CompressedSamples::new(file).group(1024) {
        let window = window?;

        let channel_bins = &window.bins()[channel_bins_fft_order.clone()];
        let active_bin_count = channel_bins.iter().filter(|bin| !bin.is_zero()).count();
        if active_bin_count != 0 {
            // Something is active
            match activation_start_time {
                Some(_) => {}
                None => activation_start_time = Some(window.time()),
            }
        } else {
            // Nothing is active
            match activation_start_time {
                Some(start_time) => {
                    assert!(window.time() > start_time);
                    activation_durations.push(window.time() - start_time);
                    activation_start_time = None;
                }
                None => {}
            }
        }
        if active_bin_count != 0 {
            active_bin_counts.push(active_bin_count);
        }
    }

    // println!("{:?}", activation_durations);
    activation_durations.sort_unstable();
    println!(
        "Durations range from {:?} to {:?} half-windows",
        activation_durations.first(),
        activation_durations.last()
    );

    // Generate activation duration CDF CSV
    println!("Duration,RatioOfActivations");
    for (i, duration) in activation_durations.windows(2).enumerate() {
        let ratio = i as f32 / activation_durations.len() as f32;
        if duration[0] != duration[1] {
            println!("{},{}", duration[0], ratio);
        }
    }

    // Generate number of bins CDF CSV
    active_bin_counts.sort_unstable();
    println!("========");
    println!("NumberOfBinsActive,RatioOfWindows");
    for (i, active_bin_count) in active_bin_counts.windows(2).enumerate() {
        let ratio = i as f32 / active_bin_counts.len() as f32;
        if active_bin_count[0] != active_bin_count[1] {
            println!("{},{}", active_bin_count[0], ratio);
        }
    }

    Ok(())
}
