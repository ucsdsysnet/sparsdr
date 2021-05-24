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

//! Fast active bin matching

use std::fmt::{self, Write};
use std::iter::FromIterator;
use std::mem;
use std::ops::{BitAnd, Range};

/// The number of bits to store
const BITS: usize = 1024;
/// An unsigned integer type to use when storing bits
// On x86_64 and a Raspberry Pi 3, u64 is fastest, slightly faster than u128
type BitInt = u64;
/// Number of bits in a BitInt
const INT_BITS: usize = 8 * mem::size_of::<BitInt>();
/// Number of integers needed to store BITS bits
const INTS: usize = BITS / INT_BITS;

/// True/false flags for 2048 bins
///
/// This implementation is optimized for fast bitwise operations and overlap checks.
#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BinMask {
    /// A bit for each bin
    /// Within each integer, bins are ordered left to right.
    bits: [BitInt; INTS],
}

impl BinMask {
    /// Returns a BinMask with all bits cleared
    pub fn zero() -> Self {
        BinMask { bits: [0; INTS] }
    }

    /// Sets all bits in this mask to zero
    pub fn clear_all(&mut self) {
        for part in self.bits.iter_mut() {
            *part = 0;
        }
    }

    /// Sets the value of the bit at the provided index
    ///
    /// # Panics
    ///
    /// This function panics if index is too large.
    pub fn set(&mut self, index: usize, bit: bool) {
        if index < BITS {
            let part = &mut self.bits[index / INT_BITS];
            let bit_mask: BitInt = (1 as BitInt) << (index % INT_BITS);
            if bit {
                *part |= bit_mask;
            } else {
                *part &= !bit_mask;
            }
        } else {
            panic!(
                "Bit index {} out of range for BinMask of size {}",
                index, BITS
            );
        }
    }

    /// Sets bits within a range to 1
    ///
    /// # Panics
    ///
    /// This function panics if any part of range is outside the valid range of bits. If this
    /// happens, no bits are modified.
    pub fn set_range(&mut self, range: Range<usize>) {
        assert!(range.start <= BITS, "Range start too large");
        assert!(range.end <= BITS, "Range end too large");
        for i in range {
            self.set(i, true);
        }
    }

    /// Returns the value of the bit at the provided index
    ///
    /// # Panics
    ///
    /// This function panics if index is too large.
    pub fn get(&self, index: usize) -> bool {
        self.get_checked(index).expect("Bit index out of range")
    }

    /// Returns the value of the bit at the provided index, or None if the index is out of range
    pub fn get_checked(&self, index: usize) -> Option<bool> {
        if index < BITS {
            let part = self.bits[index / INT_BITS];
            let bit = (part >> (index % INT_BITS)) & 1;
            Some(bit == 1)
        } else {
            None
        }
    }

    /// Returns an iterator over bits in this mask, starting at bit zero
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            bin_mask: self,
            next: 0,
        }
    }

    /// Returns true if all bits in this mask are zero
    pub fn is_zero(&self) -> bool {
        self.bits.iter().all(|part| *part == 0)
    }

    /// Computes the bitwise and of self and another, and stores the result in result
    pub fn and(&self, other: &BinMask, result: &mut BinMask) {
        for (result_part, (a, b)) in result
            .bits
            .iter_mut()
            .zip(self.bits.iter().zip(other.bits.iter()))
        {
            *result_part = *a & *b;
        }
    }

    /// Returns the number of bits in this mask that are set to one
    pub fn count_ones(&self) -> u32 {
        self.bits.iter().map(|part| part.count_ones()).sum()
    }

    /// Returns true if this bin mask and another bin mask have any overlap (any indexes where both
    /// have a bit set to 1)
    pub fn overlaps(&self, other: &BinMask) -> bool {
        self.bits
            .iter()
            .zip(other.bits.iter())
            .any(|(a, b)| *a & *b != 0)
    }

    /// Clears all bits in self that are set to 1 in mask
    ///
    /// This is equivalent to self = self & !mask.
    pub fn clear_bits(&mut self, mask: &BinMask) {
        for (self_part, mask_part) in self.bits.iter_mut().zip(mask.bits.iter()) {
            *self_part &= !*mask_part;
        }
    }

    /// Applies an FFT shift to the bits in this mask.
    ///
    /// The first half is moved to the second half, and the second half is moved to the first half.
    pub fn shift(&mut self) {
        let half_length = self.bits.len() / 2;
        self.bits.rotate_left(half_length);
    }

    /// Returns the number of bits in this mask
    pub fn len(&self) -> usize {
        BITS
    }
}

impl<'l, 'r> BitAnd<&'r BinMask> for &'l BinMask {
    type Output = BinMask;

    fn bitand(self, rhs: &'r BinMask) -> Self::Output {
        let mut result = BinMask::zero();
        self.and(rhs, &mut result);
        result
    }
}

impl FromIterator<bool> for BinMask {
    /// Creates a mask by collecting bits from an iterator
    ///
    /// Any excess items at the end will be ignored.
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = bool>,
    {
        let mut mask = BinMask::zero();

        for (i, bit) in iter.into_iter().take(BITS).enumerate() {
            mask.set(i, bit);
        }

        mask
    }
}

/// An iterator over the bits in a mask
pub struct Iter<'a> {
    bin_mask: &'a BinMask,
    next: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.bin_mask.get_checked(self.next);
        self.next += 1;
        value
    }
}

impl fmt::Debug for BinMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BinMask(")?;
        for bit in self.iter() {
            if bit {
                f.write_char('1')?;
            } else {
                f.write_char('0')?;
            }
        }
        write!(f, ")")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_overlap() {
        assert!(!bits(&[false]).overlaps(&bits(&[false])));
        assert!(!bits(&[false, true]).overlaps(&bits(&[true, false])));
        assert!(bits(&[true, true]).overlaps(&bits(&[true, false])));
        assert!(bits(&[true, true]).overlaps(&bits(&[false, true])));
    }

    #[test]
    fn test_shift() {
        let mut mask = BinMask::zero();
        mask.set(0, true);
        mask.set(5, true);
        mask.set(1022, true);

        mask.set(1025, true);
        mask.set(2047, true);

        let expected = {
            let mut expected = BinMask::zero();

            expected.set(1, true);
            expected.set(1023, true);

            expected.set(1024, true);
            expected.set(1029, true);
            expected.set(2046, true);

            expected
        };

        mask.shift();
        assert_eq!(mask, expected);
    }

    fn bits(bits: &[bool]) -> BinMask {
        bits.iter().cloned().collect()
    }
}
