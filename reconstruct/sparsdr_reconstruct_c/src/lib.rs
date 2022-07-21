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

//! This library provides a C interface to the `reconstruct` library so that non-Rust software can
//! use it

extern crate num_complex;
extern crate sparsdr_reconstruct;
extern crate sparsdr_sample_parser;

use num_complex::Complex32;
use sparsdr_reconstruct::push_reconstruct::{Reconstruct, WriteSamples};
use sparsdr_reconstruct::steps::overlap::OverlapMode;
use sparsdr_reconstruct::{BandSetupBuilder, DecompressSetup};
use sparsdr_sample_parser::{Parser, V1Parser, V2Parser};
use std::ffi::c_void;
use std::mem::align_of;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::{ptr, slice};

/// Version 1 of the compressed sample format as produced by a USRP N210
pub const SPARSDR_RECONSTRUCT_FORMAT_V1_N210: u32 = 1;
/// Version 1 of the compressed sample format as produced by a Pluto
pub const SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO: u32 = 2;
/// Version 2 of the compressed sample format
pub const SPARSDR_RECONSTRUCT_FORMAT_V2: u32 = 3;

pub const SPARSDR_RECONSTRUCT_OK: u32 = 0;
pub const SPARSDR_RECONSTRUCT_ERROR_INVALID_ARGUMENT: u32 = 1;
pub const SPARSDR_RECONSTRUCT_ERROR_FATAL: u32 = 2;

/// Function pointer used for output callbacks
pub type OutputCallback =
    Option<extern "C" fn(context: *mut c_void, samples: *const Complex32, num_samples: usize)>;

/// Settings for a band to reconstruct
#[repr(C)]
pub struct Band {
    pub frequency_offset: f32,
    pub bins: u16,
    /// A function that will be called when new reconstructed samples are ready on this band
    ///
    /// The function takes these arguments:
    /// * `context`: The `output_context` value used when creating the SparSDR context
    /// * `samples`: A pointer to zero or more 32-bit float complex values
    /// * `num_samples`: The number of samples to be read
    ///
    /// # Safety
    ///
    /// This function may be called at any time from many different threads. It must use the
    /// necessary methods to ensure thread-safety.
    ///
    pub output_callback: OutputCallback,
    /// An opaque value that will be passed to output_callback
    pub output_context: *mut c_void,
}

/// The configuration for a reconstruction session
///
/// # Compatibility
///
/// Fields may be added to this struct in the future. The authors will not attempt to maintain
/// binary compatibility. For source compatibility, any code that creates a
/// config will need to be updated so that it initializes all the fields.
///
/// To avoid accidentally forgetting to initialize some fields, the
/// `sparsdr_reconstruct_config_init` function can be used. That function will always initialize
/// all the fields. Subsequent code can then change the configuration as required.
///
#[repr(C)]
pub struct Config {
    /// One of `SPARSDR_RECONSTRUCT_FORMAT_V1_N210`, `SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO`, or
    /// `SPARSDR_RECONSTRUCT_FORMAT_V2`
    pub format: u32,
    /// FFT size used to compress the signals
    pub compression_fft_size: u32,
    /// Bandwidth (or equivalently sample rate) used to capture the signals
    pub compressed_bandwidth: f32,

    /// If `bands_length` is not zero, `bands` must be a pointer to `bands_length`
    /// band objects with the bands to reconstruct.
    ///
    /// If `bands_length` is zero, `bands` may have any value.
    pub bands: *const Band,
    /// The number of bands that `bands` points to
    pub bands_length: usize,
    /// If true, the reconstruction software will add zero samples to the outputs so the
    /// timing of samples will be correct
    pub zero_gaps: bool,
}

/// Bandwidth/sample rate for N210 compression
const N210_COMPRESSED_BANDWIDTH: f32 = 100e6;

/// Initializes and returns a configuration struct with the provided callback and context, no bands,
/// and other fields with implementation-defined defaults
///
/// The returned pointer must be passed to `sparsdr_reconstruct_config_free` to deallocate it.
#[no_mangle]
pub extern "C" fn sparsdr_reconstruct_config_init() -> *mut Config {
    let config = Config {
        format: SPARSDR_RECONSTRUCT_FORMAT_V2,
        compression_fft_size: 1024,
        compressed_bandwidth: N210_COMPRESSED_BANDWIDTH,
        bands: ptr::null(),
        bands_length: 0,
        zero_gaps: false,
    };
    Box::into_raw(Box::new(config))
}

/// Deallocates a configuration
#[no_mangle]
pub unsafe extern "C" fn sparsdr_reconstruct_config_free(config: *mut Config) {
    drop(Box::from_raw(config));
}

/// A configured reconstruction context
///
/// This is opaque to non-Rust code.
pub struct Context {
    reconstruct: Reconstruct,
}

