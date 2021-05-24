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

//!
//! Analog Devices Pluto SDR compressed data format
//!
//! Properties of this format:
//! * Receive sample rate and compressed bandwidth: 61.44 MHz
//! * 1024 bins
//!

/// Number of bytes used to represent a sample, in the format the USRP sends
pub const SAMPLE_BYTES: usize = 8;
/// FFT size used for compression
pub const BINS: u16 = 1024;
/// Compression sample rate
pub const SAMPLE_RATE: f32 = 61_440_000.0;
