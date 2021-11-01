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
//! This library reconstructs signals from SparSDR compressed data.
//! It can read compressed files in multiple formats.
//!

#![deny(
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    bad_style,
    future_incompatible,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    missing_docs
)]
#![warn(clippy::all)]
#![warn(unused)]

extern crate byteorder;
extern crate fftw;
extern crate num_complex;
extern crate num_traits;
#[macro_use]
extern crate log;
extern crate crossbeam;
extern crate libc;
extern crate nix;
extern crate sparsdr_bin_mask;
extern crate sparsdr_sample_parser;

/// Converts an Option<Result<T, E>> into T, returning None if the value is None
/// or Some(Err(e)) if the value is Some(Err(e))
macro_rules! try_option_result {
    ($e:expr) => {
        match $e {
            Some(Ok(item)) => item,
            Some(Err(e)) => return Some(Err(e)),
            None => return None,
        }
    };
}

/// Converts an Option<Status<T>> into T, returning None if the value is None and returning
/// Some(Timeout) if the value is Some(Timeout)
macro_rules! try_status {
    ($e:expr) => {
        match $e {
            Some(crate::window::Status::Ok(item)) => item,
            Some(crate::window::Status::Timeout) => return Some(crate::window::Status::Timeout),
            None => return None,
        }
    };
}

// Public modules
pub mod blocking;
pub mod input;
// These are only public to allow the benchmark code to access them
pub mod bins;
pub mod iter_ext;
pub mod steps;
pub mod window;

// Private modules
mod band_decompress;
mod channel_ext;
mod component_setup;
mod decompress;
mod stages;

pub use crate::band_decompress::{BandSetup, BandSetupBuilder};
pub use crate::decompress::{decompress, DecompressSetup, Report};
