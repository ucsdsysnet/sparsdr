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

//!
//! Loads some compressed samples and a correlation template
//!
//! A 3 MHz offset in the compressed samples should detect 12 packets. With a 6 MHz offset,
//! no packets should be detected.
//!

extern crate matfile;
extern crate num_complex;
extern crate sparsdr_reconstruct;
extern crate sparsdr_sample_parser;

use matfile::{MatFile, NumericData};
use num_complex::Complex32;
use sparsdr_reconstruct::bins::BinRange;
use sparsdr_reconstruct::iter::PushIterator;
use sparsdr_reconstruct::iter_ext::IterExt;
use sparsdr_reconstruct::steps::overflow::OverflowPushIter;
use sparsdr_reconstruct::steps::shift::ShiftPushIter;
use sparsdr_reconstruct::window::{Logical, Status, Window};
use sparsdr_reconstruct::window_filter::correlation_template::CorrelationTemplateFilter;
use sparsdr_reconstruct::window_filter::WindowFilter;
use sparsdr_sample_parser::{Parser, V1Parser, WindowKind};
use std::convert::Infallible;
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read};
use std::ops::ControlFlow;

#[test]
#[ignore]
fn correlation_template() -> Result<(), Box<dyn std::error::Error>> {
    let template = load_template()?;
    let mut filter = CorrelationTemplateFilter::new(template.len(), vec![template], 0.4)?;

    // Load compressed samples (Pluto, v1)
    let mut sample_file = BufReader::new(File::open(
        "./test-data/correlation_template/pluto_sparsdrv1_x300_tx_ramcap.iqz",
    )?);
    let mut parser = V1Parser::new_pluto(1024);

    let mut logical_samples: Vec<Status<Window<Logical>>> = Vec::new();
    let mut chain = OverflowPushIter::new(
        ShiftPushIter::new(
            Collector::new(&mut logical_samples).map(|window| Status::Ok(window)),
            1024,
        ),
        21,
    );
    loop {
        let mut buffer = [0u8; 8];
        match sample_file.read_exact(&mut buffer) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        if let Some(window) = parser.parse(&buffer).expect("Sample parse error") {
            if let Some(window) = convert_window(window) {
                chain.push(window);
            }
        }
    }
    drop(sample_file);

    // 34 bins, 3 MHz offset out of 1024 bins over 61.44 MHz
    // => centered at 512 + 25 = 537
    let bin_range_3mhz = BinRange::from(520..554);
    let matching_windows_3mhz =
        count_windows_over_threshold(&logical_samples, &mut filter, bin_range_3mhz);

    // 34 bins, 6 MHz offset out of 1024 bins over 61.44 MHz
    // => centered at 512 + 50 = 552
    let bin_range_6mhz = BinRange::from(535..569);
    let matching_windows_6mhz =
        count_windows_over_threshold(&logical_samples, &mut filter, bin_range_6mhz);
    println!(
        "Match {} 3 MHz windows, {} 6 MHz windows",
        matching_windows_3mhz, matching_windows_6mhz
    );
    assert!(matching_windows_3mhz > matching_windows_6mhz);

    Ok(())
}

/// A push iterator that collects items into a vector
struct Collector<'a, T> {
    items: &'a mut Vec<T>,
}

impl<'a, T> Collector<'a, T> {
    pub fn new(items: &'a mut Vec<T>) -> Self {
        Collector { items }
    }
}

impl<'a, T> PushIterator<T> for Collector<'a, T> {
    type Error = Infallible;

    fn push(&mut self, item: T) -> ControlFlow<Self::Error> {
        self.items.push(item);
        ControlFlow::Continue(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// If the provided window is a data window, this function returns its converted form.
/// Otherwise, this function returns None.
fn convert_window(window: sparsdr_sample_parser::Window) -> Option<Window> {
    match window.kind {
        WindowKind::Data(bins) => {
            // Convert integer to float and scale to [-1, 1]
            let scaled_bins = bins
                .into_iter()
                .map(|int_complex| {
                    Complex32::new(map_value(int_complex.re), map_value(int_complex.im))
                })
                .collect();
            Some(Window::with_bins(window.timestamp.into(), scaled_bins))
        }
        WindowKind::Average(_) => None,
    }
}

fn map_value(value: i16) -> f32 {
    f32::from(value) / 32767.0
}

fn count_windows_over_threshold(
    logical_samples: &[Status<Window<Logical>>],
    filter: &mut CorrelationTemplateFilter,
    bins: BinRange,
) -> usize {
    logical_samples
        .iter()
        .cloned()
        .filter_bins(bins, 34)
        .shift(34)
        .filter_map(|window| {
            if let Status::Ok(mut window) = window {
                // Remove a bin to match the size of the template
                window.truncate_bins(33);

                if filter.accept(&window) {
                    println!("Accept");
                    Some(())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .count()
}

fn load_template() -> Result<Vec<Complex32>, Box<dyn std::error::Error>> {
    let file = File::open("./test-data/correlation_template/bin_values_and_template.mat")?;
    let file = MatFile::parse(file)?;
    let template = file
        .find_by_name("corr_template_norm_3M_offset")
        .expect("No template in file");

    match template.data() {
        NumericData::Double {
            real,
            imag: Some(imag),
        } => {
            assert_eq!(real.len(), imag.len());
            Ok(real
                .iter()
                .zip(imag.iter())
                .map(|(real, imag)| Complex32::new(*real as f32, *imag as f32))
                .collect())
        }
        _ => panic!("Invalid template data format"),
    }
}
