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

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

use crossbeam::{channel, Receiver};
use num_complex::Complex32;

use sparsdr_sample_parser::{Parser, WindowKind};
use window::Window;

use crate::bins::BinRange;
use crate::iter::PushIterator;
use crate::stages::fft_and_output::run_fft_and_output_stage;
use crate::stages::input::{InputSetup, ToFft, ToFfts};
use crate::steps::overflow::OverflowPushIter;
use crate::steps::overlap::OverlapMode;
use crate::steps::shift::ShiftPushIter;
use crate::window::WindowOrTimestamp;
use crate::{BandSetup, DecompressSetup};

pub struct Reconstruct {
    parser: Box<dyn Parser>,
    chain: OverflowPushIter<ShiftPushIter<ToFfts>>,
    /// Join handles for all the FFT threads
    threads: Vec<JoinHandle<()>>,
    // TODO
}

impl Reconstruct {
    pub fn start(setup: DecompressSetup) -> io::Result<Self> {
        let combined_setup = configure_ffts(
            setup.bands,
            setup.channel_capacity,
            setup.compression_fft_size,
            setup.overlap_mode,
        );

        let input_setup = InputSetup {
            compression_fft_size: setup.compression_fft_size,
            timestamp_bits: setup.timestamp_bits,
            destinations: combined_setup.inputs,
        };

        let mut join_handles = Vec::new();

        // Create a stop flag that's always false if the caller did not provide one
        let stop = setup
            .stop
            .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));

        // Start threads
        for (_key, setup) in combined_setup.ffts {
            let stop = Arc::clone(&stop);
            let join_handle = thread::Builder::new()
                .name(format!("Bins {}", setup.bins))
                .spawn(move || run_fft_and_output_stage(setup, stop))?;
            join_handles.push(join_handle);
        }

        // Create input chain
        let chain = OverflowPushIter::new(
            ShiftPushIter::new(ToFfts::new(input_setup), setup.compression_fft_size),
            setup.timestamp_bits,
        );

        Ok(Reconstruct {
            parser: setup.parser,
            chain,
            threads: join_handles,
        })
    }

    pub fn process_samples(&mut self, sample_bytes: &[u8]) {
        let sample_size_bytes = self.parser.sample_bytes();
        assert_eq!(
            sample_bytes.len() % sample_size_bytes,
            0,
            "Number of sample bytes is not a multiple of sample size"
        );
        for one_sample_bytes in sample_bytes.chunks_exact(sample_size_bytes) {
            match self.parser.parse(one_sample_bytes) {
                Ok(Some(window)) => {
                    if let Some(converted_window) = convert_window(window) {
                        let todo = self.chain.push(converted_window);
                    }
                }
                Ok(_) => {}
                Err(e) => todo!(),
            }
        }
    }
}

/// If the provided window is a data window, this function returns its converted form.
/// Otherwise, this function returns None.
fn convert_window(window: sparsdr_sample_parser::Window) -> Option<Window> {
    match window.kind {
        WindowKind::Data(bins) => {
            // Convert integer to float and scale to [-1, 1]
            let scaled_bins = bins
                .into_iter()
                .map(|int_complex| {
                    Complex32::new(
                        (int_complex.re as f32) / 32767.0,
                        (int_complex.im as f32) / 32767.0,
                    )
                })
                .collect();
            Some(Window::with_bins(window.timestamp.into(), scaled_bins))
        }
        WindowKind::Average(_) => None,
    }
}

struct CombinedSetup {
    inputs: Vec<ToFft>,
    ffts: BTreeMap<FftKey, FftAndOutputSetup>,
}

fn configure_ffts(
    bands: Vec<BandSetup>,
    channel_capacity: usize,
    compression_fft_size: usize,
    overlap_mode: OverlapMode,
) -> CombinedSetup {
    // Each (bin range, fc_bins) gets one FFT and output stage
    let mut ffts: BTreeMap<FftKey, FftAndOutputSetup> = BTreeMap::new();
    let mut inputs = Vec::new();

    for band_setup in bands {
        // Create an FFT setup if none exists
        let fft_setup = ffts.entry(key(&band_setup)).or_insert_with(|| {
            // Create a new channel to this FFT stage
            let (tx, rx) = channel::bounded(channel_capacity);

            log::debug!("Band bin range {}", band_setup.bins);
            inputs.push(ToFft {
                bins: band_setup.bins.clone(),
                tx,
            });
            FftAndOutputSetup {
                source: rx,
                bins: band_setup.bins.clone(),
                compression_fft_size,
                fft_size: band_setup.fft_size,
                fc_bins: band_setup.fc_bins,
                timeout: band_setup.timeout,
                overlap: overlap_mode.clone(),
                outputs: vec![],
            }
        });

        let output_setup = OutputSetup {
            bin_offset: band_setup.bin_offset,
            destination: band_setup.destination,
        };
        fft_setup.outputs.push(output_setup);
    }

    CombinedSetup { inputs, ffts }
}

pub struct FftAndOutputSetup {
    /// Source of windows
    pub source: Receiver<WindowOrTimestamp>,
    /// The bins to decompress
    pub bins: BinRange,
    /// The FFT size used for compression
    pub compression_fft_size: usize,
    /// The actual FFT size to use
    pub fft_size: u16,
    /// Floor of the center frequency offset, in bins
    pub fc_bins: f32,
    /// Time to wait for a compressed sample before flushing output
    pub timeout: Duration,
    /// Overlap mode (gaps or flush samples)
    pub overlap: OverlapMode,
    /// The output setups
    pub outputs: Vec<OutputSetup>,
}

pub struct OutputSetup {
    /// Fractional part of center frequency offset, in bins
    pub bin_offset: f32,
    /// The destination to write decompressed samples to
    pub destination: Box<dyn WriteSamples + Send + 'static>,
}

/// Something that can handle reconstructed samples
pub trait WriteSamples {
    /// Handles zero or more reconstructed samples
    fn write_samples(&mut self, samples: &[Complex32]);
}

/// A key that uniquely identifies an FFT stage
///
/// This contains the bin range and fc_bins (normally a whole-number f32, here as an integer)
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FftKey(BinRange, i64);
/// Creates a key from a band setup
fn key(band_setup: &BandSetup) -> FftKey {
    FftKey(band_setup.bins.clone(), band_setup.fc_bins as i64)
}
