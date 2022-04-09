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
//! Reads gr-bluetooth-sample-logging output from standard input and counts the number of decoded
//! packets
//!

extern crate regex;

use std::io::{self, BufReader, Read, Result};

use regex::bytes::Regex;

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdin = BufReader::new(stdin.lock());
    let mut input = Vec::new();
    stdin.read_to_end(&mut input)?;

    // Example line: "Start of packet 543321 samples in, length 37 octets"
    let pattern = Regex::new(r"(?m)^Start of packet \d+ samples in, length \d+ octets$").unwrap();
    let packet_count = pattern.find_iter(&input).count();
    println!("Decoded {} packets", packet_count);

    Ok(())
}
