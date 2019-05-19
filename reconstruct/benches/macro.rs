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

extern crate criterion;
extern crate num_complex;
extern crate sparsdr_reconstruct;
extern crate tempfile;

use std::fs::File;
use std::io::{BufReader, BufWriter};

use criterion::{criterion_group, criterion_main, Criterion};
use sparsdr_reconstruct::input::iqzip::CompressedSamples;
use sparsdr_reconstruct::{BandSetupBuilder, DecompressSetup};

fn benchmark_macro(c: &mut Criterion) {
    c.bench_function("macro_ble_advertising_channels", |f| {
        f.iter(|| {
            let source = File::open("test-data/iqzip/ble-advertising-short.iqz")
                .expect("Failed to open source file");
            let source = CompressedSamples::new(BufReader::new(source));

            let mut ch37_out =
                BufWriter::new(tempfile::tempfile().expect("Failed to create temporary file"));
            let mut ch38_out =
                BufWriter::new(tempfile::tempfile().expect("Failed to create temporary file"));
            let mut ch39_out =
                BufWriter::new(tempfile::tempfile().expect("Failed to create temporary file"));

            let mut setup = DecompressSetup::new(source);
            // Add channels
            setup
                .add_band(
                    BandSetupBuilder::new(&mut ch37_out)
                        .bins(64)
                        .center_frequency(-48000000.0)
                        .build(),
                )
                .add_band(
                    BandSetupBuilder::new(&mut ch38_out)
                        .bins(64)
                        .center_frequency(-24000000.0)
                        .build(),
                )
                .add_band(
                    BandSetupBuilder::new(&mut ch39_out)
                        .bins(64)
                        .center_frequency(30000000.0)
                        .build(),
                );

            sparsdr_reconstruct::decompress(setup).expect("Decompress failed");
        })
    });

    c.bench_function("macro_many_narrow_channels", |f| {
        f.iter(|| {
            let mut frequencies_and_files: Vec<(u32, BufWriter<File>)> = (118_000_000
                ..=136_975_000)
                .step_by(25_000)
                .map(|frequency| {
                    let file = BufWriter::new(
                        tempfile::tempfile().expect("Failed to create temporary file"),
                    );
                    (frequency, file)
                })
                .collect();

            let source = File::open("test-data/iqzip/ble-advertising-extra-short.iqz")
                .expect("Failed to open source file");
            let source = CompressedSamples::new(BufReader::new(source));

            let mut setup = DecompressSetup::new(source);
            // Add channels
            for (frequency, file) in frequencies_and_files.iter_mut() {
                setup.add_band(
                    BandSetupBuilder::new(file)
                        .bins(2)
                        .center_frequency(*frequency as f32 - 150_000_000.0)
                        .build(),
                );
            }

            sparsdr_reconstruct::decompress(setup).expect("Decompress failed");
        })
    });
}

criterion_group!(benches, benchmark_macro);
criterion_main!(benches);
