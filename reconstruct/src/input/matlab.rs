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

//!
//! Reading of compressed data from binary files produced by MATLAB
//!
//! Each file contains zero or more chunks of 2048 complex amplitude values.
//!
//! Each chunk contains 2048 real values, followed by 2048 imaginary values. Each of these is a
//! 64-bit floating-point number in little-endian byte order
//!
//! Chunks are assumed to have sequential time values.
//!

use std::io::{ErrorKind, Read, Result};
use std::iter;
use std::iter::FlatMap;

use byteorder::{ReadBytesExt, LE};
use num_complex::Complex32;

use super::Sample;
use crate::input::ReadInput;
use std::convert::TryInto;
use std::error::Error;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Number of complex samples in a chunk
const CHUNK_SAMPLES: usize = 2048;
/// Number of real/imaginary values in a chunk
const CHUNK_NUMBERS: usize = CHUNK_SAMPLES * 2;

/// A chunk of 2048 samples
struct Chunk {
    /// Time (chunk index)
    time: u64,
    /// Amplitude values, CHUNK_NUMBERS elements, CHUNK_SAMPLES real followed by CHUNK_SAMPLES complex
    amplitudes: Vec<f32>,
}

/// A FlatMap that converts chunks into samples
type ChunkFnFlatMap<R> = FlatMap<
    Chunks<R>,
    Box<dyn Iterator<Item = Result<Sample>> + Send>,
    fn(Result<Chunk>) -> Box<dyn Iterator<Item = Result<Sample>> + Send>,
>;

/// Reads compressed samples from a byte source
pub struct Samples<R>(ChunkFnFlatMap<R>);

impl<R> Samples<R>
where
    R: Read,
{
    /// Creates an iterator that reads compressed samples from a file
    pub fn new(source: R) -> Self {
        Samples(Chunks::new(source).flat_map(flatten_vec))
    }
}

impl<R> Iterator for Samples<R>
where
    R: Read,
{
    type Item = Result<Sample>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<R> ReadInput for Samples<R>
where
    R: Read,
{
    fn sample_rate(&self) -> f32 {
        // Just for testing, this probably doesn't matter
        100_000_000.0
    }

    fn bins(&self) -> u16 {
        CHUNK_SAMPLES.try_into().unwrap()
    }

    fn set_stop_flag(&mut self, _stop: Arc<AtomicBool>) {
        // Do nothing
    }

    fn read_samples(
        &mut self,
        samples: &mut [Sample],
    ) -> std::result::Result<usize, Box<dyn Error>> {
        let mut samples_read = 0;
        for sample in samples {
            match self.next() {
                Some(Ok(sample_read)) => *sample = sample_read,
                Some(Err(e)) => return Err(e.into()),
                None => break,
            }
            samples_read += 1;
        }

        Ok(samples_read)
    }
}

fn flatten_vec(chunk: Result<Chunk>) -> Box<dyn Iterator<Item = Result<Sample>> + Send> {
    match chunk {
        Ok(chunk) => flatten_chunk(chunk),
        Err(e) => Box::new(iter::once(Err(e))),
    }
}

fn flatten_chunk(chunk: Chunk) -> Box<dyn Iterator<Item = Result<Sample>> + Send> {
    let amplitudes = chunk.amplitudes;
    let (real_values, imaginary_values) = amplitudes.split_at(amplitudes.len() / 2);
    let time = chunk.time;
    let samples = real_values
        .iter()
        .zip(imaginary_values.iter())
        .enumerate()
        .map(move |(index, (real, imaginary))| {
            let sample = Sample {
                time: time as u32,
                index: index as u16,
                amplitude: Complex32::new(*real, *imaginary),
            };
            Ok(sample)
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(samples.len(), CHUNK_SAMPLES);
    Box::new(samples.into_iter())
}

/// Reads 2048-sample chunks from a byte source
struct Chunks<R> {
    /// Byte source
    bytes: R,
    /// Index (time field) of next chunk that will be read
    index: u64,
}

impl<R> Chunks<R> {
    pub fn new(bytes: R) -> Self {
        Chunks {
            bytes,
            // Start index (time) at 1, because the decompression doesn't always work correctly
            // with time values of 0
            index: 1,
        }
    }
}

impl<R> Iterator for Chunks<R>
where
    R: Read,
{
    type Item = Result<Chunk>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut samples = vec![0f32; CHUNK_NUMBERS];
        for (i, sample) in samples.iter_mut().enumerate() {
            *sample = match self.bytes.read_f64::<LE>() {
                Ok(value) => value as f32,
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        if i == 0 {
                            // Haven't read anything yet for this chunk
                            return None;
                        } else {
                            // Partial chunk!
                            return Some(Err(e));
                        }
                    } else {
                        // Some other error
                        return Some(Err(e));
                    }
                }
            };
        }

        let chunk = Chunk {
            time: self.index,
            amplitudes: samples,
        };
        self.index = self.index.checked_add(1).expect("Index overflow");

        Some(Ok(chunk))
    }
}
