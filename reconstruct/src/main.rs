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
//! This binary decompresses IQZip/SparSDR/whatever it's called now compressed data.
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

#[macro_use]
extern crate clap;
extern crate indicatif;
extern crate log;
extern crate signal_hook;
extern crate simplelog;
extern crate sparsdr_reconstruct;
extern crate sparsdr_sample_parser;

use indicatif::ProgressBar;
use signal_hook::{flag::register, SIGHUP, SIGINT};
use simplelog::{Config, SimpleLogger, TermLogger};
use sparsdr_reconstruct::{decompress, BandSetupBuilder, DecompressSetup};
use std::convert::TryInto;

mod args;
mod setup;

use crate::args::CompressedFormat;
use sparsdr_reconstruct::input::SampleReader;
use sparsdr_sample_parser::{Parser, V1Parser, V2Parser};
use std::io::{self, Read};
use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use self::args::Args;
use self::setup::Setup;

fn run() -> io::Result<()> {
    let args = Args::get();
    // Logging
    let log_status = TermLogger::init(args.log_level, Config::default())
        .or_else(|_| SimpleLogger::init(args.log_level, Config::default()));
    if let Err(e) = log_status {
        eprintln!("Failed to set up simpler logger: {}", e);
    }

    let parser: Box<dyn Parser> = match args.sample_format {
        CompressedFormat::V1N210 => Box::new(V1Parser::new_n210(args.compression_fft_size)),
        CompressedFormat::V1Pluto => Box::new(V1Parser::new_pluto(args.compression_fft_size)),
        CompressedFormat::V2 => Box::new(V2Parser::new(
            args.compression_fft_size
                .try_into()
                .expect("FFT size too large"),
        )),
    };

    let setup = Setup::from_args(args)?;

    let progress = create_progress_bar(&setup);

    // Set up to read windows from the source
    let windows_in: SampleReader<Box<dyn Read>, Box<dyn Parser>> =
        if let Some(ref progress) = progress {
            SampleReader::new(Box::new(Box::new(progress.wrap_read(setup.source))), parser)
        } else {
            SampleReader::new(Box::new(setup.source), parser)
        };

    // Set up signal handlers for clean exit
    let stop_flag = Arc::new(AtomicBool::new(false));
    register(SIGINT, Arc::clone(&stop_flag))?;
    register(SIGHUP, Arc::clone(&stop_flag))?;

    // Configure compression
    let mut decompress_setup =
        DecompressSetup::new(windows_in, setup.compression_fft_size, setup.timestamp_bits);
    decompress_setup
        .set_channel_capacity(setup.channel_capacity)
        .set_stop_flag(Arc::clone(&stop_flag));
    for band in setup.bands {
        let band_setup = BandSetupBuilder::new(
            band.destination,
            setup.compressed_bandwidth,
            setup.compression_fft_size,
            band.bins,
            band.fft_bins,
        )
        .center_frequency(band.center_frequency);
        decompress_setup.add_band(band_setup.build());
    }

    let _report = decompress(decompress_setup)?;

    if let Some(progress) = progress {
        progress.finish();
    }

    Ok(())
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            process::exit(-1);
        }
    }
}

/// Creates and sets up a progress bar, if requested by the setup
fn create_progress_bar(setup: &Setup) -> Option<ProgressBar> {
    if setup.progress_bar {
        // Set up progress bar if input file size is known, or spinner if it is not known
        let progress = indicatif::ProgressBar::new(setup.source_length.unwrap_or(0));
        progress.set_position(0);
        progress.set_draw_delta(1000);
        if setup.source_length.is_some() {
            // Progress bar format
            progress.set_style(
                indicatif::ProgressStyle::default_bar().template("{bar:40} {percent}% ETA {eta}"),
            );
        } else {
            // Spinner format
            progress.set_style(
                indicatif::ProgressStyle::default_spinner().template("{spinner} {msg} {bytes}"),
            );
            progress.set_message("Decompressing...");
        }
        Some(progress)
    } else {
        None
    }
}
