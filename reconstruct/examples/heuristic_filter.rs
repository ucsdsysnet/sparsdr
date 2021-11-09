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
//! Reads compressed samples (Pluto v1 format) from standard input, filters them according to
//! a configurable heuristic, and writes the selected compressed samples to standard output
//!
//! Usage: heuristic_filter heuristic_name
//!

extern crate num_complex;
extern crate num_traits;
extern crate sparsdr_reconstruct;
extern crate sparsdr_sample_parser;

use num_complex::Complex;
use num_traits::Zero;
use sparsdr_sample_parser::{Parser, V1Parser, Window, WindowKind};
use std::cmp;
use std::collections::VecDeque;
use std::env;
use std::io::{self, BufReader, BufWriter, ErrorKind, Read, Result, Write};
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

fn main() -> Result<()> {
    let min_active = env::args()
        .nth(1)
        .expect("No min_active_bins")
        .parse()
        .unwrap();
    let stdin = io::stdin();
    let mut stdin = BufReader::new(stdin.lock());
    let stdout = io::stdout();
    let stdout = BufWriter::new(stdout.lock());
    let mut parser = V1Parser::new_pluto(1024);
    let mut samples_out = SampleWriter::new(stdout);

    let buffer_size = 4;
    // This stores up to buffer_size consecutive windows that the heuristic did not consider part
    // of a packet. The oldest window is at the front, and the newest is at the back.
    let mut window_buffer: VecDeque<Window> = VecDeque::with_capacity(buffer_size);
    // Heuristic things:
    // If the newest window (at the back) looks like part of a packet, accept it and all preceding
    // windows that are consecutive (overlapping).
    // Then, keep processing until there's a gap.
    let mut heuristic = SimpleActiveBinsHeuristic::new(min_active.., ..);

    loop {
        let window = loop {
            let mut buffer = [0u8; 8];
            match stdin.read_exact(&mut buffer) {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(()),
                Err(e) => return Err(e),
            }

            match parser.parse(&buffer) {
                Ok(Some(window)) => break window,
                Ok(None) => {}
                Err(_) => eprintln!("Sample parse error"),
            }
        };

        match &window.kind {
            WindowKind::Average(_) => continue,
            WindowKind::Data(_bins) => {}
        }

        if heuristic.is_packet(&window) {
            // Send all buffered windows if the current window is 1 time step after the back
            // (newest window) of the buffer, then send the current window
            let send_buffer = match window_buffer.back() {
                Some(back) if back.timestamp + 1 == window.timestamp => true,
                _ => false,
            };
            if send_buffer {
                // Iterate from front to back
                for buffered_window in window_buffer.drain(..).rev() {
                    // Write out window
                    match samples_out.write_window(&buffered_window) {
                        Ok(()) => {}
                        Err(e) if e.kind() == ErrorKind::BrokenPipe => return Ok(()),
                        Err(e) => return Err(e),
                    }
                }
            } else {
                window_buffer.clear();
            }
            // Write out window
            match samples_out.write_window(&window) {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::BrokenPipe => return Ok(()),
                Err(e) => return Err(e),
            }
        } else {
            match window_buffer.back() {
                None => window_buffer.push_back(window),
                Some(back) if back.timestamp + 1 == window.timestamp => {
                    window_buffer.push_back(window)
                }
                _ => {}
            }
        }
    }
}

struct SampleWriter<W> {
    byte_sink: W,
}

impl<W> SampleWriter<W>
where
    W: Write,
{
    pub fn new(byte_sink: W) -> Self {
        SampleWriter { byte_sink }
    }
    pub fn write_window(&mut self, window: &Window) -> Result<()> {
        // Sample format (64 bits, little-endian):
        // Average sample:

        let common_bits = u64::from(window.timestamp & 0x1fffff) << 32;
        match &window.kind {
            WindowKind::Average(averages) => {
                // Always write all the averages
                for (i, &average) in averages.iter().enumerate() {
                    let sample =
                        common_bits | (1 << 63) | ((i & 0x3ff) as u64) << 53 | u64::from(average);
                    self.byte_sink.write_all(&sample.to_le_bytes())?;
                }
            }
            WindowKind::Data(data) => {
                for (i, bin) in data.iter().enumerate() {
                    if !bin.is_zero() {
                        let sample = common_bits
                            | ((i & 0x3ff) as u64) << 53
                            | (bin.re as u16 as u64) << 16
                            | (bin.im as u16 as u64);
                        self.byte_sink.write_all(&sample.to_le_bytes())?;
                    }
                }
            }
        }
        Ok(())
    }
}

