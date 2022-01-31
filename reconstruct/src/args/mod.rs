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

//! Command-line arguments (used only in the decompress binary)

mod band_args;

use std::convert::TryInto;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{App, Arg};
use simplelog::LevelFilter;
use sparsdr_reconstruct::steps::overlap::OverlapMode;

pub use self::band_args::BandArgs;

#[derive(Debug)]
#[non_exhaustive]
pub struct Args {
    /// Path to source file, or None for stdin
    pub source_path: Option<PathBuf>,
    /// Enable buffering for source and destination
    pub buffer: bool,
    /// Bandwidth of the signal before compression
    pub compressed_bandwidth: f32,
    /// Size of the FFT used for compression
    pub compression_fft_size: usize,
    /// Number of bits in the window timestamp counter
    pub timestamp_bits: u32,
    /// The compressed sample format
    pub sample_format: CompressedFormat,
    /// Bands to decompress
    pub bands: Vec<BandArgs>,
    /// Log level
    pub log_level: LevelFilter,
    /// Flag to enable progress bar
    pub progress_bar: bool,
    /// Capacity of input -> FFT/output stage channels
    pub channel_capacity: usize,
    /// The overlap mode
    pub overlap: OverlapMode,
}

/// General help text
const ABOUT: &str = include_str!("about.txt");

/// Default FFT size for the N210 compression
const N210_DEFAULT_COMPRESSION_FFT_SIZE: usize = 2048;
/// Bandwidth/sample rate for N210 compression
const N210_COMPRESSED_BANDWIDTH: f32 = 100e6;
/// Default FFT size for the Pluto compression
const PLUTO_DEFAULT_COMPRESSION_FFT_SIZE: usize = 1024;
/// Bandwidth/sample rate for Pluto compression
const PLUTO_COMPRESSED_BANDWIDTH: f32 = 61.44e6;
const N210_TIMESTAMP_BITS: u32 = 20;
const PLUTO_TIMESTAMP_BITS: u32 = 21;
/// Compressed format version 2, from either device, has 30 full bits of timestamp
const V2_TIMESTAMP_BITS: u32 = 30;

