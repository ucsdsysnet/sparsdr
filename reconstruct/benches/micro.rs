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

use std::iter;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use num_complex::Complex32;
use sparsdr_reconstruct::bins::BinRange;
use sparsdr_reconstruct::blocking::BlockLogger;
use sparsdr_reconstruct::input::Sample;
use sparsdr_reconstruct::steps::fft::Fft;
use sparsdr_reconstruct::steps::filter_bins::FilterBinsIter;
use sparsdr_reconstruct::steps::frequency_correct::FrequencyCorrectIter;
use sparsdr_reconstruct::steps::group::Grouper;
use sparsdr_reconstruct::steps::overlap::Overlap;
use sparsdr_reconstruct::steps::phase_correct::PhaseCorrectIter;
use sparsdr_reconstruct::steps::shift::ShiftIter;
use sparsdr_reconstruct::steps::writer;
use sparsdr_reconstruct::window::{Status, TimeWindow, Window};

const COMPRESSION_BINS: u16 = 2048;

fn benchmark_fft(c: &mut Criterion) {
    let sizes_and_counts = [
        (2048_usize, 100_usize),
        (1024, 100),
        (64, 100),
        (16, 100),
        (4, 100),
    ];
    {
        let mut group = c.benchmark_group("FFT setup and teardown");
        for (size, count) in sizes_and_counts.iter() {
            group.bench_with_input(
                format!("size {}, count {}", *size, *count),
                &(*size, *count),
                |b, &(size, count)| {
                    b.iter(|| {
                        let windows = iter::repeat(Status::Ok(Window::new(0, size))).take(count);
                        let _fft = Fft::new(windows, size, usize::from(COMPRESSION_BINS));
                    })
                },
            );
        }
    }
    {
        let mut group = c.benchmark_group("FFT run");
        for (size, count) in sizes_and_counts.iter() {
            group.bench_with_input(
                format!("size {}, count {}", *size, *count),
                &(*size, *count),
                |b, &(size, count)| {
                    let windows = iter::repeat(Status::Ok(Window::new(0, size)));
                    let mut fft = Fft::new(windows, size, usize::from(COMPRESSION_BINS));
                    b.iter(|| for _ in fft.by_ref().take(count) {})
                },
            );
        }
    }
}

