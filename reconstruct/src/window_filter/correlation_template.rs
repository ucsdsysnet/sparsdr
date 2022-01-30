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

//! Window filtering using correlation tempaltes

use fftw::array::AlignedVec;
use fftw::plan::{C2CPlan, C2CPlan32};
use fftw::types::{Flag, Sign};
use num_complex::Complex32;

use crate::window::{Fft, Window};
use crate::window_filter::WindowFilter;

/// A filter that compares frequency-domain windows to one or more templates
pub struct CorrelationTemplateFilter {
    fft_size: usize,
    /// FFT setup
    fft: C2CPlan32,
    /// Templates to compare against
    ///
    /// Each template must have a length equal to fft_size.
    templates: Vec<Vec<Complex32>>,
    /// The threshold for the maximum correlation value
    threshold: f32,
}

impl CorrelationTemplateFilter {
    /// Creates a filter
    ///
    /// # Panics
    ///
    /// This function panics if any template has a length that is not equal to fft_size.
    pub fn new(
        fft_size: usize,
        templates: Vec<Vec<Complex32>>,
        threshold: f32,
    ) -> fftw::error::Result<Self> {
        let fft = C2CPlan32::aligned(
            &[fft_size],
            Sign::Backward,
            Flag::MEASURE | Flag::DESTROYINPUT,
        )?;

        for template in &templates {
            assert_eq!(template.len(), fft_size, "Incorrect template length");
        }

        Ok(CorrelationTemplateFilter {
            fft_size,
            fft,
            templates,
            threshold,
        })
    }
}

impl WindowFilter for CorrelationTemplateFilter {
    fn accept(&mut self, window: &Window<Fft>) -> bool {
        let bins = window.bins();
        assert_eq!(bins.len(), self.fft_size);
        for template in &self.templates {
            // Calculate elementwise product of window and template
            let mut product = AlignedVec::new(self.fft_size);

            for (product_entry, product) in product.iter_mut().zip(
                bins.iter()
                    .zip(template.iter())
                    .map(|(bin, template_bin)| *bin * *template_bin),
            ) {
                *product_entry = product;
            }
            // Inverse FFT
            let mut fft_output = AlignedVec::new(self.fft_size);
            self.fft
                .c2c(&mut product, &mut fft_output)
                .expect("Inverse FFT failed");

            // If the absolute value of any bin exceeds the threshold, this window matches
            println!(
                "Maximum correlation {}",
                fft_output
                    .iter()
                    .map(|bin| bin.norm())
                    .reduce(f32::max)
                    .unwrap()
            );
            for bin in fft_output.iter() {
                let magnitude = bin.norm();
                if magnitude > self.threshold {
                    // Immediate match, no need to check anything else
                    return true;
                }
            }
        }
        // No template matched
        false
    }
}
