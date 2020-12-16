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

use super::*;

fn check_deserialize(toml: &str, expected: &Config) -> Result<(), toml::de::Error> {
    let parsed: Config = toml::from_str(toml)?;
    assert_eq!(&parsed, expected);
    Ok(())
}

#[test]
fn deserialize_fail_empty() {
    let status = toml::from_str::<Config>("");
    status.expect_err("an empty configuration is not valid");
}

#[test]
fn deserialize_minimum_stdin() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'stdin'
format = 'n210'

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'stdout'
    ",
        &Config {
            source: Input::Stdin {
                format: Format::N210,
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::Stdout,
            }],
        },
    )
}

#[test]
fn deserialize_log_levels() -> Result<(), toml::de::Error> {
    fn check_log_level(name: &str, expected: LevelFilter) -> Result<(), toml::de::Error> {
        check_deserialize(
            &format!(
                r"
[source]
type = 'stdin'
format = 'n210'

[ui]
log_level = '{}'

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'stdout'
    ",
                name
            ),
            &Config {
                source: Input::Stdin {
                    format: Format::N210,
                },
                ui: UserInterface {
                    log_level: expected,
                    ..UserInterface::default()
                },
                bands: vec![Band {
                    bins: 2048,
                    frequency: 0.0,
                    destination: Output::Stdout,
                }],
            },
        )
    }

    check_log_level("off", LevelFilter::Off)?;
    check_log_level("error", LevelFilter::Error)?;
    check_log_level("warn", LevelFilter::Warn)?;
    check_log_level("info", LevelFilter::Info)?;
    check_log_level("debug", LevelFilter::Debug)?;
    check_log_level("trace", LevelFilter::Trace)?;
    Ok(())
}

#[test]
fn deserialize_minimum_file() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'file'
path = '/some/absolute/path.iqz'
format = 'pluto'

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'stdout'
    ",
        &Config {
            source: Input::File {
                path: PathBuf::from("/some/absolute/path.iqz"),
                format: Format::Pluto,
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::Stdout,
            }],
        },
    )
}

#[test]
fn deserialize_input_n210_basic() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'n210'

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'stdout'
",
        &Config {
            source: Input::N210 {
                frequency: None,
                gain: None,
                antenna: None,
                compression: Default::default(),
                args: Default::default(),
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::Stdout,
            }],
        },
    )
}

#[test]
fn deserialize_input_n210_args_basic() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'n210'
args = { addr = '192.168.10.2', type = 'usrp2' }

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'stdout'
",
        &Config {
            source: Input::N210 {
                frequency: None,
                gain: None,
                antenna: None,
                compression: Default::default(),
                args: {
                    let mut args = BTreeMap::new();
                    args.insert("addr".to_owned(), "192.168.10.2".to_owned());
                    args.insert("type".to_owned(), "usrp2".to_owned());
                    args
                },
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::Stdout,
            }],
        },
    )
}

#[test]
fn deserialize_input_n210_args_heterogenous() -> Result<(), toml::de::Error> {
    // source.args has values of non-string types
    check_deserialize(
        r"
[source]
type = 'n210'
args = { addr = '192.168.10.2', type = 'usrp2', recv_frame_size = 3000 }

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'stdout'
",
        &Config {
            source: Input::N210 {
                frequency: None,
                gain: None,
                antenna: None,
                compression: Default::default(),
                args: {
                    let mut args = BTreeMap::new();
                    args.insert("addr".to_owned(), "192.168.10.2".to_owned());
                    args.insert("type".to_owned(), "usrp2".to_owned());
                    args.insert("recv_frame_size".to_owned(), "3000".to_owned());
                    args
                },
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::Stdout,
            }],
        },
    )
}

#[test]
fn deserialize_input_n210_full() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'n210'
frequency = 150e6
gain = 31
antenna = 'TX/RX'
args = { addr = '192.168.10.2', type = 'usrp2', recv_frame_size = 3000 }
[[source.compression.thresholds]]
threshold = 3000
bins = { start = 0, end = 1760 }
[[source.compression.thresholds]]
threshold = 20000
bins = { start = 1760, end = 2048 }
[[source.compression.masks]]
start = 0
end = 120
[[source.compression.masks]]
start = 2000
end = 2048

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'stdout'
",
        &Config {
            source: Input::N210 {
                frequency: Some(150e6),
                gain: Some(31.0),
                antenna: Some("TX/RX".to_owned()),
                compression: Compression {
                    thresholds: vec![
                        ThresholdRange {
                            threshold: 3000,
                            bins: 0..1760,
                        },
                        ThresholdRange {
                            threshold: 20000,
                            bins: 1760..2048,
                        },
                    ],
                    masks: vec![0..120, 2000..2048],
                },
                args: {
                    let mut args = BTreeMap::new();
                    args.insert("addr".to_owned(), "192.168.10.2".to_owned());
                    args.insert("type".to_owned(), "usrp2".to_owned());
                    args.insert("recv_frame_size".to_owned(), "3000".to_owned());
                    args
                },
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::Stdout,
            }],
        },
    )
}

#[test]
fn deserialize_output_file() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'file'
path = '/some/absolute/path.iqz'
format = 'pluto'

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'file'
path = '/some/other/path/output_file'
    ",
        &Config {
            source: Input::File {
                path: PathBuf::from("/some/absolute/path.iqz"),
                format: Format::Pluto,
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::File {
                    path: PathBuf::from("/some/other/path/output_file"),
                },
            }],
        },
    )
}

#[test]
fn deserialize_output_tcp() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'file'
path = '/some/absolute/path.iqz'
format = 'pluto'

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'tcp_client'
server_address = '128.64.8.33:992'
    ",
        &Config {
            source: Input::File {
                path: PathBuf::from("/some/absolute/path.iqz"),
                format: Format::Pluto,
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::TcpClient {
                    server_address: "128.64.8.33:992".parse().unwrap(),
                },
            }],
        },
    )
}

#[test]
fn deserialize_output_udp() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'file'
path = '/some/absolute/path.iqz'
format = 'pluto'

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'udp'
local_address = '192.168.10.147:0'
remote_address = '128.64.8.33:992'
    ",
        &Config {
            source: Input::File {
                path: PathBuf::from("/some/absolute/path.iqz"),
                format: Format::Pluto,
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::Udp {
                    local_address: "192.168.10.147:0".parse().unwrap(),
                    remote_address: "128.64.8.33:992".parse().unwrap(),
                },
            }],
        },
    )
}

#[test]
fn deserialize_output_udp_no_local_address() -> Result<(), toml::de::Error> {
    check_deserialize(
        r"
[source]
type = 'file'
path = '/some/absolute/path.iqz'
format = 'pluto'

[[bands]]
bins = 2048
frequency = 0
[bands.destination]
type = 'udp'
remote_address = '128.64.8.33:992'
    ",
        &Config {
            source: Input::File {
                path: PathBuf::from("/some/absolute/path.iqz"),
                format: Format::Pluto,
            },
            ui: UserInterface::default(),
            bands: vec![Band {
                bins: 2048,
                frequency: 0.0,
                destination: Output::Udp {
                    local_address: any_address(),
                    remote_address: "128.64.8.33:992".parse().unwrap(),
                },
            }],
        },
    )
}
