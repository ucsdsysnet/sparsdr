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

extern crate cbindgen;

use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

use cbindgen::{Config, ExportConfig, Language, ParseConfig};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = env::var_os("CARGO_MANIFEST_DIR").expect("No CARGO_MANIFEST_DIR");
    let out_dir = env::var_os("OUT_DIR").expect("No OUT_DIR");
    // OUT_DIR is normally target/(debug|release)/build/(package name and hash)/out .
    // Go up 3 levels so the header gets placed in the same place as the library
    let header_folder = PathBuf::from(out_dir).join("../../..");

    generate_and_write(
        &manifest_path,
        &header_folder,
        cpp_config(),
        "sparsdr_reconstruct.hpp",
    )?;
    generate_and_write(
        &manifest_path,
        &header_folder,
        c_config(),
        "sparsdr_reconstruct.h",
    )?;

    Ok(())
}

fn generate_and_write(
    manifest_path: &OsStr,
    header_folder: &Path,
    config: Config,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let bindings = cbindgen::generate_with_config(manifest_path, config)?;
    let header_file = File::create(header_folder.join(file_name))?;
    bindings.write(header_file);

    Ok(())
}

/// Returns a cbindgen configuration builder with common (language-independent) settings configured
fn common_config() -> Config {
    Config {
        parse: ParseConfig {
            parse_deps: false,
            ..Default::default()
        },
        export: ExportConfig {
            rename: {
                let mut rename: HashMap<String, String> = Default::default();
                rename.insert("Band".into(), "sparsdr_reconstruct_band".into());
                rename.insert(
                    "OutputCallback".into(),
                    "sparsdr_reconstruct_output_callback".into(),
                );
                rename.insert("Config".into(), "sparsdr_reconstruct_config".into());
                rename.insert("Context".into(), "sparsdr_reconstruct_context".into());
                rename
            },
            ..Default::default()
        },
        usize_is_size_t: true,
        no_includes: true,
        ..Default::default()
    }
}

fn cpp_config() -> Config {
    let mut config = common_config();

    config.language = Language::Cxx;
    config.header = Some("/* -*- c++ -*- */\n/* Automatically generated - do not edit */".into());
    config.namespace = Some("sparsdr".into());
    config.include_guard = Some("SPARSDR_RECONSTRUCT_HPP".into());
    config.sys_includes = vec!["cstddef".into(), "cstdint".into(), "complex".into()];
    config.constant.allow_constexpr = true;
    config
        .export
        .rename
        .insert("Complex32".into(), "std::complex<float>".into());

    config
}

fn c_config() -> Config {
    let mut config = common_config();

    config.language = Language::C;
    config.header = Some("/* Automatically generated - do not edit */".into());
    config.include_guard = Some("SPARSDR_RECONSTRUCT_H".into());
    config.sys_includes = vec!["stddef.h".into(), "stdint.h".into()];
    config.constant.allow_constexpr = true;
    config
        .export
        .rename
        .insert("Complex32".into(), "float _Complex".into());

    config
}
