/*
 * Copyright 2021 The Regents of the University of California
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
//! print_samples binary: Reads compressed samples from standard input and writes a human-readable
//! table representing the samples to standard output
//!

extern crate simplelog;
extern crate sparsdr_sample_parser;

use sparsdr_sample_parser::{Parser, V2Parser, WindowKind};
use std::io;
use std::io::{BufWriter, ErrorKind, Read, Write};

fn main() -> Result<(), io::Error> {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Warn,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
    )
    .unwrap();

    let mut parser = V2Parser::new(1024);

    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());

    writeln!(
        output,
        "#Sample    |      Type |    FFT_No |     Index |  Time     |      Real |      Imag"
    )?;

    let mut sample_index = 0u64;
    loop {
        let mut sample_bytes = [0u8; 4];
        if let Err(e) = input.read_exact(&mut sample_bytes) {
            match e.kind() {
                ErrorKind::UnexpectedEof => break,
                _ => return Err(e),
            }
        }
        match parser.parse(&sample_bytes) {
            Ok(None) => {}
            Ok(Some(window)) => match window.kind {
                WindowKind::Data(data) => {
                    let fft_no = window.timestamp & 0x1;
                    for (i, value) in data.iter().enumerate() {
                        let status = writeln!(
                            output,
                            "{:<10}    FFT sample {:>10}  {:>10}  {:>10}  {:>10}  {:>10}",
                            sample_index, fft_no, i, window.timestamp, value.re, value.im
                        );

                        if let Err(e) = status {
                            if e.kind() == ErrorKind::BrokenPipe {
                                break;
                            } else {
                                return Err(e);
                            }
                        }

                        sample_index += 1;
                    }
                }
                WindowKind::Average(average) => {
                    for (i, value) in average.iter().enumerate() {
                        let status = writeln!(
                            output,
                            "{:<10}    Average                {:>10}  {:>10}      {:>10}",
                            sample_index, i, window.timestamp, *value
                        );

                        if let Err(e) = status {
                            if e.kind() == ErrorKind::BrokenPipe {
                                break;
                            } else {
                                return Err(e);
                            }
                        }

                        sample_index += 1;
                    }
                }
            },
            Err(e) => {
                if let Err(e) = writeln!(output, "Parse error {:?}", e) {
                    if e.kind() == ErrorKind::BrokenPipe {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(())
}
