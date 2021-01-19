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

#![warn(
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

extern crate clap;
extern crate indicatif;
extern crate log;
extern crate signal_hook;
extern crate simplelog;
extern crate sparsdr_reconstruct;
extern crate sparsdr_reconstruct_config;

use indicatif::ProgressBar;
use signal_hook::{flag::register, SIGHUP, SIGINT};
use simplelog::{Config, SimpleLogger, TermLogger, TerminalMode};

use sparsdr_reconstruct::{decompress, BandSetupBuilder, DecompressSetup};

mod setup;

use std::error::Error;

use std::process;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use self::setup::Setup;

fn run() -> Result<(), Box<dyn Error>> {
    let config = sparsdr_reconstruct_config::config_from_command_line()?;
    // Logging
    let log_status = TermLogger::init(config.ui.log_level, Config::default(), TerminalMode::Stderr)
        .or_else(|_| SimpleLogger::init(config.ui.log_level, Config::default()));
    if let Err(e) = log_status {
        eprintln!("Failed to set up simpler logger: {}", e);
    }

    let setup = Setup::from_config(&config)?;
    let compression_bins = setup.source.bins();
    let compression_bandwidth = setup.source.sample_rate();

    let progress = create_progress_bar(&setup);

    // Notes about signals on Linux:
    // SIGINT or SIGHUP sets the stop flag to true, but does not interrupt any read calls that are
    // in progress.
    // Set up signal handlers for clean exit
    let stop_flag = Arc::new(AtomicBool::new(false));
    register(SIGINT, Arc::clone(&stop_flag))?;
    register(SIGHUP, Arc::clone(&stop_flag))?;

    // Configure compression
    let mut decompress_setup = DecompressSetup::new(setup.source, compression_bins);
    decompress_setup.set_channel_capacity(setup.channel_capacity);
    decompress_setup.set_stop_flag(stop_flag);

    for band in setup.bands {
        decompress_setup.add_band(
            BandSetupBuilder::new(band.destination, compression_bins, compression_bandwidth)
                .center_frequency(band.center_frequency)
                .bins(band.bins)
                .build(),
        );
    }

    decompress(decompress_setup).map_err(|e: Box<dyn Error + Send>| -> Box<dyn Error> { e })?;

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
