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
//! Collections of samples
//!

use std::fmt::{self, Write};
use std::iter::Enumerate;
use std::marker::PhantomData;
use std::ops::Range;
use std::slice::Iter;

use num_complex::Complex32;
use num_traits::Zero;

use super::bins::BinRange;

/// Marker for FFT-native bin ordering
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Fft {}

/// Marker for logical bin ordering (lowest to highest frequency)
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Logical {}

/// Ordering marker
pub trait Ordering {
    /// The other ordering
    type Other: Ordering;
}

impl Ordering for Fft {
    type Other = Logical;
}

impl Ordering for Logical {
    type Other = Fft;
}

/// A tag that can be assigned to a window
#[derive(Debug, Copy, Clone, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Tag(u32);

impl Tag {
    /// Returns the tag that logically follows this tag
    pub fn next(&self) -> Tag {
        Tag(self.0.wrapping_add(1))
    }
}

impl fmt::Display for Tag {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Forward to the integer implementation
        fmt::Display::fmt(&self.0, f)
    }
}

/// A window of frequency-domain data
#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Window<Ord = Fft> {
    /// The timestamp of this window, in 1024-sample half-windows
    time: u64,
    /// FFT bins (in frequency domain)
    ///
    /// Each value is a complex amplitude (in frequency domain)
    bins: Vec<Complex32>,
    /// An optional window tag
    tag: Option<Tag>,
    /// Order phantom
    _order_phantom: PhantomData<Ord>,
}

/// A window of time-domain samples
#[derive(Debug, Clone)]
pub struct TimeWindow {
    /// The timestamp of this window, in 1024-sample half-windows
    time: u64,
    /// Time-domain samples in this window
    samples: Vec<Complex32>,
    /// An optional window tag
    tag: Option<Tag>,
}

impl Window<Fft> {
    /// Creates a window with the specified time and size, and all bins initialized to zero
    pub fn new(time: u64, size: usize) -> Self {
        Window {
            time,
            bins: vec![Complex32::zero(); size],
            tag: None,
            _order_phantom: PhantomData,
        }
    }
}

impl Window<Logical> {
    /// Creates a window with the specified time and size, and all bins initialized to zero
    pub fn new_logical(time: u64, size: usize) -> Self {
        Window {
            time,
            bins: vec![Complex32::zero(); size],
            tag: None,
            _order_phantom: PhantomData,
        }
    }

    /// Returns true if this window contains any nonzero value within the provided range of bins
    pub fn overlaps_range(&self, range: &BinRange) -> bool {
        self.bins.iter().enumerate().any(|(i, value)| {
            if i < usize::from(u16::MAX) {
                range.contains(i as u16) && *value != Complex32::zero()
            } else {
                false
            }
        })
    }
}

impl<Ord> Window<Ord>
where
    Ord: Ordering,
{
    /// Creates a window with the provided time, size and bins
    ///
    /// # Panics
    ///
    /// This function panics if size is greater than
    pub fn with_bins<I>(time: u64, size: usize, bins: I) -> Self
    where
        I: IntoIterator<Item = Complex32>,
    {
        let mut collected_bins: Vec<Complex32> = bins.into_iter().collect();
        collected_bins.resize(size, Complex32::zero());
        Window {
            time,
            bins: collected_bins,
            tag: None,
            _order_phantom: Default::default(),
        }
    }

    /// Returns the timestamp of this window
    pub fn time(&self) -> u64 {
        self.time
    }
    /// Sets the timestamp of this window
    pub fn set_time(&mut self, time: u64) {
        self.time = time;
    }

    /// Returns a reference to the bins
    pub fn bins(&self) -> &[Complex32] {
        &self.bins
    }

    /// Returns a mutable reference to the bins
    pub fn bins_mut(&mut self) -> &mut [Complex32] {
        &mut self.bins
    }

    /// Sets the amplitude in a bin
    ///
    /// Panics if bin >= the size of this window
    pub fn set_amplitude(&mut self, bin: u16, amplitude: Complex32) {
        let len = self.bins.len();
        match self.bins.get_mut(bin as usize) {
            Some(entry) => *entry = amplitude,
            None => panic!("Bin index {} out of bounds for window size {}", bin, len),
        }
    }

    /// Sets the tag of this window
    pub fn set_tag(&mut self, tag: Tag) {
        self.tag = Some(tag);
    }

    /// Returns a displayable object that visualizes the bins in this window
    #[allow(dead_code)]
    pub fn visualize(&self) -> Visualize<'_, Ord> {
        Visualize(self)
    }

    /// Returns a displayable object that shows the non-empty bins in this window
    #[allow(dead_code)]
    pub fn show_non_empty(&self) -> ShowNonEmpty<'_, Ord> {
        ShowNonEmpty(self)
    }

    /// Returns a displayable object that shows the numbers in this window
    #[allow(dead_code)]
    pub fn show_numbers(&self) -> ShowNumbers<'_, Ord> {
        ShowNumbers(self)
    }

    /// Returns an iterator over active (nonzero) ranges of bins in this window
    #[allow(dead_code)]
    pub fn active_ranges(&self) -> ActiveRanges<'_> {
        ActiveRanges::new(self)
    }

    /// Truncates the bins in this window to len elements
    pub fn truncate_bins(&mut self, len: usize) {
        self.bins.truncate(len);
    }

    /// Converts this frequency-domain window into a time-domain window with the same timestamp
    /// and data. This function reuses the complex values that have already been allocated.
    ///
    /// This function does not modify the bin data. The bin data in this window become the time-
    /// domain samples in the returned time window.
    pub fn into_time_domain(self) -> TimeWindow {
        TimeWindow {
            time: self.time,
            samples: self.bins,
            tag: self.tag,
        }
    }

    /// Converts this frequency-domain into the other bin ordering
    ///
    /// This function does not modify the bin data.
    pub fn into_other_ordering(self) -> Window<Ord::Other> {
        Window {
            time: self.time,
            bins: self.bins,
            tag: self.tag,
            _order_phantom: PhantomData,
        }
    }
}

