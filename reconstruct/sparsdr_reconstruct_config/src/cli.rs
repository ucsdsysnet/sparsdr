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

//!
//! Configuration from command-line arguments
//!

use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::str::FromStr;

use clap::{crate_authors, crate_name, crate_version, App, Arg, ArgMatches};

use crate::{Band, Config, Format, Input, Output, UserInterface};

const ABOUT: &str = "This program reads SparSDR compressed samples from a file or radio, \
reconstructs signals in one or more bands, and writes the reconstructed signals to files or other \
destinations. Command-line arguments can be used to set up basic reconstruction that reads from \
a file, reconstructs one band, and writes the output to a file. For other options, a configuration \
file must be used.";

/// Reads command-line options and either reads a configuration from a file or builds a configuration
/// from the command-line options
///
/// This function returns an error if the configuration file could not be read or could not be parsed.
/// It causes the process to exit if a command-line argument is invalid, or if `--help` or `--version`
/// is passed.
pub fn config_from_command_line() -> Result<Config, Box<dyn Error>> {
    let matches = build_app().get_matches();

    if let Some(config_path) = matches.value_of_os("config_file") {
        read_config_file(config_path)
    } else {
        Ok(config_from_matches(&matches))
    }
}

/// Creates and returns an App with command-line arguments
fn build_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .about(ABOUT)
        .author(crate_authors!())
        .arg(
            Arg::with_name("config_file")
                .long("config-file")
                .short("c")
                .takes_value(true)
                .value_name("path")
                .help(
                    "The path to a configuration file to read. \
                    This can be used to specify additional options. \
                    If this option is used, no other command-line arguments are permitted.",
                )
                .conflicts_with_all(&[
                    "source",
                    "destination",
                    "bins",
                    "center_frequency",
                    "log_level",
                    "sample_format",
                ]),
        )
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
                .default_value("2048")
                .validator(validate::<u16>)
                .help("The number of bins to decompress"),
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
            Arg::with_name("log_level")
                .long("log-level")
                .takes_value(true)
                .default_value("WARN")
                .possible_values(&["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"])
                .help("The level of logging to enable"),
        )
        .arg(
            Arg::with_name("sample_format")
                .long("format")
                .takes_value(true)
                .default_value("n210")
                .possible_values(&["n210", "pluto"])
                .help(
                    "The compressed sample format to read (this depends on the radio used to \
                capture the signals)",
                ),
        )
}

/// Reads a configuration file at the provided path, parses it, and returns it
fn read_config_file(path: &OsStr) -> Result<Config, Box<dyn Error>> {
    let file_bytes = fs::read(path)?;
    let config = toml::from_slice(&file_bytes)?;
    Ok(config)
}

/// Creates a configuration from the command-line arguments
fn config_from_matches(matches: &ArgMatches) -> Config {
    Config {
        source: input_from_matches(matches),
        ui: ui_from_matches(matches),
        bands: bands_from_matches(matches),
    }
}

fn input_from_matches(matches: &ArgMatches) -> Input {
    let format = match matches.value_of("sample_format").unwrap() {
        "n210" => Format::N210,
        "pluto" => Format::Pluto,
        other => unreachable!("Invalid sample format name \"{}\"", other),
    };
    match matches.value_of("source") {
        Some(path) => Input::File {
            path: path.into(),
            format,
        },
        None => Input::Stdin { format },
    }
}

fn ui_from_matches(matches: &ArgMatches) -> UserInterface {
    UserInterface {
        // This can't panic because the argument is required and has restricted values.
        log_level: matches.value_of("log_level").unwrap().parse().unwrap(),
    }
}

fn bands_from_matches(matches: &ArgMatches) -> Vec<Band> {
    let band = Band {
        bins: matches.value_of("bins").unwrap().parse().unwrap(),
        frequency: matches
            .value_of("center_frequency")
            .unwrap()
            .parse()
            .unwrap(),
        destination: match matches.value_of_os("destination") {
            Some(path) => Output::File { path: path.into() },
            None => Output::Stdout,
        },
    };
    vec![band]
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

#[cfg(test)]
mod test {
    use super::*;
    use log::LevelFilter;

    #[test]
    fn no_args() -> Result<(), Box<dyn Error>> {
        let app = build_app();
        let matches = app.get_matches_from_safe(vec!["sparsdr_reconstruct"])?;
        let config = config_from_matches(&matches);

        assert_eq!(
            config,
            Config {
                source: Input::Stdin {
                    format: Format::N210
                },
                ui: Default::default(),
                bands: vec![Band {
                    bins: 2048,
                    frequency: 0.0,
                    destination: Output::Stdout
                }]
            }
        );

        Ok(())
    }

    #[test]
    fn some_args() -> Result<(), Box<dyn Error>> {
        let app = build_app();
        let matches = app.get_matches_from_safe(vec![
            "sparsdr_reconstruct",
            "--format",
            "pluto",
            "--bins",
            "32",
        ])?;
        let config = config_from_matches(&matches);

        assert_eq!(
            config,
            Config {
                source: Input::Stdin {
                    format: Format::Pluto
                },
                ui: Default::default(),
                bands: vec![Band {
                    bins: 32,
                    frequency: 0.0,
                    destination: Output::Stdout
                }]
            }
        );

        Ok(())
    }

    #[test]
    fn all_args() -> Result<(), Box<dyn Error>> {
        let app = build_app();
        let matches = app.get_matches_from_safe(vec![
            "sparsdr_reconstruct",
            "--source",
            "./folder/some_file.iqz",
            "--format",
            "pluto",
            "--bins",
            "32",
            "--center-frequency",
            "20e6",
            "--log-level",
            "DEBUG",
            "--destination",
            "/absolute/iq_file",
        ])?;
        let config = config_from_matches(&matches);

        assert_eq!(
            config,
            Config {
                source: Input::File {
                    path: "./folder/some_file.iqz".into(),
                    format: Format::Pluto
                },
                ui: UserInterface {
                    log_level: LevelFilter::Debug
                },
                bands: vec![Band {
                    bins: 32,
                    frequency: 20e6,
                    destination: Output::File {
                        path: "/absolute/iq_file".into()
                    }
                }]
            }
        );

        Ok(())
    }
}
