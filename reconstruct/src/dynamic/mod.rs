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
//! Dynamic channel detection and decoder management
//!

use crate::bins::choice::choose_bins;
use crate::{DEFAULT_COMPRESSED_BANDWIDTH, NATIVE_FFT_SIZE};
use sparsdr_bin_mask::BinMask;

/// Assumed capture center frequency 2.45 GHz
const CENTER_FREQUENCY: f32 = 2.45e9;

/// A channel, identified by a range of bins, which may be active
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Channel {
    /// The bins that represent this channel
    bins: BinMask,
    /// A unique name for this channel
    name: String,
}

impl Channel {
    /// Creates a new channel
    pub fn new(name: String, bins: BinMask) -> Self {
        Channel { bins, name }
    }

    /// Returns the name of this channel
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the bin mask
    pub fn bins(&self) -> &BinMask {
        &self.bins
    }
}

/// A channel, identified by a range of bins, which may be active
#[derive(Debug)]
pub struct DetectedChannel {
    /// The channel
    channel: Channel,
    /// If the channel is active
    active: bool,
    /// If the channel was previously active
    prev_active: bool,
}

impl DetectedChannel {
    /// Creates a new inactive channel
    pub fn new(channel: Channel) -> Self {
        DetectedChannel {
            channel,
            active: false,
            prev_active: false,
        }
    }

    /// Returns true if this channel is active
    pub fn active(&self) -> bool {
        self.active
    }

    /// Sets the channel active or inactive
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn prev_active(&self) -> bool {
        self.prev_active
    }

    pub fn set_prev_active(&mut self, prev_active: bool) {
        self.prev_active = prev_active
    }

    /// Returns the name of this channel
    pub fn name(&self) -> &str {
        self.channel.name()
    }

    /// Returns the channel
    pub fn channel(&self) -> &Channel {
        &self.channel
    }
}

pub fn find_active_channels<'c, I>(mut bins: BinMask, channel_groups: I)
where
    I: IntoIterator<Item = &'c mut [DetectedChannel]>,
{
    // Portion of bins that must overlap for a channel to be considered active
    let match_threshold = 0.3f32;

    //    eprintln!("find_active_channels");

    for group in channel_groups.into_iter() {
        // Mark all channels inactive
        for channel in group.iter_mut() {
            channel.set_active(false);
        }

        while let Some((matched_channel, overlap)) = best_match_channel(&bins, group) {
            //            eprintln!(
            //                "Channel {} overlap {} / {}",
            //                matched_channel.channel().name(),
            //                overlap,
            //                matched_channel.channel().bins().count_ones()
            //            );

            // Check overlap ratio
            if f32::from(overlap)
                >= match_threshold * matched_channel.channel().bins().count_ones() as f32
            {
                // Set channel active
                matched_channel.set_active(true);
                // Clear those bins so that they don't get detected as another channel
                bins.clear_bits(&matched_channel.channel().bins());
            } else {
                // All other channels are worse than this, so ignore the rest of the channels in
                // this group
                break;
            }
        }
    }
}

/// Matches a set of channels against a bin mask, returning the channel with the best match
/// and the number of matching bins
fn best_match_channel<'c>(
    bins: &BinMask,
    channels: &'c mut [DetectedChannel],
) -> Option<(&'c mut DetectedChannel, u16)> {
    channels.iter_mut().fold(None, |best, channel| {
        let overlap = match_channel(bins, channel.channel());
        if let Some((best_channel, best_overlap)) = best {
            if overlap > best_overlap {
                // Change best
                Some((channel, overlap))
            } else {
                // Don't change
                Some((best_channel, best_overlap))
            }
        } else {
            // First channel, initial best
            Some((channel, overlap))
        }
    })
}

/// Matches a channel against a bin mask, and returns the number of bins of overlap
fn match_channel(bins: &BinMask, channel: &Channel) -> u16 {
    let overlap = channel.bins() & bins;
    return overlap.count_ones() as u16;
}

