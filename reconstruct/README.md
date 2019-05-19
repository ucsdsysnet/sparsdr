# sparsdr_reconstruct

This application reconstructs signals from SparSDR compressed data. It can decompress everything using an inverse FFT
of size 2048, or decompress various narrower bands with smaller inverse FFTs.

## Setting up

### Installing the Rust compiler and Cargo

This application the latest stable version of the Rust compiler. Installation information is available at
[https://www.rust-lang.org/learn/get-started](https://www.rust-lang.org/learn/get-started) .

### Compiling

Run `cargo build --release` to compile the software with optimizations.

## Running

The compiled application will be placed at `target/release/sparsdr_reconstruct`. Use
`target/release/sparsdr_reconstruct --help` to see the most up-to-date information about the command-line arguments.

The most commonly used arguments are `--source <path>` to specify the path to a compressed file to read, and
`--destination <path>` to specify the path to a decompressed file to write.

By default, the software will decompress the full frequency range using an inverse FFT of size 2048. This will produce
a decompressed file with a sample rate of 100 million samples/second.

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