/// An iterator over ranges of bins with non-zero amplitude values in a window
pub struct ActiveRanges<'w> {
    /// An enumerated iterator over the bins in the window
    bins: Enumerate<Iter<'w, Complex32>>,
    /// The number of bins in the window
    length: usize,
}

impl<'w> ActiveRanges<'w> {
    fn new<Ordering>(window: &'w Window<Ordering>) -> Self {
        ActiveRanges {
            bins: window.bins.iter().enumerate(),
            length: window.bins.len(),
        }
    }
}

impl<'w> Iterator for ActiveRanges<'w> {
    type Item = Range<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut start = None;
        for (i, bin) in self.bins.by_ref() {
            if let Some(start) = start {
                // Look for the end of a nonzero area
                if bin.is_zero() {
                    let end = i;
                    return Some(start..end);
                }
            } else if !bin.is_zero() {
                // This is the start of a nonzero area
                start = Some(i);
            }
        }
        if let Some(start) = start {
            // Some non-zero range seen, ending at the end of the bins
            Some(start..self.length)
        } else {
            // End of bins, no non-zero bin seen, nothing else returned
            None
        }
    }
}

/// Wraps a Window and implements Display to visualize the non-empty bins in a window
///
/// The Display output looks like
/// [-----|||---||----]
/// where each - represents an empty bin and each | represents a bin with data
///
pub struct Visualize<'w, Ord>(&'w Window<Ord>);

impl<'w, Ord> fmt::Display for Visualize<'w, Ord>
where
    Ord: Ordering,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        for bin in self.0.bins() {
            if *bin == Complex32::zero() {
                f.write_char('-')?;
            } else {
                f.write_char('|')?;
            }
        }
        f.write_char(']')?;
        Ok(())
    }
}

/// Wraps a Window and implements Display to show the indices and values of non-empty bins in a
/// window
pub struct ShowNonEmpty<'w, Ord>(&'w Window<Ord>);

impl<'w, Ord> fmt::Display for ShowNonEmpty<'w, Ord>
where
    Ord: Ordering,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('{')?;
        for (i, bin) in self.0.bins().iter().enumerate() {
            if *bin != Complex32::zero() {
                write!(f, "{} => {}, ", i, bin)?;
            }
        }
        f.write_char('}')?;
        Ok(())
    }
}

/// Wraps a Window and implements Display to show the amplitude values in a window
pub struct ShowNumbers<'w, Ord>(&'w Window<Ord>);

impl<'w, Ord> fmt::Display for ShowNumbers<'w, Ord>
where
    Ord: Ordering,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('[')?;
        for bin in self.0.bins() {
            write!(f, "{} ", *bin)?;
        }
        f.write_char(']')?;
        Ok(())
    }
}

impl TimeWindow {
    /// Creates a TimeWindow with the provided time and samples
    pub fn new(time: u64, samples: Vec<Complex32>) -> Self {
        TimeWindow {
            time,
            samples,
            tag: None,
        }
    }

    /// Returns the timestamp of this window
    pub fn time(&self) -> u64 {
        self.time
    }

    /// Returns the number of samples in this window
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns true if this window does not contain any samples
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Returns a reference to the samples in this window
    pub fn samples(&self) -> &[Complex32] {
        &self.samples
    }

    /// Returns a reference to the samples in the first half of this window
    pub fn first_half(&self) -> &[Complex32] {
        &self.samples[..self.samples.len() / 2]
    }