pub fn make_wifi_bluetooth_ble_channels() -> Vec<Vec<Channel>> {
    vec![
        // Wi-Fi channels
        vec![
            wifi_channel(2_412_000_000, "Wi-Fi 1"),
            wifi_channel(2_417_000_000, "Wi-Fi 2"),
            wifi_channel(2_422_000_000, "Wi-Fi 3"),
            wifi_channel(2_427_000_000, "Wi-Fi 4"),
            wifi_channel(2_432_000_000, "Wi-Fi 5"),
            wifi_channel(2_437_000_000, "Wi-Fi 6"),
            wifi_channel(2_442_000_000, "Wi-Fi 7"),
            wifi_channel(2_447_000_000, "Wi-Fi 8"),
            wifi_channel(2_452_000_000, "Wi-Fi 9"),
            wifi_channel(2_457_000_000, "Wi-Fi 10"),
            wifi_channel(2_462_000_000, "Wi-Fi 11"),
        ],
        // BLE channels
        // Bluetooth core specification v5.0 volume 6 part B 1.4.1
        vec![
            bluetooth_ble_channel(2_402_000_000, "BLE 37"),
            bluetooth_ble_channel(2_404_000_000, "BLE 0"),
            bluetooth_ble_channel(2_406_000_000, "BLE 1"),
            bluetooth_ble_channel(2_408_000_000, "BLE 2"),
            bluetooth_ble_channel(2_410_000_000, "BLE 3"),
            bluetooth_ble_channel(2_412_000_000, "BLE 4"),
            bluetooth_ble_channel(2_414_000_000, "BLE 5"),
            bluetooth_ble_channel(2_416_000_000, "BLE 6"),
            bluetooth_ble_channel(2_418_000_000, "BLE 7"),
            bluetooth_ble_channel(2_420_000_000, "BLE 8"),
            bluetooth_ble_channel(2_422_000_000, "BLE 9"),
            bluetooth_ble_channel(2_424_000_000, "BLE 10"),
            bluetooth_ble_channel(2_426_000_000, "BLE 38"),
            bluetooth_ble_channel(2_428_000_000, "BLE 11"),
            bluetooth_ble_channel(2_430_000_000, "BLE 12"),
            bluetooth_ble_channel(2_432_000_000, "BLE 13"),
            bluetooth_ble_channel(2_434_000_000, "BLE 14"),
            bluetooth_ble_channel(2_436_000_000, "BLE 15"),
            bluetooth_ble_channel(2_438_000_000, "BLE 16"),
            bluetooth_ble_channel(2_440_000_000, "BLE 17"),
            bluetooth_ble_channel(2_442_000_000, "BLE 18"),
            bluetooth_ble_channel(2_444_000_000, "BLE 19"),
            bluetooth_ble_channel(2_446_000_000, "BLE 20"),
            bluetooth_ble_channel(2_448_000_000, "BLE 21"),
            bluetooth_ble_channel(2_450_000_000, "BLE 22"),
            bluetooth_ble_channel(2_452_000_000, "BLE 23"),
            bluetooth_ble_channel(2_454_000_000, "BLE 24"),
            bluetooth_ble_channel(2_456_000_000, "BLE 25"),
            bluetooth_ble_channel(2_458_000_000, "BLE 26"),
            bluetooth_ble_channel(2_460_000_000, "BLE 27"),
            bluetooth_ble_channel(2_462_000_000, "BLE 28"),
            bluetooth_ble_channel(2_464_000_000, "BLE 29"),
            bluetooth_ble_channel(2_466_000_000, "BLE 30"),
            bluetooth_ble_channel(2_468_000_000, "BLE 31"),
            bluetooth_ble_channel(2_470_000_000, "BLE 32"),
            bluetooth_ble_channel(2_472_000_000, "BLE 33"),
            bluetooth_ble_channel(2_474_000_000, "BLE 34"),
            bluetooth_ble_channel(2_476_000_000, "BLE 35"),
            bluetooth_ble_channel(2_478_000_000, "BLE 36"),
            bluetooth_ble_channel(2_480_000_000, "BLE 39"),
        ],
        // Then Bluetooth-only channels
        // Bluetooth core specification v5.0 volume 2 section 2
        vec![
            bluetooth_ble_channel(2_403_000_000, "Bluetooth 1"),
            bluetooth_ble_channel(2_405_000_000, "Bluetooth 3"),
            bluetooth_ble_channel(2_407_000_000, "Bluetooth 5"),
            bluetooth_ble_channel(2_409_000_000, "Bluetooth 7"),
            bluetooth_ble_channel(2_411_000_000, "Bluetooth 9"),
            bluetooth_ble_channel(2_413_000_000, "Bluetooth 11"),
            bluetooth_ble_channel(2_415_000_000, "Bluetooth 13"),
            bluetooth_ble_channel(2_417_000_000, "Bluetooth 15"),
            bluetooth_ble_channel(2_419_000_000, "Bluetooth 17"),
            bluetooth_ble_channel(2_421_000_000, "Bluetooth 19"),
            bluetooth_ble_channel(2_423_000_000, "Bluetooth 21"),
            bluetooth_ble_channel(2_425_000_000, "Bluetooth 23"),
            bluetooth_ble_channel(2_427_000_000, "Bluetooth 25"),
            bluetooth_ble_channel(2_429_000_000, "Bluetooth 27"),
            bluetooth_ble_channel(2_431_000_000, "Bluetooth 29"),
            bluetooth_ble_channel(2_433_000_000, "Bluetooth 31"),
            bluetooth_ble_channel(2_435_000_000, "Bluetooth 33"),
            bluetooth_ble_channel(2_437_000_000, "Bluetooth 35"),
            bluetooth_ble_channel(2_439_000_000, "Bluetooth 37"),
            bluetooth_ble_channel(2_441_000_000, "Bluetooth 39"),
            bluetooth_ble_channel(2_443_000_000, "Bluetooth 41"),
            bluetooth_ble_channel(2_445_000_000, "Bluetooth 43"),
            bluetooth_ble_channel(2_447_000_000, "Bluetooth 45"),
            bluetooth_ble_channel(2_449_000_000, "Bluetooth 47"),
            bluetooth_ble_channel(2_451_000_000, "Bluetooth 49"),
            bluetooth_ble_channel(2_453_000_000, "Bluetooth 51"),
            bluetooth_ble_channel(2_455_000_000, "Bluetooth 53"),
            bluetooth_ble_channel(2_457_000_000, "Bluetooth 55"),
            bluetooth_ble_channel(2_459_000_000, "Bluetooth 57"),
            bluetooth_ble_channel(2_461_000_000, "Bluetooth 59"),
            bluetooth_ble_channel(2_453_000_000, "Bluetooth 61"),
            bluetooth_ble_channel(2_465_000_000, "Bluetooth 63"),
            bluetooth_ble_channel(2_467_000_000, "Bluetooth 65"),
            bluetooth_ble_channel(2_469_000_000, "Bluetooth 67"),
            bluetooth_ble_channel(2_471_000_000, "Bluetooth 69"),
            bluetooth_ble_channel(2_473_000_000, "Bluetooth 71"),
            bluetooth_ble_channel(2_475_000_000, "Bluetooth 73"),
            bluetooth_ble_channel(2_477_000_000, "Bluetooth 75"),
            bluetooth_ble_channel(2_479_000_000, "Bluetooth 77"),
        ],
    ]
}

