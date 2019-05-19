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

extern crate num_complex;

use std::f32::consts::PI;

use num_complex::Complex32;

fn main() {
    println!("ImagIn,RealOut,ImagOut");

    let step = 0.01f32;
    let mut imaginary = 0.0;
    while imaginary <= PI * 2.0 {
        let complex = Complex32::new(0.0, imaginary);

        let exponent = Complex32::exp(&complex);

        // CSV-like output
        println!("{},{},{}", imaginary, exponent.re, exponent.im);

        imaginary += step;
    }
}
