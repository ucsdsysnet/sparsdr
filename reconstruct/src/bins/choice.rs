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

//! Automatic bin selection

use super::super::NATIVE_FFT_SIZE;
use super::BinRange;

/// Chooses a range of bins
///
/// count: The number of bins in the range
///
/// offset: The offset from the center of the original 0..2048 range to the center of the range
///
pub fn choose_bins(count: u16, offset: i16) -> BinRange {
    // TODO clean up types and add error handling
    let low_bin = NATIVE_FFT_SIZE as i16 / 2 + offset - count as i16 / 2;
    let mut high_bin = NATIVE_FFT_SIZE as i16 / 2 + offset + count as i16 / 2;
    // Add 1 for odd values
    if count % 2 == 1 {
        high_bin += 1;
    }
    assert_eq!(
        high_bin - low_bin,
        count as i16,
        "Bin range calculation incorrect"
    );
    BinRange::from(saturating_i16_to_u16(low_bin)..saturating_i16_to_u16(high_bin))
}

/// Converts an i16 to a u16, with negative values becoming zero
fn saturating_i16_to_u16(x: i16) -> u16 {
    if x < 0 {
        0
    } else {
        x as u16
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_all_2048() {
        assert_eq!(BinRange::from(0..2048), choose_bins(2048, 0));
    }
    #[test]
    fn test_center_2047() {
        assert_eq!(BinRange::from(1..2048), choose_bins(2047, 0));
    }
    #[test]
    fn test_center_2046() {
        assert_eq!(BinRange::from(1..2047), choose_bins(2046, 0));
    }
    #[test]
    fn test_center_2045() {
        assert_eq!(BinRange::from(2..2047), choose_bins(2045, 0));
    }
    #[test]
    fn test_center_2044() {
        assert_eq!(BinRange::from(2..2046), choose_bins(2044, 0));
    }
    #[test]
    fn test_center_1() {
        assert_eq!(BinRange::from(1024..1025), choose_bins(1, 0));
    }
    #[test]
    fn test_center_2() {
        assert_eq!(BinRange::from(1023..1025), choose_bins(2, 0));
    }
    #[test]
    fn test_center_3() {
        assert_eq!(BinRange::from(1023..1026), choose_bins(3, 0));
    }

    #[test]
    fn test_offset_high_1() {
        assert_eq!(BinRange::from(2047..2048), choose_bins(1, 1023));
    }
    #[test]
    fn test_offset_low_1() {
        assert_eq!(BinRange::from(0..1), choose_bins(1, -1024));
    }

    #[test]
    fn test_low_end() {
        assert_eq!(BinRange::from(21..472), choose_bins(451, -778));
    }
    #[test]
    fn test_off_low_end() {
        assert_eq!(BinRange::from(0..512), choose_bins(1024, -1024));
    }
}
