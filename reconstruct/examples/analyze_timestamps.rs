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

extern crate sparsdr_reconstruct;
extern crate sparsdr_sample_parser;

const V2_TIMESTAMP_BITS: u32 = 30;

use sparsdr_reconstruct::input::SampleReader;
use sparsdr_reconstruct::iter_ext::IterExt;
use sparsdr_sample_parser::V2Parser;
use std::io::{self, Result};

fn main() -> Result<()> {
    let stdin = io::stdin();
    let stdin = stdin.lock();

    let parser = V2Parser::new(512);
    let windows = SampleReader::new(stdin, parser).overflow_correct(V2_TIMESTAMP_BITS);

    let mut first_timestamp = None;
    let mut last_timestamp = None;
    let mut consecutive_samples = 1u32;
    for window in windows {
        let window = window?;

        // If this is the first window, or the first window after a gap, print its time
        match last_timestamp {
            Some(last_timestamp) => {
                let gap = window.time() - last_timestamp;
                if gap == 1 {
                    consecutive_samples += 1;
                } else {
                    println!(
                        "{} (gap {} after {} consecutive samples)",
                        window.time(),
                        gap,
                        consecutive_samples
                    );
                    consecutive_samples = 1;
                }
            }
            None => println!("{}", window.time()),
        }

        if first_timestamp.is_none() {
            first_timestamp = Some(window.time());
        }
        last_timestamp = Some(window.time());
    }

    if let (Some(first_timestamp), Some(last_timestamp)) = (first_timestamp, last_timestamp) {
        assert!(first_timestamp <= last_timestamp, "Timestamps out of order");
        let range = last_timestamp - first_timestamp;
        println!("Timestamp range {} windows", range);
    } else {
        println!("No timestamps, can't calculate range");
    }

    Ok(())
}
