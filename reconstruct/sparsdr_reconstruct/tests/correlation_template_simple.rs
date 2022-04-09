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
extern crate matfile;
extern crate num_complex;
extern crate sparsdr_reconstruct;

use matfile::{MatFile, NumericData};
use num_complex::Complex32;
use sparsdr_reconstruct::window::{Fft, Window};
use sparsdr_reconstruct::window_filter::correlation_template::CorrelationTemplateFilter;
use sparsdr_reconstruct::window_filter::WindowFilter;
use std::convert::TryInto;
use std::fs::File;
use std::io::BufReader;

const FFT_SIZE: usize = 33;

#[test]
fn correlation_template_simple() -> Result<(), Box<dyn std::error::Error>> {
    let template = load_template()?;
    let windows = load_compressed_samples()?;

    let mut correlator = CorrelationTemplateFilter::new(FFT_SIZE, vec![template], 0.1)?;

    for window in windows {
        assert!(correlator.accept(&window));
    }

    Ok(())
}

fn load_template() -> Result<Vec<Complex32>, Box<dyn std::error::Error>> {
    let template_and_things = MatFile::parse(BufReader::new(File::open(
        "./test-data/correlation_template_simple/corrTemplateandCenteredBinIdx.mat",
    )?))?;
    let template_array = template_and_things
        .find_by_name("corr_template_norm")
        .unwrap();
    match template_array.data() {
        NumericData::Double {
            real,
            imag: Some(imag),
        } => {
            assert_eq!(real.len(), imag.len());

            let values: Vec<Complex32> = real
                .iter()
                .zip(imag.iter())
                .map(|(real, imag)| Complex32::new(*real as f32, *imag as f32))
                .collect();

            Ok(values)
        }
        _ => panic!("Unexpected data format"),
    }
}

/// Loads the compressed samples from a file and returns windows with 32 bins each in FFT order
fn load_compressed_samples() -> Result<Vec<Window<Fft>>, Box<dyn std::error::Error>> {
    let compressed_samples = MatFile::parse(BufReader::new(File::open(
        "./test-data/correlation_template_simple/stftWindowsCentered.mat",
    )?))?;
    let sample_array = compressed_samples.find_by_name("stft2Save").unwrap();
    assert_eq!(sample_array.size(), &[1024, 32]);

    match sample_array.data() {
        NumericData::Double {
            real,
            imag: Some(imag),
        } => {
            assert_eq!(real.len(), imag.len());
            let real_chunks = real.chunks_exact(1024);
            let imaginary_chunks = imag.chunks_exact(1024);
            let chunks = real_chunks.zip(imaginary_chunks);
            // Every set of 1024 values is one window, in frequency order
            // Choose bins 545..=577
            let bin_range = 545..=577;
            let bins = chunks
                .enumerate()
                .map(|(i, (window_real, window_imag))| {
                    let selected_real = &window_real[bin_range.clone()];
                    let selected_imag = &window_imag[bin_range.clone()];

                    let bins: Vec<Complex32> = selected_real
                        .iter()
                        .zip(selected_imag.iter())
                        .map(|(real, imag)| Complex32::new(*real as f32, *imag as f32))
                        .collect();
                    Window::with_bins(i.try_into().unwrap(), bin_range.clone().count(), bins)
                })
                .collect();

            Ok(bins)
        }
        _ => panic!("Unexpected data format"),
    }
}