fn bluetooth_ble_channel(frequency: u64, name: &str) -> Channel {
    eprintln!("B/LE {}", name);
    Channel {
        bins: mask_from_bluetooth_ble_frequency(frequency),
        name: String::from(name),
    }
}

fn wifi_channel(frequency: u64, name: &str) -> Channel {
    Channel {
        bins: mask_from_wifi_frequency(frequency),
        name: String::from(name),
    }
}

fn mask_from_wifi_frequency(frequency: u64) -> BinMask {
    // 22 MHz, 451 bins
    let bins = 451;
    let relative_frequency = frequency as f32 - CENTER_FREQUENCY;
    let exact_bin_offset =
        f32::from(NATIVE_FFT_SIZE) * relative_frequency / DEFAULT_COMPRESSED_BANDWIDTH;
    let fc_bins = exact_bin_offset.floor();

    let bin_range = choose_bins(bins, fc_bins as i16);

    let mut mask = BinMask::zero();
    mask.set_range(bin_range.as_usize_range());
    mask
}

fn mask_from_bluetooth_ble_frequency(frequency: u64) -> BinMask {
    let bins = if frequency % 2_000_000 == 0 {
        // BLE channel, 2 MHz
        // 41 bins
        41
    } else {
        // Regular Bluetooth channel, 1 MHz
        // 21 bins
        21
    };

    let relative_frequency = frequency as f32 - CENTER_FREQUENCY;
    let exact_bin_offset =
        f32::from(NATIVE_FFT_SIZE) * relative_frequency / DEFAULT_COMPRESSED_BANDWIDTH;
    let fc_bins = exact_bin_offset.floor();

    let bin_range = choose_bins(bins, fc_bins as i16);
    eprintln!("Bins {}", bin_range);

    let mut mask = BinMask::zero();
    mask.set_range(bin_range.as_usize_range());
    mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::{DerefMut, Range};

    fn bin_range(bins: Range<usize>) -> DetectedChannel {
        let mut mask = BinMask::zero();
        mask.set_range(bins);
        DetectedChannel::new(Channel::new(String::from("test"), mask))
    }

    #[test]
    fn test_none_active() {
        let mut channels = [[bin_range(10..32)]];
        let active = BinMask::zero();

        find_active_channels(active, channels.iter_mut().map(|group| &mut group[..]));

        assert!(!channels[0][0].active());
    }

    #[test]
    fn test_one_active() {
        let mut channels = [[bin_range(10..32)]];
        let mut active = BinMask::zero();
        active.set_range(10..32);

        find_active_channels(active, channels.iter_mut().map(|group| &mut group[..]));

        assert!(channels[0][0].active());
    }

    #[test]
    fn test_one_active_larger() {
        let mut channels = [[bin_range(10..32)]];
        let mut active = BinMask::zero();
        active.set_range(9..33);

        find_active_channels(active, channels.iter_mut().map(|group| &mut group[..]));

        assert!(channels[0][0].active());
    }

    #[test]
    fn test_wide_and_narrow() {
        let mut channels = [[bin_range(10..32)], [bin_range(12..20)]];
        let mut active = BinMask::zero();
        active.set_range(9..33);

        find_active_channels(active, channels.iter_mut().map(|group| &mut group[..]));

        // Wider channel active, narrower channel not active because the wider channel has claimed
        // those bins
        assert!(channels[0][0].active());
        assert!(!channels[1][0].active());
    }
}
