/*
 * Copyright 2020 The Regents of the University of California
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

extern crate clap;
extern crate log;
extern crate serde;
extern crate toml;

mod cli;
mod custom_de;
#[cfg(test)]
mod deserialize_test;

use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::ops::Range;
use std::path::PathBuf;

pub use crate::cli::config_from_command_line;

/// A configuration file for sparsdr_reconstruct
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Config {
    /// Input settings (required)
    pub source: Input,
    /// Application user interface settings (optional)
    #[serde(default)]
    pub ui: UserInterface,
    /// Reconstruction tuning (optional)
    #[serde(default)]
    pub tuning: Tuning,
    /// Reconstruction bands and outputs (at least one required)
    #[serde(deserialize_with = "crate::custom_de::deserialize_non_empty_vec")]
    pub bands: Vec<Band>,
}

/// Information about where to read the compressed samples
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Input {
    /// Read compressed samples from standard input in the specified format
    Stdin { format: Format },
    /// Read compressed samples from a file (which may be a named pipe) in the specified format
    File { path: PathBuf, format: Format },
    /// Connect to a USRP N210, enable compression, and read samples
    N210 {
        /// Center frequency, hertz (none = hardware default)
        frequency: Option<f64>,
        /// Gain, decibels (none = hardware default)
        gain: Option<f64>,
        /// Antenna name (none = hardware default)
        antenna: Option<String>,
        /// Compression options
        #[serde(default)]
        compression: Compression,
        /// Arguments to pass to UHD
        #[serde(default)]
        #[serde(deserialize_with = "crate::custom_de::permissive_deserialize_string_map")]
        args: BTreeMap<String, String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
pub enum Format {
    N210,
    Pluto,
}

/// Options for capturing and compressing signals
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Compression {
    /// Thresholds for ranges of bins
    #[serde(default)]
    pub thresholds: Vec<ThresholdRange>,
    /// Bins to mask
    ///
    /// Masked bins will not be sent from the USRP to the host.
    ///
    /// Regardless of this value, the three center bins will always be masked. These bins
    /// are nearly always active because they are at the radio's center frequency. When the FFT
    /// size is 2048, these three bins span about 146 kHz.
    ///
    /// These bin numbers are in logical order (bin 0 is the lowest frequency and the maximum
    /// bin is the highest frequency).
    #[serde(default)]
    pub masks: Vec<Range<u16>>,
    /// Base-2 logarithm of the FFT size used for compression
    ///
    /// This should normally be 11 (corresponding to 2048 bins). Other values have not been tested.
    #[serde(default = "default_fft_size_logarithm")]
    pub fft_size_logarithm: u32,
    /// FFT scaling (what is this?)
    #[serde(default = "default_fft_scaling")]
    pub fft_scaling: u32,
    /// The weight factor used when updating averages, between 0 and 1. One extreme changes the
    /// average each time an FFT is run, and the other extreme never changes the average.
    /// Which is which?
    #[serde(
        default = "default_average_weight",
        deserialize_with = "crate::custom_de::deserialize_0_1"
    )]
    pub average_weight: f32,
    /// The interval between average samples that the USRP sends
    ///
    /// This interval is in units of FFT windows. For example, an interval of 1 causes the USRP
    /// to send averages after every set of FFT samples. The interval will be rounded to
    /// a power of two.
    #[serde(default = "default_average_interval")]
    pub average_sample_interval: u32,
}

/// Returns the default size (11 => 2^11 = 2048 bins)
fn default_fft_size_logarithm() -> u32 {
    11
}

/// Returns the default FFT scaling
fn default_fft_scaling() -> u32 {
    // Why this value?
    0x6ab
}

/// Returns the default average sample interval
fn default_average_interval() -> u32 {
    // 2^12
    1 << 12
}

/// Returns the default average weight
fn default_average_weight() -> f32 {
    0.85
}

impl Default for Compression {
    fn default() -> Self {
        Compression {
            thresholds: Default::default(),
            masks: Default::default(),
            fft_size_logarithm: default_fft_size_logarithm(),
            fft_scaling: default_fft_scaling(),
            average_weight: default_average_weight(),
            average_sample_interval: default_average_interval(),
        }
    }
}

/// A range of bins and a threshold that applies to those bins
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ThresholdRange {
    /// The threshold for these bins
    pub threshold: u32,
    /// The bins to apply to
    ///
    /// These bin numbers are in logical order (bin 0 is the lowest frequency and the maximum
    /// bin is the highest frequency).
    pub bins: Range<u16>,
}

/// Reconstruction performance tuning
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Tuning {
    /// Capacity of channels used to send samples from the main (input) thread to reconstruction
    /// threads
    ///
    /// Larger values take up more memory, but may reduce problems caused by not reading samples
    /// from the USRP frequently enough.
    pub channel_capacity: usize,
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning {
            // This channel capacity works well on Rasberry Pis and more powerful computers.
            channel_capacity: 128,
        }
    }
}

/// Information about a band to reconstruct and where to write its output
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Band {
    /// Number of FFT bins used for reconstruction
    ///
    /// The minimum value is 2, and the maximum is the FFT size used for compression.
    pub bins: u16,
    /// The frequency to reconstruct for this band, in hertz relative to the center frequency
    /// used for compression
    pub frequency: f32,
    /// The output to write reconstructed samples to
    pub destination: Output,
}

/// User interface options
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct UserInterface {
    /// Minimum log level to print
    #[serde(default = "log_level_warn")]
    pub log_level: LevelFilter,
}

impl Default for UserInterface {
    fn default() -> Self {
        UserInterface {
            log_level: log_level_warn(),
        }
    }
}

fn log_level_warn() -> LevelFilter {
    LevelFilter::Warn
}

/// How to write the reconstructed samples from a band
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Output {
    /// Write files to standard output
    Stdout,
    /// Write samples to a file
    File {
        /// The path to the file
        path: PathBuf,
    },
    /// Connect to a TCP server and send samples over a socket
    TcpClient {
        /// Address of the server to connect to
        server_address: SocketAddr,
    },
    /// Send samples over a UDP socket
    Udp {
        /// Local address to bind to
        #[serde(default = "any_address")]
        local_address: SocketAddr,
        /// Remote address to send samples to
        remote_address: SocketAddr,
        /// Format of headers to add to each packet
        #[serde(default)]
        header_format: UdpHeaderFormat,
        /// The maximum number of bytes to send in each packet, including headers added by
        /// the selected header format but excluding the IP and UDP headers
        #[serde(default = "default_mtu")]
        mtu: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "snake_case")]
pub enum UdpHeaderFormat {
    None,
    Sequence,
    SequenceAndLength,
}

/// Returns a header format that uses no headers
impl Default for UdpHeaderFormat {
    fn default() -> Self {
        UdpHeaderFormat::None
    }
}

/// Returns the default MTU of 1472 bytes, which works for most network links
fn default_mtu() -> usize {
    1472
}

/// Returns an address that allows binding to an unspecified IP address and port
fn any_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
}