impl Args {
    pub fn get() -> Self {
        let matches = App::new(crate_name!())
            .version(crate_version!())
            .about(ABOUT)
            .author(crate_authors!())
            .arg(
                Arg::with_name("source")
                    .long("source")
                    .takes_value(true)
                    .value_name("path")
                    .help(
                        "A file to read compressed samples from. If no file is specified, samples \
                         will be read from standard input.",
                    ),
            )
            .arg(
                Arg::with_name("destination")
                    .long("destination")
                    .takes_value(true)
                    .value_name("path")
                    .help(
                        "A file to write uncompressed samples to. If no file is specified, \
                         samples will be written to standard output.",
                    ),
            )
            .arg(
                Arg::with_name("bins")
                    .long("bins")
                    .takes_value(true)
                    .validator(validate_positive_even_number)
                    .help(
                        "The number of bins to reconstruct. This must be an even number. \
                        The default value is the number of bins in the FFT used for compression.",
                    ),
            )
            .arg(
                Arg::with_name("reconstruct_fft_bins")
                    .long("reconstruct-fft-bins")
                    .takes_value(true)
                    .validator(validate_positive_even_number)
                    .help(
                        "The FFT size to use when reconstructing. This must be an even number \
                    greater than or equal to the \"bins\" parameter. This can be used to generate \
                    reconstructed samples with a higher sample rate, without expanding the \
                    frequency range. The default value is the same as the \"bins\" parameter",
                    ),
            )
            .arg(
                Arg::with_name("center_frequency")
                    .long("center-frequency")
                    .takes_value(true)
                    .default_value("0")
                    .validator(validate::<f32>)
                    .value_name("hertz")
                    .help(
                        "The desired center frequency of the decompressed signal, relative to \
                         the center frequency of the compressed data.",
                    ),
            )
            .arg(
                Arg::with_name("compressed_bandwidth")
                    .long("compressed-bandwidth")
                    .takes_value(true)
                    .validator(validate::<f32>)
                    .value_name("hertz")
                    .required_unless_one(&[
                        "n210_v1_defaults",
                        "n210_v2_defaults",
                        "pluto_v1_defaults",
                        "pluto_v2_defaults",
                    ])
                    .help("The bandwidth of the signal before compression, in hertz"),
            )
            .arg(
                Arg::with_name("compression_fft_size")
                    .long("compression-fft-size")
                    .takes_value(true)
                    .validator(validate::<usize>)
                    .required_unless_one(&[
                        "n210_v1_defaults",
                        "n210_v2_defaults",
                        "pluto_v1_defaults",
                        "pluto_v2_defaults",
                    ])
                    .help("The number of bins in the FFT used to compress the received signal"),
            )
            .arg(
                Arg::with_name("sample_format")
                    .long("sample-format")
                    .takes_value(true)
                    .validator(validate::<CompressedFormat>)
                    .possible_values(&["v1-n210", "v1-pluto", "v2"])
                    .required_unless_one(&[
                        "n210_v1_defaults",
                        "n210_v2_defaults",
                        "pluto_v1_defaults",
                        "pluto_v2_defaults",
                    ])
                    .help(
                        "The compressed sample format to use. This depends on the image \
                loaded onto the radio.",
                    ),
            )
            .arg(
                Arg::with_name("timestamp_bits")
                    .long("timestamp-bits")
                    .takes_value(true)
                    .validator(validate::<u32>)
                    .required_unless_one(&[
                        "n210_v1_defaults",
                        "n210_v2_defaults",
                        "pluto_v1_defaults",
                        "pluto_v2_defaults",
                    ])
                    .help(
                        "The number of bits in the FPGA's window timestamp counter that are sent \
                (used to correct timestamp overflow)",
                    ),
            )
            .arg(
                Arg::with_name("n210_v1_defaults")
                    .long("n210-v1-defaults")
                    .conflicts_with_all(&[
                        "n210_v2_defaults",
                        "pluto_v1_defaults",
                        "pluto_v2_defaults",
                        "compressed_bandwidth",
                        "compression_fft_size",
                        "sample_format",
                        "timestamp_bits",
                    ])
                    .help(
                        "Sets default values for a USRP N210 using compressed sample format \
                version 1 (equivalent to --compressed-bandwidth 100e6 --compression-fft-size \
                2048 --sample-format v1-n210 --timestamp-bits 20)",
                    ),
            )
            .arg(
                Arg::with_name("n210_v2_defaults")
                    .long("n210-v2-defaults")
                    .conflicts_with_all(&[
                        "n210_v1_defaults",
                        "pluto_v1_defaults",
                        "pluto_v2_defaults",
                        "compressed_bandwidth",
                        "compression_fft_size",
                        "sample_format",
                        "timestamp_bits",
                    ])
                    .help(
                        "Sets default values for a USRP N210 using compressed sample format \
                version 2 (equivalent to --compressed-bandwidth 100e6 --compression-fft-size \
                2048 --sample-format v2 --timestamp-bits 30)",
                    ),
            )
            .arg(
                Arg::with_name("pluto_v1_defaults")
                    .long("pluto-v1-defaults")
                    .conflicts_with_all(&[
                        "n210_v1_defaults",
                        "n210_v2_defaults",
                        "pluto_v2_defaults",
                        "compressed_bandwidth",
                        "compression_fft_size",
                        "sample_format",
                        "timestamp_bits",
                    ])
                    .help(
                        "Sets default values for a Pluto using compressed sample format \
                version 1 (equivalent to --compressed-bandwidth 61.44e6 --compression-fft-size \
                1024 --sample-format v1-pluto --timestamp-bits 21)",
                    ),
            )
            .arg(
                Arg::with_name("pluto_v2_defaults")
                    .long("pluto-v2-defaults")
                    .conflicts_with_all(&[
                        "n210_v1_defaults",
                        "n210_v2_defaults",
                        "pluto_v1_defaults",
                        "compressed_bandwidth",
                        "compression_fft_size",
                        "sample_format",
                        "timestamp_bits",
                    ])
                    .help(
                        "Sets default values for a Pluto using compressed sample format \
                version 2 (equivalent to --compressed-bandwidth 61.44e6 --compression-fft-size \
                1024 --sample-format v2 --timestamp-bits 30)",
                    ),
            )
            .arg(
                Arg::with_name("unbuffered")
                    .long("unbuffered")
                    .help("Disables buffering on the source and destination"),
            )
            .arg(
                Arg::with_name("log_level")
                    .long("log-level")
                    .takes_value(true)
                    .default_value("WARN")
                    .possible_values(&["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"])
                    .help("The level of logging to enable"),
            )
            .arg(
                Arg::with_name("reconstruct_band")
                    .long("reconstruct-band")
                    .takes_value(true)
                    .multiple(true)
                    .value_name("bins:fft_bins:frequency[:path]")
                    .help(
                        "The number of bins, center frequency, and output file path of a band to \
                    be decompressed. If the output file path is not specified, decompressed \
                    samples from this band will be written to standard output. This argument may \
                    be repeated to decompress multiple bands. \
                    \"bins\" and \"fft_bins\" must be positive and even. fft_bins must be greater \
                    than or equal to bins. \
                    The frequency is relative to the center frequency used to receive the samples",
                    )
                    .conflicts_with_all(&[
                        "destination",
                        "bins",
                        "reconstruct_fft_bins",
                        "center_frequency",
                    ])
                    .validator(validate::<BandArgs>),
            )
            .arg(
                Arg::with_name("no_progress")
                    .long("no-progress-bar")
                    .help("Disables the command-line progress bar"),
            )
            .arg(
                Arg::with_name("channel_capacity")
                    .long("channel-capacity")
                    .takes_value(true)
                    .default_value("32")
                    .validator(validate::<usize>)
                    .value_name("windows")
                    .help(
                        "Capacity of input -> FFT/output stage channels (this option is unstable)",
                    ),
            )
            .arg(
                Arg::with_name("flush_samples")
                    .long("flush-samples")
                    .takes_value(true)
                    .default_value("0")
                    .validator(validate::<u32>)
                    .value_name("samples")
                    .help("The number of output zero samples written in time gaps"),
            )
            .arg(
                Arg::with_name("zero_gaps")
                .long("zero-gaps")
                .conflicts_with("flush_samples")
                .help("Produces zero samples in the output file(s) representing periods with no active signals")
            )
            .get_matches();

        let buffer = !matches.is_present("unbuffered");

        let (compression_fft_size, compressed_bandwidth, sample_format, timestamp_bits) =
            if matches.is_present("n210_v1_defaults") {
                (
                    N210_DEFAULT_COMPRESSION_FFT_SIZE,
                    N210_COMPRESSED_BANDWIDTH,
                    CompressedFormat::V1N210,
                    N210_TIMESTAMP_BITS,
                )
            } else if matches.is_present("n210_v2_defaults") {
                (
                    N210_DEFAULT_COMPRESSION_FFT_SIZE,
                    N210_COMPRESSED_BANDWIDTH,
                    CompressedFormat::V2,
                    V2_TIMESTAMP_BITS,
                )
            } else if matches.is_present("pluto_v1_defaults") {
                (
                    PLUTO_DEFAULT_COMPRESSION_FFT_SIZE,
                    PLUTO_COMPRESSED_BANDWIDTH,
                    CompressedFormat::V1Pluto,
                    PLUTO_TIMESTAMP_BITS,
                )
            } else if matches.is_present("pluto_v2_defaults") {
                (
                    PLUTO_DEFAULT_COMPRESSION_FFT_SIZE,
                    PLUTO_COMPRESSED_BANDWIDTH,
                    CompressedFormat::V2,
                    V2_TIMESTAMP_BITS,
                )
            } else {
                // Values must be specified individually
                (
                    matches
                        .value_of("compression_fft_size")
                        .unwrap()
                        .parse()
                        .unwrap(),
                    matches
                        .value_of("compressed_bandwidth")
                        .unwrap()
                        .parse()
                        .unwrap(),
                    matches.value_of("sample_format").unwrap().parse().unwrap(),
                    matches.value_of("timestamp_bits").unwrap().parse().unwrap(),
                )
            };

        let bands = if let Some(band_strings) = matches.values_of("reconstruct_band") {
            // New multi-band version
            band_strings
                .map(|s| BandArgs::from_str(s).unwrap())
                .collect()
        } else {
            // Legacy single-band version
            let bins = matches
                .value_of("bins")
                .map(|s| s.parse().unwrap())
                .unwrap_or_else(|| compression_fft_size.try_into().expect("FFT size too large"));

            let fft_bins = matches
                .value_of("reconstruct_fft_bins")
                .map(|s| s.parse().unwrap())
                .unwrap_or(bins);
            assert!(
                bins <= fft_bins,
                "Number of reconstruct FFT bins {} \
            must be greater than or equal to number of bins {}",
                fft_bins,
                bins
            );

            let band = BandArgs {
                bins,
                fft_bins,
                center_frequency: matches
                    .value_of("center_frequency")
                    .unwrap()
                    .parse()
                    .unwrap(),
                path: matches.value_of("destination").map(PathBuf::from),
            };
            vec![band]
        };

        let overlap = if matches.is_present("zero_gaps") {
            OverlapMode::Gaps
        } else {
            let flush_samples = matches
                .value_of("flush_samples")
                .map(|samples| samples.parse().unwrap())
                .unwrap_or(0);
            OverlapMode::Flush(flush_samples)
        };

        Args {
            source_path: matches.value_of_os("source").map(PathBuf::from),
            buffer,
            compressed_bandwidth,
            compression_fft_size,
            timestamp_bits,
            sample_format,
            bands,
            log_level: matches.value_of("log_level").unwrap().parse().unwrap(),
            progress_bar: !matches.is_present("no_progress"),
            channel_capacity: matches
                .value_of("channel_capacity")
                .unwrap()
                .parse()
                .unwrap(),
            overlap,
        }
    }
}