fn benchmark_filter_bins(c: &mut Criterion) {
    let window_size = 2408;
    let count = 100;

    c.bench_function("step_filter_bins_no_match", move |b| {
        b.iter_batched(
            || {
                let windows =
                    iter::repeat(Status::Ok(Window::new_logical(0, window_size))).take(count);
                let bins = BinRange::from(512..768);
                FilterBinsIter::new(windows, bins, window_size as u16)
            },
            |step| {
                for _window in step {}
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("step_filter_bins_all_match", move |b| {
        b.iter_batched(
            || {
                let mut window = Window::new_logical(0, window_size);
                // Put a non-empty value in the middle
                window.bins_mut()[window_size / 2] = Complex32::new(6.21, 9.26);
                let windows = iter::repeat(Status::Ok(window)).take(count);

                let bins = BinRange::from(512..768);
                FilterBinsIter::new(windows, bins, window_size as u16)
            },
            |step| {
                for _window in step {}
            },
            BatchSize::SmallInput,
        )
    });
}

fn benchmark_frequency_correct(c: &mut Criterion) {
    c.bench_function("step_frequency_correct", |b| {
        b.iter_batched(
            || {
                let window = TimeWindow::new(0, vec![Complex32::new(0.0, 0.0); 2048]);
                let windows = iter::repeat(window).take(100);
                FrequencyCorrectIter::new(windows, 0.3, 2048)
            },
            |step| {
                for _window in step {}
            },
            BatchSize::SmallInput,
        )
    });
}

struct GenerateSamples {
    /// Time of next sample
    time: u32,
    /// Index of next sample
    index: u16,
    /// FFT size (determines maximum index)
    fft_size: u16,
}

impl GenerateSamples {
    pub fn new(fft_size: u16) -> Self {
        GenerateSamples {
            time: 0,
            index: 0,
            fft_size,
        }
    }
}

impl Iterator for GenerateSamples {
    type Item = Sample;
    fn next(&mut self) -> Option<Self::Item> {
        let sample = Sample {
            time: self.time,
            index: self.index,
            amplitude: Complex32::new(0.0, 0.0),
        };

        self.time = self.time.wrapping_add(1);
        self.index += 1;
        if self.index == self.fft_size {
            self.index = 0;
        }

        Some(sample)
    }
}

fn benchmark_grouper(c: &mut Criterion) {
    c.bench_function("step_grouper", |b| {
        b.iter_batched(
            || {
                let samples = GenerateSamples::new(2048).map(Ok).take(2048 * 2);
                Grouper::new(samples, 2048)
            },
            |step| {
                for _window in step {}
            },
            BatchSize::SmallInput,
        )
    });
}

struct GenerateTimeWindows {
    /// Time of the next window
    time: u64,
    /// Window size
    window_size: usize,
}

impl GenerateTimeWindows {
    pub fn new(window_size: usize) -> Self {
        GenerateTimeWindows {
            time: 0,
            window_size,
        }
    }
}

impl Iterator for GenerateTimeWindows {
    type Item = TimeWindow;
    fn next(&mut self) -> Option<Self::Item> {
        let window = TimeWindow::new(self.time, vec![Complex32::new(0.0, 0.0); self.window_size]);
        self.time = self.time.wrapping_add(1);
        Some(window)
    }
}

fn benchmark_overlap(c: &mut Criterion) {
    c.bench_function("step_overlap", |b| {
        b.iter_batched(
            || {
                let windows = GenerateTimeWindows::new(2048).map(Status::Ok).take(100);
                Overlap::new(windows, 2048)
            },
            |step| {
                for _window in step {}
            },
            BatchSize::SmallInput,
        )
    });
}

fn benchmark_phase_correct(c: &mut Criterion) {
    c.bench_function("step_phase_correct", |b| {
        b.iter_batched(
            || {
                let window = Window::new(0, 2048);
                let windows = iter::repeat(window).map(Status::Ok).take(100);
                PhaseCorrectIter::new(windows, 13.0)
            },
            |step| {
                for _window in step {}
            },
            BatchSize::SmallInput,
        )
    });
}

fn benchmark_shift(c: &mut Criterion) {
    c.bench_function("step_shift", |b| {
        b.iter_batched(
            || {
                let window = Window::new_logical(0, 2048);
                let windows = iter::repeat(window).map(Status::Ok).take(100);
                ShiftIter::new(windows, 2048)
            },
            |step| {
                for _window in step {}
            },
            BatchSize::SmallInput,
        )
    });
}

fn benchmark_window(c: &mut Criterion) {
    c.bench_function("window_clone_owned", |b| {
        b.iter_batched(
            || {
                let mut window = Window::new(0, 2048);
                // Modify the window to make it owned
                window.bins_mut()[0].re = 3.91;
                window
            },
            |window| {
                let _window1 = window.clone();
                let _window2 = window.clone();
            },
            BatchSize::SmallInput,
        )
    });
}

fn benchmark_write(c: &mut Criterion) {
    c.bench_function("step_write", |b| {
        b.iter_batched(
            || {
                let window = TimeWindow::new(0, vec![Complex32::new(0.0, 0.0); 2048]);
                let windows = iter::repeat(window).take(100);
                windows
            },
            |windows| {
                let destination = std::io::sink();
                let mut writer = writer::Writer::new();
                let block_logger = BlockLogger::default();
                writer
                    .write_windows(destination, windows, &block_logger, None)
                    .unwrap();
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    benchmark_fft,
    benchmark_filter_bins,
    benchmark_frequency_correct,
    benchmark_window,
    benchmark_grouper,
    benchmark_overlap,
    benchmark_phase_correct,
    benchmark_shift,
    benchmark_write,
);
criterion_main!(benches);
