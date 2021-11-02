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
//! This program analyzes a Pluto compressed sample and a file with output from a gr-bluetooth
//! multi_sniffer block.
//!
//! For each decoded Bluetooth or BLE packet, it calculates the corresponding range of compressed
//! samples and determines how many bins were active.
//!

extern crate image;
extern crate num_traits;
extern crate regex;
extern crate sparsdr_reconstruct;
extern crate sparsdr_sample_parser;
extern crate statrs;

use image::{ImageBuffer, Rgb, RgbImage};
use num_traits::Zero;
use regex::Regex;
use sparsdr_reconstruct::bins::BinRange;
use sparsdr_reconstruct::input::SampleReader;
use sparsdr_reconstruct::iter_ext::IterExt;
use sparsdr_reconstruct::window::{Logical, Status, Window};
use sparsdr_sample_parser::V1Parser;
use statrs::statistics::Statistics;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::ops::Range;
use std::{cmp, env};

/// Number of samples that represent one bit (at 1 megabit/second)
const SAMPLES_PER_BIT: f32 = COMPRESSED_SAMPLE_RATE / 1_000_000.0;
/// Sample rate of reconstructed samples (from a 64-bin FFT)
const COMPRESSED_SAMPLE_RATE: f32 = 3.84e6;
/// Sample rate of reconstructed signals, after remapping
///
/// Note: The sample numbers from the compressed file are still at 3.84 million samples/second.
const RECONSTRUCTED_SAMPLE_RATE: f32 = 3.125e6;

/// Number of bins used when receiving
const NATIVE_FFT_SIZE: u16 = 1024;
/// Number of bins used when reconstructing
const RECONSTRUCTION_FFT_SIZE: u16 = 64;

/// Range of bins representing the Bluetooth channel
const BINS: Range<u16> = 928..962;

fn main() -> Result<(), io::Error> {
    let mut args = env::args_os().skip(1);
    let compressed_path = args.next().expect("Expected a compressed sample file path");
    let decoded_path = args.next().expect("Expected a decoded packet file path");

    let packets = parse_decoded_packets(&decoded_path)?;

    // Read compressed samples (format version 1), handle time overflow, shift, and filter bins
    let compressed_file = BufReader::new(File::open(compressed_path)?);
    let parser = V1Parser::new_pluto(NATIVE_FFT_SIZE.into());
    let sample_reader = SampleReader::new(compressed_file, parser);
    let windows = sample_reader
        .shift_result(NATIVE_FFT_SIZE)
        .map(|window_result| Status::Ok(window_result.unwrap()))
        .filter_bins(BinRange::from(BINS), NATIVE_FFT_SIZE);

    // Calculate the number of samples that each window reconstructs to

    // The time (in 32-sample half windows) of the previous window
    let mut last_window_time: Option<u64> = None;
    // The number of output samples produced by all previous frames
    let mut output_samples: u64 = 0;

    let mut samples_to_active_bins = BTreeMap::new();
    let mut isolated_windows = 0u64;
    let mut overlapped_windows = 0u64;

    for window in windows {
        let window: Window<Logical> = match window {
            Status::Ok(window) => window,
            Status::Timeout => panic!("Timeout status"),
        };

        match last_window_time {
            None => {
                // This is the first window
                output_samples = RECONSTRUCTION_FFT_SIZE.into();
                isolated_windows += 1;
            }
            Some(time) if time.wrapping_add(1) == window.time() => {
                // Windows separated by one time step, so they overlap
                output_samples += u64::from(RECONSTRUCTION_FFT_SIZE / 2);
                overlapped_windows += 1;
            }
            Some(_time) => {
                // No overlap
                output_samples += u64::from(RECONSTRUCTION_FFT_SIZE);
                isolated_windows += 1;
            }
        }
        last_window_time = Some(window.time());

        samples_to_active_bins.insert(
            output_samples,
            AnalysisWindow {
                active_bins: BinMask::new(window.bins().iter().map(|bin| !bin.is_zero()).collect()),
                part_of_packet: false,
            },
        );
    }
    println!(
        "{} isolated windows, {} overlapping windows",
        isolated_windows, overlapped_windows
    );

    for packet in packets {
        let start_sample = packet.sample;
        let end_sample = start_sample
            .checked_add((packet.length_bits as f32 * SAMPLES_PER_BIT) as u64)
            .unwrap();
        let packet_duration_seconds =
            (end_sample - start_sample) as f32 / RECONSTRUCTED_SAMPLE_RATE;
        if !(44e-6..=2120e-6).contains(&packet_duration_seconds) {
            println!(
                "Warning: Packet duration {} seconds is outside expected range",
                packet_duration_seconds
            );
        }

        let range = samples_to_active_bins.range_mut(start_sample..=end_sample);
        // Mark all the affected windows as part of a packet
        for (_, window) in range {
            window.part_of_packet = true;
        }
        // Mark only the final window as part of a packet
        // if let Some((_, window)) = range.last() {
        //     window.part_of_packet = true;
        // }
    }

    let max_width = 100000_usize;
    {
        // Make a graphical representation of active bins and packets
        let mut image: RgbImage = ImageBuffer::new(
            cmp::min(
                samples_to_active_bins.len().try_into().unwrap(),
                max_width.try_into().unwrap(),
            ),
            // One row for each bin of interest, plus one for packet or not
            u32::from(BINS.end - BINS.start) + 1,
        );
        for (i, window) in samples_to_active_bins
            .values()
            .enumerate()
            .take(image.width() as usize)
        {
            let i: u32 = i.try_into().unwrap();
            // Row 0 is black for non-packet, white for packet
            let row = image.height() - 1;
            image[(i, row)] = if window.part_of_packet {
                Rgb([255, 255, 255])
            } else {
                Rgb([0, 0, 0])
            };
            for bin in BINS {
                let row = image.height() - 2 - u32::from(bin - BINS.start);
                let color = if window.active_bins.get(bin.into()) {
                    Rgb([0, 0, 255])
                } else {
                    Rgb([0, 0, 0])
                };
                image[(i, row)] = color;
            }
        }
        image.save("active_bins.png").unwrap();
    }

    if let Some((&last_samples, _)) = samples_to_active_bins.iter().next_back() {
        println!(
            "Last window at {} samples, {} bytes",
            last_samples,
            last_samples * 8
        );
        let last_samples_3mhz =
            (last_samples as f32 * RECONSTRUCTED_SAMPLE_RATE / COMPRESSED_SAMPLE_RATE) as u64;
        println!(
            "Last 3.125 MHz window at {} samples, {} bytes",
            last_samples_3mhz,
            last_samples_3mhz * 8
        );
    }

    // Statistics

    let packet_active_bin_numbers: Vec<f64> = samples_to_active_bins
        .values()
        .filter(|window| window.part_of_packet)
        .map(|window| window.active_bins.count_ones() as f64)
        .collect();
    let non_packet_active_bin_numbers: Vec<f64> = samples_to_active_bins
        .values()
        .filter(|window| !window.part_of_packet)
        .map(|window| window.active_bins.count_ones() as f64)
        .collect();

    let packet_consecutive_active_bin_numbers: Vec<f64> = samples_to_active_bins
        .values()
        .filter(|window| window.part_of_packet)
        .map(|window| window.active_bins.count_consecutive_active() as f64)
        .collect();
    let non_packet_consecutive_active_bin_numbers: Vec<f64> = samples_to_active_bins
        .values()
        .filter(|window| !window.part_of_packet)
        .map(|window| window.active_bins.count_consecutive_active() as f64)
        .collect();
    println!(
        "{} windows part of packets, {} not",
        packet_active_bin_numbers.len(),
        non_packet_active_bin_numbers.len()
    );

    print_statistics("packet active bins", &packet_active_bin_numbers);
    print_statistics("non-packet active bins", &non_packet_active_bin_numbers);
    print_statistics(
        "packet consecutive active bins",
        &packet_consecutive_active_bin_numbers,
    );
    print_statistics(
        "non-packet consecutive active bins",
        &non_packet_consecutive_active_bin_numbers,
    );

    Ok(())
}