/// Validates that a string can be parsed into a value of type T
// As required by clap, this function accepts a String.
#[allow(clippy::needless_pass_by_value)]
fn validate<T>(s: String) -> Result<(), String>
where
    T: FromStr,
    T::Err: ToString,
{
    s.parse::<T>().map(|_| ()).map_err(|e| e.to_string())
}

/// Validates that a string can be parsed into a posiitve, even 16-bit integer
// As required by clap, this function accepts a String.
#[allow(clippy::needless_pass_by_value)]
fn validate_positive_even_number(s: String) -> Result<(), String> {
    s.parse::<u16>()
        .map_err(|e| e.to_string())
        .and_then(|value| {
            if value % 2 != 0 {
                Err("Value must be an even number".into())
            } else if value < 2 {
                Err("Value must be positive".into())
            } else {
                Ok(())
            }
        })
}

/// A compressed sample format
#[derive(Debug)]
pub enum CompressedFormat {
    /// Version 1 as produced by a USRP N210
    V1N210,
    /// Version 1 as produced by a Pluto
    V1Pluto,
    /// Version 2 from either device
    V2,
}

impl FromStr for CompressedFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "v1-n210" => Ok(CompressedFormat::V1N210),
            "v1-pluto" => Ok(CompressedFormat::V1Pluto),
            "v2" => Ok(CompressedFormat::V2),
            _ => Err("Invalid compressed format, expected v1-n210, v1-pluto, or v2"),
        }
    }
}
