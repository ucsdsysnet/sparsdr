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

//! The overlap step

use std::cmp;
use std::iter::Fuse;

use crate::cursor::{ReadCursorExt, WriteCursorExt};
use crate::window::{Status, TimeWindow};

/// Modifies the second half of the first window by adding to each element the corresponding
/// value from the first half of the second window
fn overlap_windows(first: &mut TimeWindow, second: &TimeWindow) {
    let first_second_half = first.second_half_mut();
    let second_first_half = second.first_half();
    // If the slice lengths are not the same, copy over a shorter range
    // Align to the end of first_second_half and the beginning of second_first_half
    let copy_length = cmp::min(first_second_half.len(), second_first_half.len());

    let first_exclude = first_second_half.len() - copy_length;
    let first_second_half = &mut first_second_half[first_exclude..];

    let second_first_half = &second_first_half[..copy_length];

    assert_eq!(first_second_half.len(), second_first_half.len());
    for (first, second) in first_second_half.iter_mut().zip(second_first_half.iter()) {
        *first += *second;
    }
}

/// Overlaps windows
///
/// This implementation does not include gaps between samples.
pub struct Overlap {
    /// Previous window, of which half has been written
    prev_window: Option<TimeWindow>,
}

impl Overlap {
    /// Creates an overlapper
    pub fn new() -> Self {
        Overlap { prev_window: None }
    }

    /// Overlaps windows
    ///
    /// Returns the number of windows consumed and produced
    pub fn run(
        &mut self,
        windows_in: &[TimeWindow],
        windows_out: &mut [TimeWindow],
    ) -> OverlapResult {
        let mut windows_in = windows_in.read_cursor();
        let mut windows_out = windows_out.write_cursor();

        while !windows_out.is_empty() {
            if windows_in.is_empty() {
                break;
            }
            for new_window in windows_in.by_ref() {
                if let Some(mut prev_window) = self.prev_window.take() {
                    assert!(
                        new_window.time() > prev_window.time(),
                        "New window is not after previous window"
                    );
                    let time_difference = new_window.time() - prev_window.time();
                    match time_difference {
                        1 => {
                            // Overlap
                            overlap_windows(&mut prev_window, &new_window);
                            self.prev_window = Some(new_window.clone());
                            // Send second half of previous window
                            windows_out.write(prev_window.into_second_half());
                            break;
                        }
                        _ => {
                            // Don't overlap, just send second half of previous window followed by
                            // first half of new window

                            // Copy second half of previous window into first half
                            {
                                let (prev_first, prev_second) = prev_window.halves_mut();
                                prev_first.copy_from_slice(prev_second);
                            }

                            // Copy first half of new window into second half of previous window
                            {
                                prev_window
                                    .second_half_mut()
                                    .copy_from_slice(new_window.first_half());
                            }

                            // Store new window
                            self.prev_window = Some(new_window.clone());

                            // Send previous window
                            windows_out.write(prev_window);
                            break;
                        }
                    }
                } else {
                    // This is the first window
                    // Send out the first half and store the rest
                    let first_half =
                        TimeWindow::new(new_window.time(), new_window.first_half().to_vec());
                    self.prev_window = Some(new_window.clone());
                    windows_out.write(first_half);
                    // Break out of windows_in.by_ref() and check if windows_out is empty
                    break;
                }
            }
        }

        OverlapResult {
            windows_consumed: windows_in.read_count(),
            windows_produced: windows_out.write_count(),
        }
    }

    /// Removes and returns the buffered samples, if any exist
    pub fn flush(&mut self) -> Option<TimeWindow> {
        let prev_window = self.prev_window.take()?;
        // The first half of this window has already been provided.
        Some(prev_window.into_second_half())
    }
}

/// Information about an overlap operation
pub struct OverlapResult {
    /// Number of input windows consumed
    pub windows_consumed: usize,
    /// Number of output windows produced
    pub windows_produced: usize,
}

/// An iterator adapter that overlaps windows
///
/// This implementation does not include gaps between samples.
pub struct OverlapIter<I> {
    /// Inner iterator
    inner: Fuse<I>,
    /// Previous window, of which half has been written
    prev_window: Option<TimeWindow>,
    /// Window size, samples
    window_size: usize,
}

