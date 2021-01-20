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
use std::io::BufReader;

use criterion::{criterion_group, criterion_main, Criterion};

use num_complex::Complex32;
use sparsdr_reconstruct::input::format::n210::N210SampleReader;
use sparsdr_reconstruct::output::WriteOutput;
use sparsdr_reconstruct::{BandSetupBuilder, DecompressSetup};
use std::error::Error;

const COMPRESSION_BINS: u16 = 2048;
const COMPRESSION_BANDWIDTH: f32 = 100_000_000.0;

struct NullOutput;

impl WriteOutput for NullOutput {
    fn write_samples(&mut self, _samples: &[Complex32]) -> Result<(), Box<dyn Error + Send>> {
        // Ignore
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }
}

fn benchmark_macro(c: &mut Criterion) {
    c.bench_function("macro_ble_advertising_channels", |f| {
        f.iter(|| {
            let source = File::open("test-data/iqzip/ble-advertising.iqz")
                .expect("Failed to open source file");
            let source = N210SampleReader::new(BufReader::new(source));

            let ch37_out = NullOutput;
            let ch38_out = NullOutput;
            let ch39_out = NullOutput;

            let mut setup = DecompressSetup::new(Box::new(source), COMPRESSION_BINS);
            // Add channels
            setup
                .add_band(
                    BandSetupBuilder::new(
                        Box::new(ch37_out),
                        COMPRESSION_BINS,
                        COMPRESSION_BANDWIDTH,
                    )
                    .bins(64)
                    .center_frequency(-48000000.0)
                    .build(),
                )
                .add_band(
                    BandSetupBuilder::new(
                        Box::new(ch38_out),
                        COMPRESSION_BINS,
                        COMPRESSION_BANDWIDTH,
                    )
                    .bins(64)
                    .center_frequency(-24000000.0)
                    .build(),
                )
                .add_band(
                    BandSetupBuilder::new(
                        Box::new(ch39_out),
                        COMPRESSION_BINS,
                        COMPRESSION_BANDWIDTH,
                    )
                    .bins(64)
                    .center_frequency(30000000.0)
                    .build(),
                );

            sparsdr_reconstruct::decompress(setup).expect("Decompress failed");
        })
    });

    c.bench_function("macro_many_narrow_channels", |f| {
        f.iter(|| {
            let mut frequencies_and_files: Vec<(u32, NullOutput)> = (118_000_000..=136_975_000)
                .step_by(25_000)
                .map(|frequency| (frequency, NullOutput))
                .collect();

            let source = File::open("test-data/iqzip/ble-advertising.iqz")
                .expect("Failed to open source file");
            let source = Box::new(N210SampleReader::new(BufReader::new(source)));

            let mut setup = DecompressSetup::new(source, COMPRESSION_BINS);
            // Add channels
            for (frequency, file) in frequencies_and_files {
                setup.add_band(
                    BandSetupBuilder::new(Box::new(file), COMPRESSION_BINS, COMPRESSION_BANDWIDTH)
                        .bins(2)
                        .center_frequency(frequency as f32 - 150_000_000.0)
                        .build(),
                );
            }

            sparsdr_reconstruct::decompress(setup).expect("Decompress failed");
        })
    });
}

criterion_group!(benches, benchmark_macro);
criterion_main!(benches);
