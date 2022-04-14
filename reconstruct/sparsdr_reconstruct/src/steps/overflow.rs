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
//! Handles overflow of a time counter
//!

use iter::PushIterator;
use std::convert::TryInto;
use std::io::Result;
use std::ops::ControlFlow;

use crate::window::Window;

/// Keeps track of a periodically overflowing 20-bit counter and expands its values into
/// 64-bit integers
pub struct Overflow {
    /// The current offset to add to each value
    offset: u64,
    /// The last input value, before expansion
    previous: u32,
    /// The maximum value that the counter can hold
    max: u64,
}

impl Overflow {
    /// Creates an overflow calculator
    pub fn new(counter_bits: u32) -> Self {
        Overflow {
            offset: 0,
            previous: 0,
            max: (1 << counter_bits) - 1,
        }
    }

    /// Expands a 20-bit counter value into a 64-bit counter value, handling overflows correctly
    pub fn expand(&mut self, value: u32) -> u64 {
        if value < self.previous {
            // Overflow (assume counter has only overflowed once)
            self.offset = self.offset.wrapping_add(self.max + 1);
        }
        let expanded = self.offset.wrapping_add(u64::from(value));

        self.previous = value;
        expanded
    }
}

/// An iterator that applies overflow correction to windows
pub struct OverflowResultIter<I> {
    inner: I,
    overflow: Overflow,
}

impl<I> OverflowResultIter<I> {
    /// Creates a window overflow corrector
    pub fn new(inner: I, timestamp_bits: u32) -> Self {
        OverflowResultIter {
            inner,
            overflow: Overflow::new(timestamp_bits),
        }
    }
}

impl<I> Iterator for OverflowResultIter<I>
where
    I: Iterator<Item = Result<Window>>,
{
    type Item = Result<Window>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut window = try_option_result!(self.inner.next());

        window.set_time(
            self.overflow
                .expand(window.time().try_into().expect("Window time too large")),
        );

        Some(Ok(window))
    }
}

/// A push iterator that applies overflow correction to windows
pub struct OverflowPushIter<I> {
    inner: I,
    overflow: Overflow,
}

impl<I> PushIterator<Window> for OverflowPushIter<I>
where
    I: PushIterator<Window>,
{
    type Error = I::Error;

    fn push(&mut self, mut window: Window) -> ControlFlow<Self::Error> {
        let expanded_time = self
            .overflow
            .expand(window.time().try_into().expect("Window time too large"));
        window.set_time(expanded_time);
        self.inner.push(window)
    }

    fn flush(&mut self) -> std::result::Result<(), Self::Error> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    /// 20 bits
    const COUNTER_MAX: u64 = 0xfffff;

    #[test]
    fn test_no_overflow() {
        let mut overflow = Overflow::new(20);
        assert_eq!(0, overflow.expand(0));
        assert_eq!(0, overflow.expand(0));
        assert_eq!(1, overflow.expand(1));
        assert_eq!(2, overflow.expand(2));
        // Maximum value of 20-bit counter
        assert_eq!(COUNTER_MAX, overflow.expand(COUNTER_MAX as u32));
    }

    #[test]
    fn test_overflow() {
        let mut overflow = Overflow::new(20);
        assert_eq!(0, overflow.expand(0));
        assert_eq!(COUNTER_MAX, overflow.expand(COUNTER_MAX as u32));
        // Overflow by one
        assert_eq!(COUNTER_MAX + 1, overflow.expand(0));
        assert_eq!(COUNTER_MAX + 2, overflow.expand(1));
    }
}
