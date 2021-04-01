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
use sparsdr_reconstruct::output;
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
pub struct Setup<'source> {
    /// Source for compressed samples
    pub source: Box<dyn ReadInput + 'source>,
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

impl<'source> Setup<'source> {
    /// Creates a setup from a configuration and passes it to the provided operation
    pub fn run_from_config<F, R>(config: &Config, operation: F) -> Result<R, Box<dyn Error>>
    where
        F: FnOnce(Setup<'_>) -> Result<R, Box<dyn Error>>,
    {
        // Storage for values that have references stored in the setup
        let stdin_storage: Stdin;
        let usrp_storage: Option<Usrp>;

        // Convert config
        let source: Box<dyn ReadInput> = match &config.source {
            Input::Stdin { format } => {
                // stdin already has a BufReader
                stdin_storage = io::stdin();
                let lock = stdin_storage.lock();
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
                // Put the USRP in the higher-up storage
                usrp_storage = Some(usrp);
                let mut n210 = N210::new(usrp_storage.as_ref().unwrap(), 0, 0)?;
                // Compression setup: Follow the steps in uhd_rx_compressed_cfile
                // https://github.com/ucsdsysnet/sparsdr/blob/master/examples/uhd_rx_compressed_cfile/uhd_compressed_rx_cfile
                n210.set_compression_enabled(true)?;
                n210.stop_all()?;
                n210.set_fft_size_log2(compression.fft_size_logarithm)?;
                n210.set_fft_scaling(compression.fft_scaling)?;

                // Set up bin conversion
                let fft_size = 1u16 << compression.fft_size_logarithm;

                let logical_to_fft_bin = |logical_bin: u16| -> u16 {
                    if logical_bin >= fft_size / 2 {
                        logical_bin - fft_size / 2
                    } else {
                        logical_bin + fft_size / 2
                    }
                };

                // Set thresholds
                // The configured thresholds have logical bin numbers, which need to be converted
                // to FFT bin numbers for the USRP.
                for threshold_range in &compression.thresholds {
                    for bin in threshold_range.bins.clone() {
                        n210.set_threshold(logical_to_fft_bin(bin), threshold_range.threshold)?;
                    }
                }

                // Clear all masks
                for bin in 0..fft_size {
                    n210.set_mask_enabled(bin, false)?;
                }
                // Set configured masks
                // The configured masks have logical bin numbers, which need to be converted
                // to FFT bin numbers for the USRP.
                for mask_range in &compression.masks {
                    for bin in mask_range.clone() {
                        n210.set_mask_enabled(logical_to_fft_bin(bin), true)?;
                    }
                }
                // Set masks for bins 0, 1, and -1 (FFT bin numbers)
                // If these were converted to logical, they would be the 3 center bins.
                n210.set_mask_enabled(0, true)?;
                n210.set_mask_enabled(1, true)?;
                n210.set_mask_enabled(fft_size - 1, true)?;

                n210.set_average_weight(compression.average_weight)?;
                n210.set_average_packet_interval(compression.average_sample_interval)?;

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

        let channel_capacity = config.tuning.channel_capacity;
        let bands: Vec<BandSetup> = config
            .bands
            .iter()
            .map(BandSetup::from_config)
            .collect::<Result<Vec<BandSetup>, Box<dyn Error>>>()?;

        let setup = Setup {
            source,
            source_length,
            log_level,
            bands,
            progress_bar,
            channel_capacity,
        };

        // Run the provided closure
        operation(setup)
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
            Output::TcpClient { server_address } => {
                Box::new(output::tcp_client(server_address.clone())?)
            }
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

        if config.bins < 2 {
            return Err(SetupError(format!(
                "Can't reconstruct a band with {} bins (the minimum number of bins is 2)",
                config.bins
            ))
            .into());
        }

        Ok(BandSetup {
            bins: config.bins,
            center_frequency: config.frequency,
            destination,
        })
    }
}

/// Errors that occur when creating a setup
#[derive(Debug, Clone)]
struct SetupError(String);

impl Error for SetupError {}

mod fmt {
    use super::SetupError;
    use std::fmt;

    impl fmt::Display for SetupError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Setup error: {}", self.0)
        }
    }
}
