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

//! Reads compressed samples from standard input, parses them, and prints any errors
//!
//! This application does not print any samples that are successfully parsed.

extern crate sparsdr_sample_parser;

use std::io::{self, BufWriter, ErrorKind, Read, Write};

use sparsdr_sample_parser::Parser;

fn main() -> Result<(), io::Error> {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Warn,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
    )
    .unwrap();

    let mut parser = Parser::new(1024);

    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());

    let mut windows = 0u64;
    let mut errors = 0u64;
    loop {
        let mut sample_bytes = [0u8; 4];
        if let Err(e) = input.read_exact(&mut sample_bytes) {
            match e.kind() {
                ErrorKind::UnexpectedEof => break,
                _ => return Err(e),
            }
        }
        let sample = u32::from_le_bytes(sample_bytes);
        match parser.accept(sample) {
            Ok(None) => {}
            Ok(Some(_window)) => {
                windows = windows.saturating_add(1);
            }
            Err(e) => {
                errors = errors.saturating_add(1);
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

    let error_ratio = errors as f64 / windows as f64;
    writeln!(output, "Got {} windows, {} errors", windows, errors)?;
    writeln!(output, "Error ratio {}", error_ratio)?;

    Ok(())
}