impl<I> OverlapIter<I>
where
    I: Iterator<Item = Status<TimeWindow>>,
{
    /// Creates an overlap iterator for the provided window size
    pub fn new(inner: I, window_size: usize) -> Self {
        OverlapIter {
            inner: inner.fuse(),
            prev_window: None,
            window_size,
        }
    }

    fn handle_window(&mut self, new_window: TimeWindow) -> Option<TimeWindow> {
        if let Some(mut prev_window) = self.prev_window.take() {
            assert!(
                new_window.time() > prev_window.time(),
                "New window is not after previous window"
            );
            let time_difference = new_window.time() - prev_window.time();
            match time_difference {
                1 => {
                    // Overlap
                    overlap_windows(&mut prev_window, &new_window);
                    self.prev_window = Some(new_window);
                    // Send second half of previous window
                    Some(prev_window.into_second_half())
                }
                _ => {
                    // Don't overlap, just send second half of previous window followed by
                    // first half of new window

                    // Copy second half of previous window into first half
                    {
                        let (prev_first, prev_second) = prev_window.halves_mut();
                        prev_first.copy_from_slice(prev_second);
                    }

                    // Copy first half of new window into second half of previous window
                    {
                        prev_window
                            .second_half_mut()
                            .copy_from_slice(new_window.first_half());
                    }

                    // Store new window
                    self.prev_window = Some(new_window);

                    // Send previous window
                    Some(prev_window)
                }
            }
        } else {
            // Send out the first half of the new window and store the rest
            let first_half = TimeWindow::new(new_window.time(), new_window.first_half().to_vec());
            self.prev_window = Some(new_window);
            Some(first_half)
        }
    }

    fn handle_end(&mut self) -> Option<TimeWindow> {
        if let Some(prev) = self.prev_window.take() {
            // Need to send the second half of the previous window
            Some(prev.into_second_half())
        } else {
            // The end
            None
        }
    }
}

