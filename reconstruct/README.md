# sparsdr_reconstruct

This application reconstructs signals from SparSDR compressed data. It can decompress the full frequency range,
or decompress various narrower bands.

## Setting up

### Installing the Rust compiler and Cargo

This application the latest stable version of the Rust compiler. Installation information is available at
<https://www.rust-lang.org/learn/get-started> .

### Compiling

Run `cargo build --release` to compile the software with optimizations.

## Running

Note: If you are using the GNU Radio blocks to reconstruct signals, the blocks automatically run sparsdr_reconstruct
and this section does not apply.

The compiled application will be placed at `target/release/sparsdr_reconstruct`. Use
`target/release/sparsdr_reconstruct --help` to see the most up-to-date information about the command-line arguments.

The most commonly used arguments are `--source <path>` to specify the path to a compressed file to read, and
`--destination <path>` to specify the path to a decompressed file to write. Depending on the radio used to receive the
signals and the compressed sample format version, one of these options is also needed: `--n210-v1-defaults`,
`--n210-v2-defaults`, `--pluto-v1-defaults`, or `--pluto-v2-defaults`.

By default, the software will decompress the full frequency range. The output file will have a sample rate of 100
million samples per second (if the signals were received with a USRP N210), or 61.44 million samples per second
(if the signals were received with a Pluto).

## Installing

Run `cargo install --path .` to install `sparsdr_reconstruct` in the `~/.cargo/bin/` folder.

## License

Apache 2.0:

    Copyright 2019 The Regents of the University of California

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
