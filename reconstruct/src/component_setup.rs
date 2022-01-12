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

//! Determines a good setup for the stages of decompression

use std::collections::BTreeMap;
use std::io::Result;

use crossbeam::channel;

use crate::band_decompress::BandSetup;
use crate::bins::BinRange;
use crate::channel_ext::{LoggingReceiver, LoggingSender};
use crate::stages::fft_and_output::{FftAndOutputSetup, OutputSetup};
use crate::stages::input::{InputSetup, ToFft};
use crate::steps::overlap::OverlapMode;
use crate::window::{Window, WindowOrTimestamp};

/// Setups for the input stage, and the combined FFT and output stages
pub struct StagesCombined<'w, I> {
    /// Input stage setup
    pub input: InputSetup<I>,
    /// FFT and output stages setup
    pub fft_and_output: Vec<FftAndOutputSetup<'w>>,
}

/// Calculates and returns setups for the input, FFT and output stages to decompress the provided
/// bands from the provided source of samples
///
/// samples: an iterator that yields samples to be decompressed
///
/// bands: an iterator that yields the bands to be decompressed
///
/// channel_capacity: the capacity of the channels connecting the input stage to each output stage
///
pub fn set_up_stages_combined<'w, I, B>(
    windows: I,
    bands: B,
    compression_fft_size: usize,
    timestamp_bits: u32,
    channel_capacity: usize,
    overlap_mode: OverlapMode,
) -> StagesCombined<'w, I::IntoIter>
where
    I: IntoIterator<Item = Result<Window>>,
    B: IntoIterator<Item = BandSetup<'w>>,
{
    let bands = bands.into_iter();
    // Each (bin range, fc_bins) gets one FFT and output stage
    let mut ffts: BTreeMap<FftKey, FftAndOutputSetup<'w>> = BTreeMap::new();

    let mut input = InputSetup {
        samples: windows.into_iter(),
        compression_fft_size,
        timestamp_bits,
        destinations: Vec::new(),
    };

    for band_setup in bands {
        // Create an FFT setup if none exists
        let fft_setup = ffts.entry(key(&band_setup)).or_insert_with(|| {
            // Create a new channel to this FFT stage
            let (tx, rx) = channel::bounded(channel_capacity);
            let tx: LoggingSender<WindowOrTimestamp> = LoggingSender::new(tx);
            let rx: LoggingReceiver<WindowOrTimestamp> = LoggingReceiver::new(rx);

            log::debug!("Band bin range {}", band_setup.bins);
            input.destinations.push(ToFft {
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
            time_log: band_setup.time_log,
        };
        fft_setup.outputs.push(output_setup);
    }
    let fft_and_output = ffts.into_iter().map(|(_, value)| value).collect::<Vec<_>>();

    StagesCombined {
        input,
        fft_and_output,
    }
}

/// A key that uniquely identifies an FFT stage
///
/// This contains the bin range and fc_bins (normally a whole-number f32, here as an integer)
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct FftKey(BinRange, i64);
/// Creates a key from a band setup
fn key(band_setup: &BandSetup<'_>) -> FftKey {
    FftKey(band_setup.bins.clone(), band_setup.fc_bins as i64)
}
