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
#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Compression {
    pub thresholds: Vec<ThresholdRange>,
    pub masks: Vec<Range<u16>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ThresholdRange {
    pub threshold: u32,
    pub bins: Range<u16>,
}

/// Information about a band to reconstruct and where to write its output
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Band {
    pub bins: u16,
    pub frequency: f32,
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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Output {
    Stdout,
    File {
        path: PathBuf,
    },
    TcpClient {
        server_address: SocketAddr,
    },
    Udp {
        #[serde(default = "any_address")]
        local_address: SocketAddr,
        remote_address: SocketAddr,
        #[serde(default)]
        header_format: UdpHeaderFormat,
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

impl Default for UdpHeaderFormat {
    fn default() -> Self {
        UdpHeaderFormat::None
    }
}

/// Returns an address that allows binding to an unspecified IP address and port
fn any_address() -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
}
