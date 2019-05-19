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
//! Handles overflow of the 20-bit time counter
//!

/// 20 bits
const COUNTER_MAX: u64 = 0xfffff;

/// Keeps track of a periodically overflowing 20-bit counter and expands its values into
/// 64-bit integers, also applying an offset so that the first expanded value is always zero
#[derive(Debug, Default)]
pub struct Overflow {
    /// The current offset to add to each value
    ///
    /// This is None if expand() has not been called.
    offset: Option<i64>,
    /// The last input value, before expansion
    previous: u32,
}

impl Overflow {
    /// Creates an overflow calculator
    pub fn new() -> Self {
        Self::default()
    }

    /// Expands a 20-bit counter value into a 64-bit counter value, handling overflows correctly
    pub fn expand(&mut self, value: u32) -> u64 {
        if self.offset.is_none() {
            self.offset = Some(-i64::from(value));
        }
        let offset = self.offset.unwrap();

        let expanded = if value >= self.previous {
            // No overflow
            // Add offset and wrap
            offset.wrapping_add(i64::from(value))
        } else {
            // Overflow (assume counter has only overflowed once)
            let new_offset = offset.wrapping_add(COUNTER_MAX as i64 + 1);
            self.offset = Some(new_offset);
            new_offset.wrapping_add(i64::from(value))
        };
        self.previous = value;
        expanded as u64
    }

    /// Expands a 20-bit counter value into a 64-bit counter value, adding the offset but not
    /// checking for overflow or updating the previous value
    pub fn expand_ignore_overflow(&mut self, value: u32) -> u64 {
        self.offset.unwrap_or(0).wrapping_add(i64::from(value)) as u64
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_overflow() {
        let mut overflow = Overflow::new();
        assert_eq!(0, overflow.expand(0));
        assert_eq!(0, overflow.expand(0));
        assert_eq!(1, overflow.expand(1));
        assert_eq!(2, overflow.expand(2));
        // Maximum value of 20-bit counter
        assert_eq!(COUNTER_MAX, overflow.expand(COUNTER_MAX as u32));
    }

    #[test]
    fn test_overflow() {
        let mut overflow = Overflow::new();
        assert_eq!(0, overflow.expand(0));
        assert_eq!(COUNTER_MAX, overflow.expand(COUNTER_MAX as u32));
        // Overflow by one
        assert_eq!(COUNTER_MAX + 1, overflow.expand(0));
        assert_eq!(COUNTER_MAX + 2, overflow.expand(1));
    }

    #[test]
    fn test_first_nonzero_overflow() {
        let mut overflow = Overflow::new();
        assert_eq!(0, overflow.expand(10));
        assert_eq!(1, overflow.expand(11));
        assert_eq!(COUNTER_MAX - 11, overflow.expand(COUNTER_MAX as u32 - 1));
        assert_eq!(COUNTER_MAX - 10, overflow.expand(COUNTER_MAX as u32));
        assert_eq!(COUNTER_MAX - 9, overflow.expand(0));
        assert_eq!(COUNTER_MAX - 8, overflow.expand(1));
        // ...
        assert_eq!(COUNTER_MAX - 1, overflow.expand(8));
        assert_eq!(COUNTER_MAX, overflow.expand(9));
        assert_eq!(COUNTER_MAX + 1, overflow.expand(10));

    }
}
