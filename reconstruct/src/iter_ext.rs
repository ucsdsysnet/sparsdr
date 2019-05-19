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

//! Extension traits for the various iterator adapters

use std::io::Result;

use crate::bins::BinRange;
use crate::input::Sample;
use crate::steps::fft::Fft;
use crate::steps::filter_bins::FilterBinsIter;
use crate::steps::frequency_correct::FrequencyCorrectIter;
use crate::steps::group::Grouper;
use crate::steps::overlap::Overlap;
use crate::steps::phase_correct::PhaseCorrectIter;
use crate::steps::shift::{ShiftIter, ShiftWindowResultIter};

use super::window::{Logical, Status, TimeWindow, Window};

/// Iterator extension for signal processing
pub trait IterExt {
    /// Groups samples with the same time field into windows
    fn group(self, fft_size: usize) -> Grouper<Self>
    where
        Self: Iterator<Item = Result<Sample>> + Sized,
    {
        Grouper::new(self, fft_size)
    }

    /// Filters windows based on a bin range
    ///
    /// bin_range: The range of bins to keep
    ///
    /// fft_size: The size of the windows to produce
    fn filter_bins(self, bins: BinRange, fft_size: u16) -> FilterBinsIter<Self>
    where
        Self: Iterator<Item = Status<Window<Logical>>> + Sized,
    {
        FilterBinsIter::new(self, bins, fft_size)
    }
    /// Shifts the bins in windows between logical and FFT order
    fn shift<Ord>(self, fft_size: u16) -> ShiftIter<Self>
    where
        Self: Iterator<Item = Status<Window<Ord>>> + Sized,
    {
        ShiftIter::new(self, fft_size)
    }
    /// Shifts the bins in windows between logical and FFT order
    ///
    /// This version works on iterators with item type Result<Window>.
    fn shift_result<Ord>(self, fft_size: u16) -> ShiftWindowResultIter<Self>
    where
        Self: Iterator<Item = Result<Window<Ord>>> + Sized,
    {
        ShiftWindowResultIter::new(self, fft_size)
    }

    /// Applies a phase correction to windows
    fn phase_correct(self, fc_bins: f32) -> PhaseCorrectIter<Self>
    where
        Self: Iterator<Item = Status<Window>> + Sized,
    {
        PhaseCorrectIter::new(self, fc_bins)
    }

    /// Applies an inverse FFT to windows
    fn fft(self, fft_size: u16) -> Fft<Self>
    where
        Self: Iterator<Item = Status<Window>> + Sized,
    {
        Fft::new(self, usize::from(fft_size))
    }

    /// Overlaps windows with consecutive time values
    fn overlap(self, window_size: usize) -> Overlap<Self>
    where
        Self: Iterator<Item = Status<TimeWindow>> + Sized,
    {
        Overlap::new(self, window_size)
    }

    /// Applies frequency correction to time-domain samples
    fn frequency_correct(self, selected_center: f32, fft_size: u16) -> FrequencyCorrectIter<Self>
    where
        Self: Iterator<Item = TimeWindow> + Sized,
    {
        FrequencyCorrectIter::new(self, selected_center, fft_size)
    }
}

impl<I> IterExt for I where I: Iterator {}
