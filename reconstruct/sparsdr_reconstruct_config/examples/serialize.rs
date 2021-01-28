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

extern crate sparsdr_reconstruct_config;
extern crate toml;

use std::path::PathBuf;

use sparsdr_reconstruct_config::{
    Band, Compression, Config, Format, Input, Output, ThresholdRange, UdpHeaderFormat,
    UserInterface,
};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_config_1 = Config {
        source: Input::Stdin {
            format: Format::N210,
        },
        ui: UserInterface::default(),
        bands: vec![
            Band {
                bins: 64,
                frequency: 10e6,
                destination: Output::Stdout,
            },
            Band {
                bins: 64,
                frequency: 20e6,
                destination: Output::File {
                    path: PathBuf::from("/tmp/file"),
                },
            },
            Band {
                bins: 10,
                frequency: -1e6,
                destination: Output::Udp {
                    local_address: "127.0.0.1:0".parse()?,
                    remote_address: "127.0.0.1:3920".parse()?,
                    header_format: UdpHeaderFormat::None,
                    mtu: 1472,
                },
            },
        ],
    };
    println!("{}", toml::to_string(&test_config_1)?);
    let uhd_args = {
        let mut args = BTreeMap::new();
        args.insert("type".to_owned(), "usrp2".to_owned());
        args.insert("addr".to_owned(), "192.168.10.2".to_owned());
        args.insert("recv_frame_size".to_owned(), "3000".to_owned());
        args
    };
    let test_config_2 = Config {
        source: Input::N210 {
            args: uhd_args,
            frequency: Some(120e6),
            gain: None,
            antenna: Some("RX2".to_owned()),
            compression: Compression {
                thresholds: vec![ThresholdRange {
                    bins: 10..20,
                    threshold: 10_000,
                }],
                masks: vec![],
                ..Compression::default()
            },
        },
        ui: UserInterface::default(),
        bands: vec![
            Band {
                bins: 64,
                frequency: 10e6,
                destination: Output::Stdout,
            },
            Band {
                bins: 64,
                frequency: 20e6,
                destination: Output::File {
                    path: PathBuf::from("/tmp/file"),
                },
            },
            Band {
                bins: 10,
                frequency: -1e6,
                destination: Output::Udp {
                    local_address: "127.0.0.1:0".parse()?,
                    remote_address: "127.0.0.1:3920".parse()?,
                    header_format: UdpHeaderFormat::Sequence,
                    mtu: 1472,
                },
            },
            Band {
                bins: 8,
                frequency: -37e6,
                destination: Output::TcpClient {
                    server_address: "127.0.0.1:50320".parse()?,
                },
            },
        ],
    };
    println!("{}", toml::to_string(&test_config_2)?);

    Ok(())
}
