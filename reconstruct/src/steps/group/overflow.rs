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

/// 21 bits for Pluto
const COUNTER_MAX: u64 = 0x1fffff;

/// Keeps track of a periodically overflowing 21-bit counter and expands its values into
/// 64-bit integers
pub struct Overflow {
    /// The current offset to add to each value
    offset: u64,
    /// The last input value, before expansion
    previous: u32,
}

impl Overflow {
    /// Creates an overflow calculator
    pub fn new() -> Self {
        Overflow {
            offset: 0,
            previous: 0,
        }
    }

    /// Expands a 20-bit counter value into a 64-bit counter value, handling overflows correctly
    pub fn expand(&mut self, value: u32) -> u64 {
        // Temporary workaround for the time value sometimes temporarily decreasing by 1
        if value == self.previous.wrapping_sub(1) {
            return self.expand(value.wrapping_add(1));
        }

        let expanded = if value >= self.previous {
            // No overflow
            // Add offset and wrap
            self.offset.wrapping_add(u64::from(value))
        } else {
            // For debugging only
            println!("Overflowed {} -> {}", self.previous, value);
            // Overflow (assume counter has only overflowed once)
            self.offset = self.offset.wrapping_add(COUNTER_MAX + 1);
            self.offset.wrapping_add(u64::from(value))
        };
        self.previous = value;
        expanded
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
        // Maximum value of 21-bit counter
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
}
