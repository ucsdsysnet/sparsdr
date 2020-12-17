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
// Keep extern crates, like in 2015
#![allow(unused_extern_crates)]

#[macro_use]
extern crate clap;
extern crate indicatif;
extern crate log;
extern crate signal_hook;
extern crate simplelog;
extern crate sparsdr_reconstruct;

use indicatif::ProgressBar;
use signal_hook::{flag::register, SIGHUP, SIGINT};
use simplelog::{Config, SimpleLogger, TermLogger, TerminalMode};
use sparsdr_reconstruct::blocking::BlockLogger;
use sparsdr_reconstruct::input::iqzip::CompressedSamples;
use sparsdr_reconstruct::{decompress, BandSetupBuilder, DecompressSetup};

mod args;
mod setup;

use std::error::Error;
use std::io::Read;
use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use self::args::Args;
use self::setup::Setup;

fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::get();
    // Logging
    let log_status = TermLogger::init(args.log_level, Config::default(), TerminalMode::Stderr)
        .or_else(|_| SimpleLogger::init(args.log_level, Config::default()));
    if let Err(e) = log_status {
        eprintln!("Failed to set up simpler logger: {}", e);
    }

    let sample_format = args.sample_format.clone();
    let setup = Setup::from_args(args)?;

    let progress = create_progress_bar(&setup);

    // Set up to read IQZip samples from file
    let in_block_logger = BlockLogger::new();
    let samples_in: CompressedSamples<'_, Box<dyn Read>> = if let Some(ref progress) = progress {
        CompressedSamples::with_block_logger(
            Box::new(progress.wrap_read(setup.source)),
            &in_block_logger,
            sample_format,
        )
    } else {
        CompressedSamples::with_block_logger(
            Box::new(setup.source),
            &in_block_logger,
            sample_format,
        )
    };

    // Set up signal handlers for clean exit
    let stop_flag = Arc::new(AtomicBool::new(false));
    register(SIGINT, Arc::clone(&stop_flag))?;
    register(SIGHUP, Arc::clone(&stop_flag))?;

    // Configure compression
    let mut decompress_setup = DecompressSetup::new(samples_in, setup.sample_format.fft_size());
    decompress_setup
        .set_channel_capacity(setup.channel_capacity)
        .set_source_block_logger(&in_block_logger)
        .set_stop_flag(Arc::clone(&stop_flag));
    if let Some(input_time_log) = setup.input_time_log {
        decompress_setup.set_input_time_log(input_time_log);
    }
    for band in setup.bands {
        let mut band_setup =
            BandSetupBuilder::new(band.destination, setup.sample_format.fft_size())
                .compressed_bandwidth(setup.sample_format.compressed_bandwidth())
                .center_frequency(band.center_frequency)
                .bins(band.bins);
        if let Some(time_log) = band.time_log {
            band_setup = band_setup.time_log(time_log);
        }
        decompress_setup.add_band(band_setup.build());
    }

    let report =
        decompress(decompress_setup).map_err(|e: Box<dyn Error + Send>| -> Box<dyn Error> { e })?;

    if let Some(progress) = progress {
        progress.finish();
    }

    if setup.report {
        eprintln!("{:#?}", report);
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