    /// Returns a mutable reference to the samples in this window
    pub fn samples_mut(&mut self) -> &mut [Complex32] {
        &mut self.samples
    }

    /// Returns a mutable reference to the samples in the second half of this window
    pub fn second_half_mut(&mut self) -> &mut [Complex32] {
        let len = self.len();
        &mut self.samples[len / 2..]
    }
    /// Returns mutable references to the first and second halves of the samples in this window
    pub fn halves_mut(&mut self) -> (&mut [Complex32], &mut [Complex32]) {
        let len = self.len();
        self.samples.split_at_mut(len / 2)
    }
    /// Truncates the samples in this window to len samples
    fn truncate_samples(&mut self, len: usize) {
        self.samples.truncate(len);
    }

    /// Converts this window into a window containing the second half of samples in self
    pub fn into_second_half(mut self) -> Self {
        {
            let (first_half, second_half) = self.halves_mut();
            first_half.copy_from_slice(second_half);
        }
        let len = self.len();
        self.truncate_samples(len / 2);
        self
    }

    /// Returns this window's tag, if any
    pub fn tag(&self) -> Option<&Tag> {
        self.tag.as_ref()
    }

    /// Consumes this window and returns its samples
    pub fn into_samples(self) -> Vec<Complex32> {
        self.samples
    }
}

/// Converts a time-domain window into an iterator over its samples
impl IntoIterator for TimeWindow {
    type Item = Complex32;
    type IntoIter = std::vec::IntoIter<Complex32>;

    fn into_iter(self) -> Self::IntoIter {
        self.samples.into_iter()
    }
}

/// Success, timeout, or error
#[derive(Debug, Clone)]
pub enum Status<T> {
    /// Normal data
    Ok(T),
    /// Data have not been received in a while
    ///
    /// This indicates that later steps should flush any buffered data instead of waiting
    /// for more compressed samples
    Timeout,
    /// Every FFT and output stage gets this, which contains the timestamp of the first window
    /// of samples read from the file. The overlap step uses this to insert zeros before the
    /// first reconstructed time window.
    ///
    /// The timestamp is in half-windows (1024 samples at 100 MHz sample rate for the USRP N210)
    FirstWindowTime(u64),
}

/// A window, or the timestamp of the first window
#[derive(Debug)]
pub enum WindowOrTimestamp {
    /// A window
    Window(Window<Logical>),
    /// The timestamp of the first window
    FirstWindowTimestamp(u64),
}

#[cfg(test)]
mod test_active_ranges {
    use super::*;

    #[test]
    fn test_empty() {
        let window = Window::new(0, 0);
        let mut ranges = window.active_ranges();
        assert_eq!(None, ranges.next());
    }
    #[test]
    fn test_all_zero() {
        let size = 64;
        let window = Window::new(0, size);
        let mut ranges = window.active_ranges();
        assert_eq!(None, ranges.next());
    }
    #[test]
    fn test_one_beginning() {
        let size = 64;
        let mut window = Window::new(0, size);
        window.set_amplitude(0, Complex32::new(1.0, 1.0));
        let mut ranges = window.active_ranges();
        assert_eq!(Some(0..1), ranges.next());
        assert_eq!(None, ranges.next());
    }
    #[test]
    fn test_one_end() {
        let size = 64;
        let mut window = Window::new(0, size);
        window.set_amplitude(size as u16 - 1, Complex32::new(1.0, 1.0));
        let mut ranges = window.active_ranges();
        assert_eq!(Some(size - 1..size), ranges.next());
        assert_eq!(None, ranges.next());
    }
    #[test]
    fn test_one_middle() {
        let size = 64;
        let mut window = Window::new(0, size);
        window.set_amplitude(20, Complex32::new(1.0, 1.0));
        let mut ranges = window.active_ranges();
        assert_eq!(Some(20..21), ranges.next());
        assert_eq!(None, ranges.next());
    }
    #[test]
    fn test_two_beginning() {
        let size = 64;
        let mut window = Window::new(0, size);
        window.set_amplitude(0, Complex32::new(1.0, 1.0));
        window.set_amplitude(1, Complex32::new(1.0, 1.0));
        let mut ranges = window.active_ranges();
        assert_eq!(Some(0..2), ranges.next());
        assert_eq!(None, ranges.next());
    }
    #[test]
    fn test_one_beginning_one_end() {
        let size = 64;
        let mut window = Window::new(0, size);
        window.set_amplitude(0, Complex32::new(1.0, 1.0));
        window.set_amplitude(size as u16 - 1, Complex32::new(1.0, 1.0));
        let mut ranges = window.active_ranges();
        assert_eq!(Some(0..1), ranges.next());
        assert_eq!(Some(size - 1..size), ranges.next());
        assert_eq!(None, ranges.next());
    }
}