/// Creates a reconstruct context
///
/// Arguments:
/// * `context`: A pointer to a pointer to a context. If this function returns
///   `SPARSDR_RECONSTRUCT_OK`, it has initialized the pointed-to value with a valid pointer to a
///   newly allocated and initialized context.
/// * `config`: A pointer to a configuration struct
///
/// This function returns `SPARSDR_RECONSTRUCT_OK` on success, or another value if an error occurs.
///
/// If this function returns anything other than `SPARSDR_RECONSTRUCT_OK`, no other functions may
/// be called with the same context.
///
/// # Safety
///
/// `context` must be non-null, aligned, and pointing to memory where a pointer can be written.
///
/// `config` must be non-null, aligned, and pointing to a configuration object with
/// all fields properly initialized.
///
/// Some violations of these requirements are detected and cause this function to return error
/// codes, but others are unavoidable undefined behavior.
///
#[no_mangle]
pub extern "C" fn sparsdr_reconstruct_init(
    context: *mut *mut Context,
    config: *const Config,
) -> u32 {
    // Prevent unwinding through FFI
    let status = catch_unwind(AssertUnwindSafe(|| {
        // Check pointers
        if !is_non_null_and_aligned(context) || !is_non_null_and_aligned(config) {
            return SPARSDR_RECONSTRUCT_ERROR_INVALID_ARGUMENT;
        }

        let setup = match unsafe { convert_setup(config) } {
            Ok(setup) => setup,
            Err(code) => return code,
        };
        let reconstruct = match Reconstruct::start(setup) {
            Ok(reconstruct) => reconstruct,
            // TODO: Better error handling
            Err(_e) => return SPARSDR_RECONSTRUCT_ERROR_FATAL,
        };

        let context_box = Box::new(Context { reconstruct });
        let new_context_ptr = Box::into_raw(context_box);
        unsafe { *context = new_context_ptr };
        SPARSDR_RECONSTRUCT_OK
    }));
    match status {
        Ok(status_code) => status_code,
        Err(_) => SPARSDR_RECONSTRUCT_ERROR_FATAL,
    }
}

#[no_mangle]
pub unsafe extern "C" fn sparsdr_reconstruct_handle_samples(
    context: *mut Context,
    samples: *const c_void,
    num_bytes: usize,
) -> u32 {
    let status = catch_unwind(AssertUnwindSafe(|| {
        let samples_slice: &[u8] = slice::from_raw_parts(samples as *const u8, num_bytes);
        (*context).reconstruct.process_samples(samples_slice)
    }));
    match status {
        Ok(_) => SPARSDR_RECONSTRUCT_OK,
        Err(_) => SPARSDR_RECONSTRUCT_ERROR_FATAL,
    }
}

#[no_mangle]
pub extern "C" fn sparsdr_reconstruct_free(context: *mut Context) {
    // Prevent unwinding through FFI
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let context_box = unsafe { Box::from_raw(context) };
        let context = *context_box;
        context.reconstruct.shutdown();
    }));
}

/// Converts a C reconstruct configuration into a Rust reconstruct configuration
unsafe fn convert_setup(config: *const Config) -> Result<DecompressSetup, u32> {
    const N210_TIMESTAMP_BITS: u32 = 20;
    const PLUTO_TIMESTAMP_BITS: u32 = 21;
    /// Compressed format version 2, from either device, has 30 full bits of timestamp
    const V2_TIMESTAMP_BITS: u32 = 30;

    let compression_fft_size = (*config).compression_fft_size as usize;
    let timestamp_bits;
    let parser: Box<dyn Parser> = match (*config).format {
        SPARSDR_RECONSTRUCT_FORMAT_V1_N210 => {
            timestamp_bits = N210_TIMESTAMP_BITS;
            Box::new(V1Parser::new_n210(compression_fft_size))
        }
        SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO => {
            timestamp_bits = PLUTO_TIMESTAMP_BITS;
            Box::new(V1Parser::new_pluto(compression_fft_size))
        }
        SPARSDR_RECONSTRUCT_FORMAT_V2 => {
            timestamp_bits = V2_TIMESTAMP_BITS;
            Box::new(V2Parser::new(
                compression_fft_size
                    .try_into()
                    .expect("Compression FFT size too large for u32"),
            ))
        }
        _ => return Err(SPARSDR_RECONSTRUCT_ERROR_INVALID_ARGUMENT),
    };

    let mut setup = DecompressSetup::new(parser, compression_fft_size, timestamp_bits);
    if (*config).zero_gaps {
        setup.set_overlap_mode(OverlapMode::Gaps);
    } else {
        setup.set_overlap_mode(OverlapMode::Flush(0));
    }

    for i in 0..(*config).bands_length {
        let config_band: *const Band = (*config).bands.add(i);
        let frequency_offset = (*config_band).frequency_offset;
        let bins = (*config_band).bins;

        let output_callback = match (*config_band).output_callback {
            Some(output_callback) => output_callback,
            None => return Err(SPARSDR_RECONSTRUCT_ERROR_INVALID_ARGUMENT),
        };

        let destination = Box::new(SampleCallback {
            callback: output_callback,
            context: (*config_band).output_context,
        });

        setup.add_band(
            BandSetupBuilder::new(
                destination,
                (*config).compressed_bandwidth,
                compression_fft_size,
                bins,
                bins,
            )
            .center_frequency(frequency_offset)
            .build(),
        );
    }

    Ok(setup)
}

/// Returns true if the provided pointer is non-null and appropriately aligned for a value of type
/// T.
fn is_non_null_and_aligned<T>(ptr: *const T) -> bool {
    !ptr.is_null() && is_aligned(ptr)
}
/// Returns true if the provided pointer is appropriately aligned for a value of type T
fn is_aligned<T>(ptr: *const T) -> bool {
    (ptr as usize) % align_of::<T>() == 0
}

/// A callback and the associated context that passes samples to the callback
struct SampleCallback {
    callback: extern "C" fn(context: *mut c_void, samples: *const Complex32, num_samples: usize),
    context: *mut c_void,
}

/// The documentation requires that the callback can be safely called from any thread
unsafe impl Send for SampleCallback {}

impl WriteSamples for SampleCallback {
    fn write_samples(&mut self, samples: &[Complex32]) {
        (self.callback)(self.context, samples.as_ptr(), samples.len())
    }
}