trait PacketHeuristic {
    //noinspection RsSelfConvention
    fn is_packet(&mut self, window: &Window) -> bool;
}

impl<T> PacketHeuristic for Box<T>
where
    T: PacketHeuristic,
{
    fn is_packet(&mut self, window: &Window) -> bool {
        <T as PacketHeuristic>::is_packet(&mut *self, window)
    }
}

struct AcceptAllHeuristic;
impl PacketHeuristic for AcceptAllHeuristic {
    fn is_packet(&mut self, _window: &Window) -> bool {
        true
    }
}

/// A simple heuristic that detects a packet when some number of bins is active, until a gap
struct SimpleActiveBinsHeuristic<A, C> {
    last_active_time: Option<u32>,
    active_bins: A,
    consecutive_active_bins: C,
}

impl<A, C> SimpleActiveBinsHeuristic<A, C> {
    pub fn new(active_bins: A, consecutive_active_bins: C) -> Self {
        SimpleActiveBinsHeuristic {
            last_active_time: None,
            active_bins,
            consecutive_active_bins,
        }
    }
}

impl<A, C> PacketHeuristic for SimpleActiveBinsHeuristic<A, C>
where
    A: RangeContains<usize>,
    C: RangeContains<usize>,
{
    fn is_packet(&mut self, window: &Window) -> bool {
        match &window.kind {
            WindowKind::Data(bins) => {
                match &self.last_active_time {
                    None => {
                        let bin_count = count_active_bins(bins);
                        let consecutive_bin_count = count_consecutive_active_bins(bins);
                        if self.active_bins.contains(&bin_count)
                            && self
                                .consecutive_active_bins
                                .contains(&consecutive_bin_count)
                        {
                            self.last_active_time = Some(window.timestamp);
                            true
                        } else {
                            false
                        }
                    }
                    Some(last_active_time) => {
                        let gap = window.timestamp - *last_active_time;
                        if gap == 1 {
                            // Overlapped windows
                            self.last_active_time = Some(window.timestamp);
                            true
                        } else {
                            self.last_active_time = None;
                            false
                        }
                    }
                }
            }
            WindowKind::Average(_) => false,
        }
    }
}

trait RangeContains<T> {
    fn contains(&self, value: &T) -> bool;
}

impl RangeContains<usize> for Range<usize> {
    fn contains(&self, value: &usize) -> bool {
        Self::contains(self, value)
    }
}
impl RangeContains<usize> for RangeFrom<usize> {
    fn contains(&self, value: &usize) -> bool {
        Self::contains(self, value)
    }
}
impl RangeContains<usize> for RangeTo<usize> {
    fn contains(&self, value: &usize) -> bool {
        Self::contains(self, value)
    }
}
impl RangeContains<usize> for RangeFull {
    fn contains(&self, _value: &usize) -> bool {
        true
    }
}

fn count_active_bins(bins: &[Complex<i16>]) -> usize {
    bins.iter().filter(|bin| !bin.is_zero()).count()
}

fn count_consecutive_active_bins(bins: &[Complex<i16>]) -> usize {
    let mut consecutive_active = 0;
    let mut consecutive_active_this_sequence = 0;
    let mut prev_active = false;

    for bin in bins.iter() {
        if !bin.is_zero() {
            if prev_active {
                // Second or subsequent consecutive active bin
                consecutive_active_this_sequence += 1;
            } else {
                // First active bin
                consecutive_active_this_sequence = 1;
            }
            consecutive_active = cmp::max(consecutive_active, consecutive_active_this_sequence);
        }
        prev_active = !bin.is_zero();
    }

    consecutive_active
}
