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

extern crate sparsdr_reconstruct;

use std::env;
use std::fs::File;
use std::io::{self, BufReader, Result, Write};
use std::process;

use sparsdr_reconstruct::input::matlab::Samples;

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            process::exit(-1);
        }
    }
}

fn run() -> Result<()> {
    let path = env::args_os().skip(1).next().unwrap_or_else(|| {
        eprintln!("Usage: read_doubles path");
        process::exit(-1);
    });
    let file = BufReader::new(File::open(path)?);
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let samples = Samples::new(file);
    for (i, sample) in samples.enumerate() {
        let sample = sample?;
        if let Err(_) = writeln!(stdout, "{} {}", i, sample.amplitude) {
            break;
        }
    }

    Ok(())
}
