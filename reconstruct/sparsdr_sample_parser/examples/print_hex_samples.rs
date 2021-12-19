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

use std::io;
use std::io::{BufWriter, ErrorKind, Read, Write};

fn main() -> Result<(), io::Error> {
    let stdin = io::stdin();
    let mut input = stdin.lock();
    let stdout = io::stdout();
    let mut output = BufWriter::new(stdout.lock());

    for i in 0u64.. {
        let mut sample_bytes = [0u8; 4];
        if let Err(e) = input.read_exact(&mut sample_bytes) {
            match e.kind() {
                ErrorKind::UnexpectedEof => break,
                _ => return Err(e),
            }
        }
        let sample = u32::from_le_bytes(sample_bytes);
        writeln!(output, "{}: {:#010x}", i, sample)?;
    }

    Ok(())
}
