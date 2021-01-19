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

//! The shift step

use std::io::Result;

use crate::window::{Fft, Logical, Ordering, Status, Window};

/// Shifts data samples between FFT ordering and logical ordering
///
/// FFT ordering has the center frequency at bin 0, maximum frequency at bin size / 2 - 1,
/// minimum frequency at bin size / 2, and slightly less than center frequency at bin size - 1.
///
/// Logical ordering has the center frequency at bin size / 2 - 1, minimum frequency at bin 0,
/// and maximum frequency at bin size - 1.
///
/// This step adjusts the index fields of data samples to perform this conversion
/// in either direction.
///
pub struct Shift {
    /// FFT size
    size: u16,
}

impl Shift {
    /// Creates a shifter
    ///
    /// size: The size of the FFT to shift for
    ///
    /// This function will panic if size is zero or greater than one and odd.
    pub fn new(size: u16) -> Self {
        assert_ne!(size, 0, "size must not be zero");
        if size != 1 {
            assert_eq!(size % 2, 0, "size must be even if it is greater than one");
        }
        Shift { size }
    }

    /// Shifts all samples in a window from one ordering to the other
    pub fn shift_window<ORD>(&self, mut window: Window<ORD>) -> Window<ORD::Other>
    where
        ORD: Ordering,
    {
        debug_assert_eq!(
            window.bins().len(),
            usize::from(self.size),
            "Incorrect window size"
        );
        {
            // Swap first half and second half
            let half_size = usize::from(self.size / 2);
            let bins = window.bins_mut();
            bins.rotate_right(half_size);
        }
        // Do the same for the active bin bits
        window.active_bins_mut().shift();
        window.into_other_ordering()
    }

    /// Shifts multiple windows
    ///
    /// Returns the number of windows processed
    pub fn shift_windows<ORD>(
        &self,
        windows_in: &mut [Window<ORD>],
        windows_out: &mut [Window<ORD::Other>],
    ) -> usize
    where
        ORD: Ordering,
    {
        windows_in
            .iter_mut()
            .zip(windows_out.iter_mut())
            .map(|(window_in, window_out)| {
                debug_assert_eq!(
                    window_in.bins().len(),
                    usize::from(self.size),
                    "Incorrect window size"
                );
                // Apply the shift to window_in, then swap it with window_out to change the type
                let half_size = usize::from(self.size / 2);
                window_in.bins_mut().rotate_right(half_size);
                window_in.active_bins_mut().shift();
                window_in.swap_with_other_ordering(window_out);
            })
            .count()
    }
}

/// An iterator adapter that applies the shift to windows
pub struct ShiftIter<I> {
    /// Inner iterator
    inner: I,
    /// Shift logic
    shift: Shift,
}

impl<I> ShiftIter<I> {
    /// Creates a shift iterator for the provided FFT size
    pub fn new(inner: I, size: u16) -> Self {
        ShiftIter {
            inner,
            shift: Shift::new(size),
        }
    }
}

impl<I> Iterator for ShiftIter<I>
where
    I: Iterator<Item = Status<Window<Logical>>>,
{
    type Item = Status<Window<Fft>>;

    fn next(&mut self) -> Option<Self::Item> {
        let window: Window<Logical> = try_status!(self.inner.next());
        let window: Window<Fft> = self.shift.shift_window(window);
        Some(Status::Ok(window))
    }
}

/// An iterator adapter that applies the shift to windows
pub struct ShiftWindowResultIter<I> {
    /// Inner iterator
    inner: I,
    /// Shift logic
    shift: Shift,
}

impl<I> ShiftWindowResultIter<I> {
    /// Creates a window shift iterator for the provided FFT size
    pub fn new(inner: I, size: u16) -> Self {
        ShiftWindowResultIter {
            inner,
            shift: Shift::new(size),
        }
    }
}

impl<I, Ord> Iterator for ShiftWindowResultIter<I>
where
    I: Iterator<Item = Result<Window<Ord>>>,
    Ord: Ordering,
{
    type Item = Result<Window<Ord::Other>>;

    fn next(&mut self) -> Option<Self::Item> {
        let window = try_option_result!(self.inner.next());
        let window = self.shift.shift_window(window);
        Some(Ok(window))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::input::Sample;
    use num_complex::Complex32;
    use num_traits::Zero;

    #[test]
    fn test_size_1() {
        let input = vec![
            Sample {
                time: 0,
                index: 0,
                amplitude: Complex32::zero(),
            },
            Sample {
                time: 1,
                index: 0,
                amplitude: Complex32::zero(),
            },
        ];
        check_window(1, input.iter().cloned(), input.iter().cloned());
    }

    #[test]
    fn test_size_2() {
        let input = vec![
            Sample {
                time: 0,
                index: 0,
                amplitude: Complex32::zero(),
            },
            Sample {
                time: 0,
                index: 1,
                amplitude: Complex32::zero(),
            },
        ];
        let expected = vec![
            Sample {
                time: 0,
                index: 1,
                amplitude: Complex32::zero(),
            },
            Sample {
                time: 0,
                index: 0,
                amplitude: Complex32::zero(),
            },
        ];
        check_window(2, input.iter().cloned(), expected.iter().cloned());
    }
    #[test]
    fn test_size_4() {
        let input = vec![
            Sample {
                time: 0,
                index: 0,
                amplitude: Complex32::zero(),
            },
            Sample {
                time: 0,
                index: 1,
                amplitude: Complex32::zero(),
            },
            Sample {
                time: 0,
                index: 3,
                amplitude: Complex32::zero(),
            },
        ];
        let expected = vec![
            Sample {
                time: 0,
                index: 2,
                amplitude: Complex32::zero(),
            },
            Sample {
                time: 0,
                index: 3,
                amplitude: Complex32::zero(),
            },
            Sample {
                time: 0,
                index: 1,
                amplitude: Complex32::zero(),
            },
        ];
        check_window(4, input.iter().cloned(), expected.iter().cloned());
    }

    fn check_window<I1, I2>(size: u16, input: I1, expected: I2)
    where
        I1: IntoIterator<Item = Sample>,
        I2: IntoIterator<Item = Sample>,
    {
        let in_window: Window<Fft> = Window::with_samples(0, usize::from(size), input);
        let expected_window: Window<Logical> = Window::with_samples(0, usize::from(size), expected);

        let shift = Shift::new(size);
        let actual_window = shift.shift_window(in_window.clone());

        assert_eq!(expected_window, actual_window);

        // Reverse shift should restore
        let restored_window = shift.shift_window(actual_window);
        assert_eq!(in_window, restored_window);
    }
}