fn print_statistics(name: &str, values: &[f64]) {
    println!(
        "{}: min {:.0} max {:.0} mean {:.2} standard deviation {:.2}",
        name,
        Statistics::min(values),
        Statistics::max(values),
        values.mean(),
        values.population_std_dev(),
    )
}

struct AnalysisWindow {
    active_bins: BinMask,
    part_of_packet: bool,
}

#[derive(Debug)]
struct DecodedPacket {
    /// Index of the reconstructed sample around the end of the packet
    sample: u64,
    /// Length of the packet, in bits
    length_bits: u32,
}

fn parse_decoded_packets(path: &OsStr) -> Result<Vec<DecodedPacket>, io::Error> {
    // Example line: "Start of packet 543321 samples in, length 37 octets"
    let pattern = Regex::new(
        r"(?m)^Start of packet (?P<samples>\d+) samples in, length (?P<length>\d+) octets$",
    )
    .unwrap();
    let content = fs::read_to_string(path)?;

    Ok(pattern
        .captures_iter(&content)
        .map(|captures| {
            let samples: u64 = captures["samples"].parse().unwrap();
            let length_octets: u32 = captures["length"].parse().unwrap();
            // How the length works for BLE advertising packets: The length field is the number of
            // octets in the PDU payload.
            // Before the PDU payload are 2 octets of PDU header, 4 octets of access address, and
            // 1 octet of preamble (for the 1M PHY).
            // After the PDU payload is 3 octets of CRC.
            let length_bits = 8 * (1 + 4 + 2 + length_octets + 2);

            // Scale down samples and length to match 3.84 million samples/second sample rate
            let samples =
                (samples as f32 * COMPRESSED_SAMPLE_RATE / RECONSTRUCTED_SAMPLE_RATE) as u64;

            DecodedPacket {
                sample: samples,
                length_bits,
            }
        })
        .collect())
}

struct BinMask {
    bins: Vec<bool>,
}

impl BinMask {
    pub fn new(bins: Vec<bool>) -> Self {
        BinMask { bins }
    }

    pub fn get(&self, index: usize) -> bool {
        self.bins[index]
    }

    pub fn count_ones(&self) -> usize {
        self.bins.iter().filter(|active| **active).count()
    }
    pub fn count_consecutive_active(&self) -> usize {
        let mut consecutive_active = 0;
        let mut consecutive_active_this_sequence = 0;
        let mut prev_active = false;

        for &bin in self.bins.iter() {
            if bin {
                if prev_active {
                    // Second or subsequent consecutive active bin
                    consecutive_active_this_sequence += 1;
                } else {
                    // First active bin
                    consecutive_active_this_sequence = 1;
                }
                consecutive_active = cmp::max(consecutive_active, consecutive_active_this_sequence);
            }
            prev_active = bin;
        }

        consecutive_active
    }
}
