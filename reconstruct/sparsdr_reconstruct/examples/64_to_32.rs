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

//!
//! Reads 64-bit floating-point values from a file, converts them to 32-bit floating-point values,
//! and writes them to another file
//!

extern crate byteorder;

use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Result};
use std::process;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

fn main() {
    let mut args = env::args_os().skip(1);
    let in_path = args.next().unwrap_or_else(|| print_usage_and_exit());
    let out_path = args.next().unwrap_or_else(|| print_usage_and_exit());

    match run(&in_path, &out_path) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            process::exit(-1);
        }
    }
}

fn run(in_path: &OsStr, out_path: &OsStr) -> Result<()> {
    let mut in_file = BufReader::new(File::open(in_path)?);
    let mut out_file = BufWriter::new(File::create(out_path)?);

    loop {
        let value = match in_file.read_f64::<LE>() {
            Ok(value) => value,
            Err(e) => match e.kind() {
                ErrorKind::UnexpectedEof => break,
                _ => return Err(e),
            },
        };
        let value = value as f32;
        out_file.write_f32::<LE>(value)?;
    }

    Ok(())
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: 64_to_32 input-path output-path");
    process::exit(-1);
}
