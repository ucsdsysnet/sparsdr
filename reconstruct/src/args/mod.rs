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

use std::path::PathBuf;
use std::str::FromStr;

use clap::{App, Arg};
use simplelog::LevelFilter;

pub use self::band_args::BandArgs;

#[derive(Debug)]
pub struct Args {
    /// Path to source file, or None for stdin
    pub source_path: Option<PathBuf>,
    /// Enable buffering for source and destination
    pub buffer: bool,
    /// Bandwidth of the signal before compression
    pub compressed_bandwidth: f32,
    /// Bands to decompress
    pub bands: Vec<BandArgs>,
    /// Log level
    pub log_level: LevelFilter,
    /// Flag to enable progress bar
    pub progress_bar: bool,
    /// Flag to enable reporting of implementation-defined information
    pub report: bool,
    /// Capacity of input -> FFT/output stage channels
    pub channel_capacity: usize,
    /// Window input time log path
    pub input_time_log_path: Option<PathBuf>,
    /// Private field to prevent exhaustive matching and literal creation
    _0: (),
}

/// General help text
const ABOUT: &str = include_str!("about.txt");

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
            ).arg(
                Arg::with_name("destination")
                    .long("destination")
                    .takes_value(true)
                    .value_name("path")
                    .help(
                        "A file to write uncompressed samples to. If no file is specified, \
                         samples will be written to standard output.",
                    ),
            ).arg(
                Arg::with_name("bins")
                    .long("bins")
                    .takes_value(true)
                    .default_value("2048")
                    .validator(validate::<u16>)
                    .help("The number of bins to decompress"),
            ).arg(
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
            ).arg(
                Arg::with_name("compressed_bandwidth")
                    .long("compressed-bandwidth")
                    .takes_value(true)
                    .default_value("100000000")
                    .validator(validate::<f32>)
                    .value_name("hertz")
                    .help(
                        "The bandwidth of the signal before compression (also known as Fs in the\
                         MATLAB code). The default value is 100 MHz.",
                    ),
            ).arg(
                Arg::with_name("unbuffered")
                    .long("unbuffered")
                    .help("Disables buffering on the source and destination"),
            ).arg(
                Arg::with_name("log_level")
                    .long("log-level")
                    .takes_value(true)
                    .default_value("WARN")
                    .possible_values(&["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"])
                    .help("The level of logging to enable"),
            )
            .arg(Arg::with_name("decompress_band")
                .long("decompress-band")
                .takes_value(true)
                .multiple(true)
                .value_name("bins:frequency[[:path]:time_log_path]")
                .help("The number of bins, center frequency, and output file path of a band to \
                    be decompressed. If the output file path is not specified, decompressed \
                    samples from this band will be written to standard output. This argument may \
                    be repeated to decompress multiple bands.")
                .conflicts_with_all(&["destination", "bins", "center_frequency"])
                .validator(validate::<BandArgs>)
            )
            .arg(Arg::with_name("no_progress")
                .long("no-progress-bar")
                .help("Disables the command-line progress bar")
            )
            .arg(Arg::with_name("report")
                .long("report")
                .help("Displays a report of implementation-defined information about the \
                reconstruction process")
            )
            .arg(Arg::with_name("channel_capacity")
                .long("channel-capacity")
                .takes_value(true)
                .default_value("32")
                .validator(validate::<usize>)
                .value_name("windows")
                .help("Capacity of input -> FFT/output stage channels (this option is unstable)")
            )
            .arg(Arg::with_name("input_log_path")
                .long("input-log")
                .takes_value(true)
                .value_name("path")
                .help("A path to a file to log the times when windows are read")
            )
            .get_matches();

        let buffer = !matches.is_present("unbuffered");

        let bands = if let Some(band_strings) = matches.values_of("decompress_band") {
            // New multi-band version
            band_strings
                .map(|s| BandArgs::from_str(s).unwrap())
                .collect()
        } else {
            // Legacy single-band version
            let band = BandArgs {
                bins: matches.value_of("bins").unwrap().parse().unwrap(),
                center_frequency: matches
                    .value_of("center_frequency")
                    .unwrap()
                    .parse()
                    .unwrap(),
                path: matches.value_of("destination").map(PathBuf::from),
                time_log_path: None,
            };
            vec![band]
        };

        Args {
            source_path: matches.value_of_os("source").map(PathBuf::from),
            buffer,
            compressed_bandwidth: matches
                .value_of("compressed_bandwidth")
                .unwrap()
                .parse()
                .unwrap(),
            bands,
            log_level: matches.value_of("log_level").unwrap().parse().unwrap(),
            report: matches.is_present("report"),
            progress_bar: !matches.is_present("no_progress"),
            channel_capacity: matches
                .value_of("channel_capacity")
                .unwrap()
                .parse()
                .unwrap(),
            input_time_log_path: matches.value_of("input_log_path").map(PathBuf::from),
            _0: (),
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
