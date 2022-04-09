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

//! This module provides a C interface to the `reconstruct` library so that non-Rust software can
//! use it

use num_complex::Complex32;
use sparsdr_sample_parser::{Parser, V1Parser, V2Parser};
use std::ffi::c_void;
use std::mem::align_of;
use std::panic::catch_unwind;
use std::ptr;

/// Version 1 of the compressed sample format as produced by a USRP N210
pub const SPARSDR_RECONSTRUCT_FORMAT_V1_N210: u32 = 1;
/// Version 1 of the compressed sample format as produced by a Pluto
pub const SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO: u32 = 2;
/// Version 2 of the compressed sample format
pub const SPARSDR_RECONSTRUCT_FORMAT_V2: u32 = 3;

pub const SPARSDR_RECONSTRUCT_OK: u32 = 0;
pub const SPARSDR_RECONSTRUCT_ERROR_INVALID_ARGUMENT: u32 = 1;
pub const SPARSDR_RECONSTRUCT_ERROR_FATAL: u32 = 2;

#[repr(C)]
pub struct SparsdrReconstructBand {
    pub frequency_offset: f32,
    pub bins: u16,
}

pub type OutputCallback =
    Option<extern "C" fn(context: *mut c_void, samples: *const Complex32, num_samples: usize)>;

/// The configuration for a reconstruction session
///
/// # Compatibility
///
/// Fields may be added to this struct in the future. The authors will not attempt to maintain
/// binary compatibility. For source compatibility, any code that creates a
/// `SparsdrReconstructConfig` will need to be updated so that it initializes all the fields.
///
/// To avoid accidentally forgetting to initialize some fields, the
/// `sparsdr_reconstruct_config_init` function can be used. That function will always initialize
/// all the fields. Subsequent code can then change the configuration as required.
///
#[repr(C)]
pub struct SparsdrReconstructConfig {
    /// One of `SPARSDR_RECONSTRUCT_FORMAT_V1_N210`, `SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO`, or
    /// `SPARSDR_RECONSTRUCT_FORMAT_V2`
    pub format: u32,
    /// FFT size used to compress the signals
    pub compression_fft_size: u32,

    /// If `bands_length` is not zero, `bands` must be a pointer to `bands_length`
    /// `SparsdrReconstructBand` objects with the bands to reconstruct.
    ///
    /// If `bands_length` is zero, `bands` may have any value.
    pub bands: *const SparsdrReconstructBand,
    /// The number of bands that `bands` points to
    pub bands_length: usize,

    /// An opaque value that will be passed to output_callback
    pub output_context: *mut c_void,
    /// A function that will be called when new reconstructed samples are ready
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
}

/// Initializes a configuration struct with the provided callback and context, no bands, and
/// other fields with implementation-defined defaults
#[no_mangle]
pub unsafe extern "C" fn sparsdr_reconstruct_config_init(
    output_callback: OutputCallback,
    output_context: *mut c_void,
    config: *mut SparsdrReconstructConfig,
) {
    // Initialize everything in a way that's safe if *config is uninitialized
    ptr::write(
        config,
        SparsdrReconstructConfig {
            format: SPARSDR_RECONSTRUCT_FORMAT_V2,
            compression_fft_size: 1024,
            bands: ptr::null(),
            bands_length: 0,
            output_context,
            output_callback,
        },
    )
}

/// A configured reconstruction context
///
/// This is opaque to non-Rust code.
pub struct SparsdrReconstructContext {
    // TODO
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
/// # Safety
///
/// `context` must be non-null, aligned, and pointing to memory where a pointer can be written.
///
/// `config` must be non-null, aligned, and pointing to a `SparsdrReconstructConfig` object with
/// all fields properly initialized.
///
/// Some violations of these requirements are detected and cause this function to return error
/// codes, but others are unavoidable undefined behavior.
///
#[no_mangle]
pub extern "C" fn sparsdr_reconstruct_init(
    context: *mut *mut SparsdrReconstructContext,
    config: *const SparsdrReconstructConfig,
) -> u32 {
    // Prevent unwinding through FFI
    let status = catch_unwind(|| {
        // Check pointers
        if !is_non_null_and_aligned(context) || !is_non_null_and_aligned(config) {
            return SPARSDR_RECONSTRUCT_ERROR_INVALID_ARGUMENT;
        }
        // Since config is non-null and aligned, some (but not all) of the unsafety in this line has been eliminated.
        let config: &SparsdrReconstructConfig = unsafe { &*config };

        let output_callback = match config.output_callback {
            Some(output_callback) => output_callback,
            None => return SPARSDR_RECONSTRUCT_ERROR_INVALID_ARGUMENT,
        };

        let parser: Box<dyn Parser> = match config.format {
            SPARSDR_RECONSTRUCT_FORMAT_V1_N210 => {
                Box::new(V1Parser::new_n210(config.compression_fft_size as usize))
            }
            SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO => {
                Box::new(V1Parser::new_pluto(config.compression_fft_size as usize))
            }
            SPARSDR_RECONSTRUCT_FORMAT_V2 => Box::new(V2Parser::new(config.compression_fft_size)),
            _ => return SPARSDR_RECONSTRUCT_ERROR_INVALID_ARGUMENT,
        };

        let context_box = Box::new(SparsdrReconstructContext {});
        let new_context_ptr = Box::into_raw(context_box);
        unsafe { *context = new_context_ptr };
        SPARSDR_RECONSTRUCT_OK
    });
    match status {
        Ok(status_code) => status_code,
        Err(_) => SPARSDR_RECONSTRUCT_ERROR_FATAL,
    }
}

#[no_mangle]
pub extern "C" fn sparsdr_reconstruct_handle_samples(
    context: *mut SparsdrReconstructContext,
    samples: *const c_void,
    num_bytes: usize,
) -> u32 {
    let status = catch_unwind(|| {
        todo!();
    });
    match status {
        Ok(()) => SPARSDR_RECONSTRUCT_OK,
        Err(_) => SPARSDR_RECONSTRUCT_ERROR_FATAL,
    }
}

#[no_mangle]
pub extern "C" fn sparsdr_reconstruct_free(context: *mut SparsdrReconstructContext) {
    // Prevent unwinding through FFI
    let _ = catch_unwind(|| {
        let context_box = unsafe { Box::from_raw(context) };
        drop(context_box)
    });
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
