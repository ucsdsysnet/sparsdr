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

use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Stdin};

use simplelog::LevelFilter;

use byteorder::NativeEndian;
use sparsdr_reconstruct::input::format::n210::N210SampleReader;
use sparsdr_reconstruct::input::n210::N210;
use sparsdr_reconstruct::input::ReadInput;
use sparsdr_reconstruct::output::stdio::StdioOutput;
use sparsdr_reconstruct::output::udp::{
    NoHeaders, SequenceAndSizeHeaders, SequenceHeaders, UdpOutput,
};
use sparsdr_reconstruct::output::WriteOutput;
use sparsdr_reconstruct_config::{Band, Config, Format, Input, Output, UdpHeaderFormat};
use std::collections::BTreeMap;
use uhd::{TuneRequest, Usrp};

/// The setup for a decompression operation
///
/// A Setup is created from the command-line arguments (Args)
pub struct Setup {
    /// Source for compressed samples
    pub source: Box<dyn ReadInput>,
    /// Size of the source file in bytes, if known
    pub source_length: Option<u64>,
    /// Log level
    pub log_level: LevelFilter,
    /// Bands to decompress
    pub bands: Vec<BandSetup>,
    /// Flag to enable progress bar
    pub progress_bar: bool,
    /// Capacity of input -> FFT/output stage channels
    pub channel_capacity: usize,
}

/// The setup for decompressing a band
pub struct BandSetup {
    /// Number of bins to decompress
    pub bins: u16,
    /// Center frequency to decompress
    pub center_frequency: f32,
    /// Destination to write to
    pub destination: Box<dyn WriteOutput + Send>,
}

impl Setup {
    pub fn from_config(config: &Config) -> Result<Self, Box<dyn Error>> {
        let source: Box<dyn ReadInput> = match &config.source {
            Input::Stdin { format } => {
                // stdin already has a BufReader
                // Use leak to create a lock with 'static lifetime.
                let stdin: &'static Stdin = Box::leak(Box::new(io::stdin()));
                let lock = stdin.lock();
                match format {
                    Format::N210 => Box::new(N210SampleReader::new(lock)),
                    Format::Pluto => unimplemented!("Pluto format is not yet implemented"),
                }
            }
            Input::File { path, format } => {
                let file = BufReader::new(File::open(path)?);
                match format {
                    Format::N210 => Box::new(N210SampleReader::new(file)),
                    Format::Pluto => unimplemented!("Pluto format is not yet implemented"),
                }
            }
            Input::N210 {
                frequency,
                gain,
                antenna,
                compression,
                args,
            } => {
                let usrp = Usrp::open(&join_uhd_args(args))?;
                // USRP setup
                if let Some(frequency) = frequency {
                    usrp.set_rx_frequency(&TuneRequest::with_frequency(*frequency), 0)?;
                }
                if let Some(gain) = gain {
                    // Leave the gain element blank for default gain
                    usrp.set_rx_gain(*gain, 0, "")?;
                }
                if let Some(antenna) = antenna.as_deref() {
                    usrp.set_rx_antenna(antenna, 0)?;
                }
                // Leak to get a 'static Usrp
                let usrp: &'static Usrp = Box::leak(Box::new(usrp));
                let n210 = N210::new(usrp, 0, 0)?;
                // Compression setup
                for mask_range in &compression.masks {
                    for bin in mask_range.clone() {
                        n210.set_mask_enabled(bin, true)?;
                    }
                }
                for threshold_range in &compression.thresholds {
                    for bin in threshold_range.bins.clone() {
                        n210.set_threshold(bin, threshold_range.threshold)?;
                    }
                }

                Box::new(n210)
            }
        };

        let source_length = match &config.source {
            Input::File { path, .. } => fs::metadata(path)
                .ok()
                .map(|metadata| metadata.len())
                .and_then(|length| if length != 0 { Some(length) } else { None }),
            _ => None,
        };

        let log_level = config.ui.log_level;
        // TODO: Restore progress bar option
        let progress_bar = false;
        // TODO: Restore channel capacity option and default value
        let channel_capacity = 32;

        let bands: Vec<BandSetup> = config
            .bands
            .iter()
            .map(BandSetup::from_config)
            .collect::<Result<Vec<BandSetup>, Box<dyn Error>>>()?;

        Ok(Setup {
            source,
            source_length,
            log_level,
            bands,
            progress_bar,
            channel_capacity,
        })
    }
}

/// Joins a key-value map into a comma-separated string
fn join_uhd_args(args: &BTreeMap<String, String>) -> String {
    fn join<I>(mut strings: I, separator: &str) -> String
    where
        I: Iterator<Item = String>,
    {
        match strings.next() {
            None => String::new(),
            Some(first) => {
                let mut buffer = String::with_capacity(strings.size_hint().0 * separator.len());
                buffer.push_str(&first);
                for item in strings {
                    buffer.push_str(separator);
                    buffer.push_str(&item)
                }
                buffer
            }
        }
    }

    join(
        args.iter().map(|(key, value)| format!("{}={}", key, value)),
        ",",
    )
}

impl BandSetup {
    fn from_config(config: &Band) -> Result<Self, Box<dyn Error>> {
        let destination: Box<dyn WriteOutput + Send> = match &config.destination {
            Output::Stdout => Box::new(StdioOutput::new(io::stdout())),
            Output::File { path } => {
                let file = BufWriter::new(File::create(path)?);
                Box::new(StdioOutput::new(file))
            }
            Output::TcpClient { .. } => unimplemented!("TCP output not yet implemented"),
            Output::Udp {
                local_address,
                remote_address,
                header_format,
                mtu,
            } => match header_format {
                UdpHeaderFormat::None => Box::new(UdpOutput::<NativeEndian, NoHeaders>::new(
                    *local_address,
                    *remote_address,
                    *mtu,
                    NoHeaders,
                )?),
                UdpHeaderFormat::Sequence => Box::new(UdpOutput::<
                    NativeEndian,
                    SequenceHeaders<NativeEndian>,
                >::new(
                    *local_address,
                    *remote_address,
                    *mtu,
                    SequenceHeaders::new(),
                )?),
                UdpHeaderFormat::SequenceAndLength => Box::new(UdpOutput::<
                    NativeEndian,
                    SequenceAndSizeHeaders<NativeEndian>,
                >::new(
                    *local_address,
                    *remote_address,
                    *mtu,
                    SequenceAndSizeHeaders::new(),
                )?),
            },
        };

        Ok(BandSetup {
            bins: config.bins,
            center_frequency: config.frequency,
            destination,
        })
    }
}
