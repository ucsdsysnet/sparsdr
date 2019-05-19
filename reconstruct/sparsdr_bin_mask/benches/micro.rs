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
extern crate sparsdr_bin_mask;

use criterion::{criterion_group, criterion_main, Criterion};
use sparsdr_bin_mask::BinMask;

fn benchmark_overlap(c: &mut Criterion) {
    let bin_range = bin_range_mask();
    let window = window_active_mask();
    c.bench_function("overlap", move |bencher| {
        bencher.iter(|| bin_range.overlaps(&window))
    });
}

fn bin_range_mask() -> BinMask {
    let mut mask = BinMask::zero();
    mask.set_range(123..140);
    mask
}

fn window_active_mask() -> BinMask {
    let mut mask = BinMask::zero();
    mask.set(0, true);
    mask.set(3, true);
    mask.set(2012, true);
    mask.set(130, true);
    mask
}

criterion_group!(benches, benchmark_overlap,);
criterion_main!(benches);
