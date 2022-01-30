/*
 * Copyright 2022 The Regents of the University of California
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

extern crate simplelog;
extern crate sparsdr_sample_parser;

use sparsdr_sample_parser::{Parser, V2Parser, WindowKind};
use std::io;
use std::io::{ErrorKind, Read};

fn main() -> Result<(), io::Error> {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Warn,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
    )
    .unwrap();

    let mut parser = V2Parser::new(512);

    let stdin = io::stdin();
    let mut input = stdin.lock();

    let mut bin_max = 0u32;

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
                    for bin_value in data {
                        let real_abs = i32::from(bin_value.re).abs() as u32;
                        let imaginary_abs = i32::from(bin_value.im).abs() as u32;
                        bin_max = bin_max.max(real_abs.max(imaginary_abs));
                    }
                }
                WindowKind::Average(_averages) => {}
            },
            Err(_e) => {}
        }
    }

    println!("Maximum absolute value of bin values: {}", bin_max);

    Ok(())
}