impl<I> Iterator for OverlapIter<I>
where
    I: Iterator<Item = Status<TimeWindow>>,
{
    type Item = TimeWindow;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(Status::Ok(new_window)) => {
                    assert_eq!(new_window.len(), self.window_size, "Incorrect window size");
                    return self.handle_window(new_window);
                }
                Some(Status::Timeout) => {
                    if let Some(prev) = self.prev_window.take() {
                        // Send the second half of the window
                        return Some(prev.into_second_half());
                    } else {
                        // Continue waiting for something to happen
                    }
                }
                None => return self.handle_end(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use num_complex::Complex32;
    use std::iter;

    #[test]
    fn test_one_window() {
        let samples = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0, 6.21),
            Complex32::new(-0.3, -9.2),
        ];
        let windows = iter::once(TimeWindow::new(0, samples.clone()));
        check_iter(4, windows.into_iter().map(Status::Ok), &samples);
    }

    #[test]
    fn test_non_iter_one_window() {
        let samples = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0, 6.21),
            Complex32::new(-0.3, -9.2),
        ];
        let windows = &[TimeWindow::new(0, samples.clone())];
        check_non_iter(windows, &samples);
    }

    #[test]
    fn test_two_windows_no_gap() {
        let samples1 = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0, 6.21),
            Complex32::new(-0.3, -9.2),
        ];

        let samples2 = vec![
            Complex32::new(5.0, 6.0),
            Complex32::new(3.2, 127.05),
            Complex32::new(6.0, 9.26),
            Complex32::new(-2.3, -16.2),
        ];
        // Middle two samples overlap
        let expected_samples = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0 + 5.0, 6.21 + 6.0),
            Complex32::new(-0.3 + 3.2, -9.2 + 127.05),
            Complex32::new(6.0, 9.26),
            Complex32::new(-2.3, -16.2),
        ];

        let windows = vec![TimeWindow::new(0, samples1), TimeWindow::new(1, samples2)];
        check_iter(4, windows.into_iter().map(Status::Ok), &expected_samples);
    }

    #[test]
    fn test_non_iter_two_windows_no_gap() {
        let samples1 = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0, 6.21),
            Complex32::new(-0.3, -9.2),
        ];

        let samples2 = vec![
            Complex32::new(5.0, 6.0),
            Complex32::new(3.2, 127.05),
            Complex32::new(6.0, 9.26),
            Complex32::new(-2.3, -16.2),
        ];
        // Middle two samples overlap
        let expected_samples = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0 + 5.0, 6.21 + 6.0),
            Complex32::new(-0.3 + 3.2, -9.2 + 127.05),
            Complex32::new(6.0, 9.26),
            Complex32::new(-2.3, -16.2),
        ];

        let windows = vec![TimeWindow::new(0, samples1), TimeWindow::new(1, samples2)];
        check_non_iter(&windows, &expected_samples);
    }

    #[test]
    fn test_two_windows_timeout() {
        let samples1 = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0, 6.21),
            Complex32::new(-0.3, -9.2),
        ];

        let samples2 = vec![
            Complex32::new(5.0, 6.0),
            Complex32::new(3.2, 127.05),
            Complex32::new(6.0, 9.26),
            Complex32::new(-2.3, -16.2),
        ];
        // No overlap because of timeout
        let expected_samples = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0, 6.21),
            Complex32::new(-0.3, -9.2),
            Complex32::new(5.0, 6.0),
            Complex32::new(3.2, 127.05),
            Complex32::new(6.0, 9.26),
            Complex32::new(-2.3, -16.2),
        ];

        let windows = vec![
            Status::Ok(TimeWindow::new(0, samples1)),
            Status::Timeout,
            Status::Ok(TimeWindow::new(1, samples2)),
        ];
        check_iter(4, windows, &expected_samples);
    }

    #[test]
    fn test_non_iter_two_windows_timeout() {
        let samples1 = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0, 6.21),
            Complex32::new(-0.3, -9.2),
        ];

        let samples2 = vec![
            Complex32::new(5.0, 6.0),
            Complex32::new(3.2, 127.05),
            Complex32::new(6.0, 9.26),
            Complex32::new(-2.3, -16.2),
        ];
        // No overlap because of timeout
        let expected_samples = vec![
            Complex32::new(1.0, 2.0),
            Complex32::new(0.2, 0.05),
            Complex32::new(127.0, 6.21),
            Complex32::new(-0.3, -9.2),
            Complex32::new(5.0, 6.0),
            Complex32::new(3.2, 127.05),
            Complex32::new(6.0, 9.26),
            Complex32::new(-2.3, -16.2),
        ];

        let mut overlap = Overlap::new();
        // 4 half-length windows expected
        let mut out_windows = vec![TimeWindow::new(0, vec![]); 4];
        let status = overlap.run(&[TimeWindow::new(0, samples1.to_vec())], &mut out_windows);
        assert_eq!(1, status.windows_consumed);
        assert_eq!(1, status.windows_produced);
        // Flush to reflect timeout
        out_windows[1] = overlap.flush().expect("flush() did not produce a window");
        let status2 = overlap.run(
            &[TimeWindow::new(1, samples2.to_vec())],
            &mut out_windows[2..],
        );
        assert_eq!(1, status2.windows_consumed);
        assert_eq!(1, status2.windows_produced);
        // Flush to end
        out_windows[3] = overlap.flush().expect("flush() did not produce a window");
        // Check samples
        let actual_samples = out_windows
            .iter()
            .flat_map(|window| window.samples().to_vec().into_iter())
            .collect::<Vec<Complex32>>();
        assert_eq!(actual_samples, expected_samples);
    }

    fn check_iter<I>(window_size: usize, windows: I, expected: &[Complex32])
    where
        I: IntoIterator<Item = Status<TimeWindow>>,
    {
        let overlap = OverlapIter::new(windows.into_iter(), window_size);
        let result = overlap.flatten().collect::<Vec<Complex32>>();
        assert_eq!(&*result, expected);
    }
    fn check_non_iter(windows: &[TimeWindow], expected: &[Complex32]) {
        let mut overlap = Overlap::new();
        // Maximum output length = input length
        let mut output = vec![TimeWindow::new(0, vec![]); windows.len()];
        let result = overlap.run(windows, &mut output);
        assert_eq!(result.windows_consumed, windows.len());
        // Flush
        output.truncate(result.windows_produced);
        output.extend(overlap.flush());
        let output_samples = output
            .into_iter()
            .flat_map(|window| window.samples().to_vec().into_iter())
            .collect::<Vec<Complex32>>();
        assert_eq!(&output_samples[..], expected);
    }
}
