/*
 * Copyright 2020 The Regents of the University of California
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
//! USRP N210 input
//!

use std::error::Error;
use std::{cmp, mem};

use uhd::{
    ReceiveErrorKind, ReceiveStreamer, StreamArgs, StreamCommand, StreamCommandType, StreamTime,
    TuneRequest, TuneResult, Usrp,
};

use super::format::n210::{parse_sample, N210Sample, SAMPLE_BYTES};
use super::{ReadInput, Sample};
use num_complex::Complex;
use std::convert::TryInto;
use std::ops::Not;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Size of FFT used for compression
const BINS: u16 = 2048;

/// Sample rate used to receive time-domain samples for compression
const SAMPLE_RATE: f32 = 100e6;

/// Maximum of Complex<i16> samples to read at a time
///
/// How we got this: The default receive frame size is 1472 bytes = 368 Complex<i16>s
/// Then we rounded up to the nearest power of two.
const RECEIVE_BUFFER_SIZE: usize = 512;

mod registers {
    /// Scaling (what is this?)
    pub const SCALING: u8 = 10;
    /// Per-bin threshold set command
    pub const THRESHOLD: u8 = 11;
    /// Per-bin mask set command
    pub const MASK: u8 = 12;
    /// Average weight
    pub const AVG_WEIGHT: u8 = 13;
    /// Average interval
    pub const AVG_INTERVAL: u8 = 14;
    /// Enable FFT sample sending
    pub const FFT_SEND: u8 = 15;
    /// Enable average sample sending
    pub const AVG_SEND: u8 = 16;
    /// Enable FFT
    pub const RUN_FFT: u8 = 17;
    /// Register to enable/disable compression
    pub const ENABLE_COMPRESSION: u8 = 19;
    /// FFT size
    pub const FFT_SIZE: u8 = 20;
}

/// An input that reads from a USRP N210
///
/// When start() is called, compression is automatically enabled. Client code does not need to
/// manually enable it.
pub struct N210<'usrp> {
    /// USRP
    usrp: &'usrp Usrp,
    /// Receive stream
    stream: ReceiveStreamer<'usrp, Complex<i16>>,
    /// The USRP motherboard number to control (usually 0)
    mboard: usize,
    /// The receive channel number to use (usually 0)
    channel: usize,
    /// Stop flag
    stop: Arc<AtomicBool>,

    // Diagnostics section
    /// The time when the last data sample was received (or when read_samples() was first called,
    /// if no data sample has been received yet)
    last_data_sample_time: Option<Instant>,
    average_collector: AverageCollector,
    /// If a mask is enabled for each bin
    masks_enabled: [bool; 2048],
}

impl<'usrp> N210<'usrp> {
    /// Creates a receiver that uses a USRP
    ///
    /// mboard and channel are normally 0, but other values may be needed if multiple USRPs
    /// are connected.
    pub fn new(usrp: &'usrp Usrp, mboard: usize, channel: usize) -> Result<Self, uhd::Error> {
        // Arguments: Stream the selected channel only
        let args = StreamArgs::<Complex<i16>>::builder()
            .channels(vec![channel])
            .build();
        let stream = usrp.get_rx_stream(&args)?;
        Ok(N210 {
            usrp,
            stream,
            mboard,
            channel,
            stop: Arc::new(AtomicBool::new(false)),
            last_data_sample_time: None,
            average_collector: AverageCollector::new(),
            masks_enabled: [false; 2048],
        })
    }

    /// Sets the center frequency for receiving
    pub fn set_frequency(&mut self, frequency: &TuneRequest) -> Result<TuneResult, uhd::Error> {
        self.usrp.set_rx_frequency(frequency, self.channel)
    }

    /// Sets the antenna used to receive
    pub fn set_antenna(&mut self, antenna: &str) -> Result<(), uhd::Error> {
        self.usrp.set_rx_antenna(antenna, self.channel)
    }

    /// Sets the receive gain for the gain element with the provided name
    ///
    /// If name is empty, a gain element will be chosen automatically.
    pub fn set_gain(&mut self, gain: f64, name: &str) -> Result<(), uhd::Error> {
        self.usrp.set_rx_gain(gain, self.channel, name)
    }

    /// Enables or disables compression
    ///
    /// When compression is disabled, the USRP will send uncompressed samples as if it were using
    /// the standard FPGA image.
    pub fn set_compression_enabled(&mut self, enabled: bool) -> Result<(), uhd::Error> {
        self.usrp
            .set_user_register(registers::ENABLE_COMPRESSION, enabled as u32, self.mboard)
    }

    /// Enables/disables running the FFT
    pub fn set_fft_enabled(&mut self, enabled: bool) -> Result<(), uhd::Error> {
        self.usrp
            .set_user_register(registers::RUN_FFT, enabled as u32, self.mboard)
    }

    /// Enables/disables sending of FFT samples
    pub fn set_fft_send_enabled(&mut self, enabled: bool) -> Result<(), uhd::Error> {
        self.usrp
            .set_user_register(registers::FFT_SEND, enabled as u32, self.mboard)
    }

    /// Enables/disables sending of average samples
    pub fn set_average_send_enabled(&mut self, enabled: bool) -> Result<(), uhd::Error> {
        self.usrp
            .set_user_register(registers::AVG_SEND, enabled as u32, self.mboard)
    }

    /// Enables the FFT, sending of FFT samples, and sending of average samples
    pub fn start_all(&mut self) -> Result<(), uhd::Error> {
        self.set_fft_send_enabled(true)?;
        self.set_average_send_enabled(true)?;
        self.set_fft_enabled(true)?;
        Ok(())
    }

    /// Disables the FFT, sending of FFT samples, and sending of average samples
    pub fn stop_all(&mut self) -> Result<(), uhd::Error> {
        self.set_fft_send_enabled(false)?;
        self.set_average_send_enabled(false)?;
        self.set_fft_enabled(false)?;
        Ok(())
    }

    /// Sets the number of bins used for the FFT
    pub fn set_fft_size(&mut self, size: u32) -> Result<(), uhd::Error> {
        self.usrp
            .set_user_register(registers::FFT_SIZE, size, self.mboard)
    }

    /// Sets the FFT scaling factor (what is this?)
    pub fn set_fft_scaling(&mut self, scaling: u32) -> Result<(), uhd::Error> {
        self.usrp
            .set_user_register(registers::SCALING, scaling, self.mboard)
    }

    /// Sets the threshold for one bin
    ///
    /// TODO: Threshold units and more documentation
    pub fn set_threshold(&mut self, index: u16, threshold: u32) -> Result<(), uhd::Error> {
        // Register format:
        // Bits 31:21 : index (11 bits)
        // Bits 20:0 : threshold shifted right by 11 bits (21 bits)

        // Check that index fits within 11 bits
        assert!(index <= 0x7ff, "index must fit within 11 bits");

        let command: u32 = (u32::from(index) << 21) | (threshold >> 11);
        self.usrp
            .set_user_register(registers::THRESHOLD, command, self.mboard)
    }

    /// Enables or disables masking for one bin
    pub fn set_mask_enabled(&mut self, index: u16, enabled: bool) -> Result<(), uhd::Error> {
        self.masks_enabled[usize::from(index)] = enabled;
        // Register format:
        // Bits 31:1 : index (31 bits)
        // Bit 0 : set mask (1) / clear mask (0)

        let command: u32 = (u32::from(index) << 1) | enabled as u32;
        self.usrp
            .set_user_register(registers::MASK, command, self.mboard)
    }

    /// Sets the average weight
    ///
    /// TODO: What is this?, more documentation
    pub fn set_average_weight(&mut self, weight: f32) -> Result<(), uhd::Error> {
        assert!(
            weight >= 0.0 && weight <= 1.0,
            "weight must be in the range [0, 1]"
        );

        // Map to 0...255
        let mapped = (weight * 255.0) as u8;
        self.usrp
            .set_user_register(registers::AVG_WEIGHT, u32::from(mapped), self.mboard)
    }

    /// Sets the interval between sets of average samples
    ///
    /// TODO: What units?
    pub fn set_average_packet_interval(&mut self, interval: u32) -> Result<(), uhd::Error> {
        assert_ne!(interval, 0, "interval must not be 0");

        // Register format: ceiling of the base-2 logarithm of the interval
        let ceiling_log_interval = 31 - interval.leading_zeros();
        self.usrp
            .set_user_register(registers::AVG_INTERVAL, ceiling_log_interval, self.mboard)
    }

    /// Stops and then resumes the operation of the USRP
    fn recover_from_overflow(&mut self) -> Result<(), uhd::Error> {
        self.stop_all()?;
        self.stream.send_command(&StreamCommand {
            command_type: StreamCommandType::StopContinuous,
            time: StreamTime::Now,
        })?;
        self.start_all()?;
        self.stream.send_command(&StreamCommand {
            command_type: StreamCommandType::StartContinuous,
            time: StreamTime::Now,
        })?;
        Ok(())
    }
}

impl ReadInput for N210<'_> {
    fn sample_rate(&self) -> f32 {
        SAMPLE_RATE
    }

    fn bins(&self) -> u16 {
        BINS
    }

    fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.start_all()?;
        // Start sending samples now and expect to continue forever
        self.stream.send_command(&StreamCommand {
            time: StreamTime::Now,
            command_type: StreamCommandType::StartContinuous,
        })?;
        Ok(())
    }

    fn set_stop_flag(&mut self, stop: Arc<AtomicBool>) {
        self.stop = stop;
    }

    /// How this works:
    /// The USRP sends chunks of data samples and chunks of average samples. All samples in each
    /// chunk have the same time value.
    /// When the time value changes and no more samples are immediately read,
    /// the input stage (higher up) may want to flush the samples out of the grouper and send them
    /// on.
    fn read_samples(&mut self, samples: &mut [Sample]) -> Result<usize, Box<dyn Error>> {
        // Calculate the number of Complex<i16>s to read. This uses double samples.len()
        // because each sample is 8 bytes (two Complex<i16>s)

        // Create an iterator over the output samples that can be used to write them
        let mut samples_out = samples.iter_mut();
        let mut samples_produced = 0usize;
        // Need to loop in case all the samples received are average samples.
        // The loop will exit if at least one data sample was received and the last sample
        // received was an average sample.
        while self.stop.load(Ordering::Relaxed).not() {
            let now = Instant::now();

            // Calculate the number of 32-bit samples we want to read from the USRP this time
            let samples_wanted = samples_out.len();
            let raw_sample_count = cmp::min(samples_wanted * 2, RECEIVE_BUFFER_SIZE);
            let mut raw_buffer = [Complex::<i16>::default(); RECEIVE_BUFFER_SIZE];
            let mut raw_buffer = &mut raw_buffer[..raw_sample_count];

            let metadata = self.stream.receive(&mut [&mut raw_buffer], 1.0, false)?;

            // List of possible outcomes:
            // If the last sample read is a data sample and the buffer has not been filled,
            // the next receive call could yield more data samples with the same time.
            // If the last sample read is an average sample, we can ignore it and return.

            // So: If the buffer has been filled, just return
            // If the buffer has not been filled and the last sample was not an average, call
            // receive again.
            // If the buffer has not been filled and the last sample was an average, return.

            match metadata.last_error() {
                Some(e) => {
                    match e.kind() {
                        ReceiveErrorKind::Timeout => {
                            // If a SIGINT/SIGHUP was received, UHD will return a Timeout error. The stop flag
                            // will also be set, so this is not an error.
                            if self.stop.load(Ordering::Relaxed) {
                                // Interrupted, no more samples
                                return Ok(0);
                            } else {
                                log::warn!("UHD RX timed out. Overflow has probably happened.");
                                self.recover_from_overflow()?;
                            }
                        }
                        ReceiveErrorKind::OutOfSequence => {
                            // UHD has already printed a "D", don't need to do anything else.
                        }
                        _ => {
                            log::error!("UHD RX error: {}", e);
                            return Err(e.into());
                        }
                    }
                }
                None => {
                    // No error, got some samples
                    let raw_received = complex_to_bytes(&raw_buffer[..metadata.samples()]);
                    // Parse samples, get data samples only, convert to generic samples, copy into output buffer
                    let samples_received = raw_received
                        .chunks_exact(SAMPLE_BYTES)
                        .map(|chunk| parse_sample(chunk.try_into().unwrap()));
                    // Copy data samples to output, and determine two things:
                    // Did we get at least one data sample?
                    // Is the last sample an average sample?
                    let (any_data_sample, last_sample_is_average) =
                        samples_received.fold((false, false), |(any_data, _is_average), sample| {
                            match sample {
                                N210Sample::Data(data_sample) => {
                                    let slot_out = samples_out
                                        .next()
                                        .expect("no out sample slot; incorrect size");
                                    *slot_out = data_sample.into();
                                    samples_produced += 1;
                                    self.last_data_sample_time = Some(now);
                                    (true, false)
                                }
                                N210Sample::Average(average_sample) => {
                                    self.average_collector.record_average(
                                        average_sample.index,
                                        average_sample.magnitude,
                                    );
                                    (any_data, true)
                                }
                            }
                        });

                    if samples_out.len() == 0 {
                        // All the samples the caller wants have been read.
                        return Ok(samples_produced);
                    } else {
                        if any_data_sample && last_sample_is_average {
                            // Didn't split up a block of data samples, can return
                            return Ok(samples_produced);
                        } else {
                            // May need to receive again to get the rest of the data samples
                            // in this block
                            /* continue */
                        }
                    }
                }
            }
        }
        // Exited loop because stop flag was set
        Ok(0)
    }
}

/// Converts a slice of complex values to a view of the same memory as bytes
fn complex_to_bytes(samples: &[Complex<i16>]) -> &[u8] {
    let ptr = samples.as_ptr();
    let scale = mem::size_of::<Complex<i16>>() / mem::size_of::<u8>();
    let length = samples.len() * scale;
    // This is safe as long as u8 does not require greater alignment than Complex<i16>.
    unsafe { std::slice::from_raw_parts(ptr as *const u8, length) }
}

/// Collects average samples to display
struct AverageCollector {
    /// The most recent magnitude from each bin
    magnitudes: [u32; 2048],
}

impl AverageCollector {
    pub fn new() -> AverageCollector {
        AverageCollector {
            magnitudes: [0; 2048],
        }
    }

    pub fn record_average(&mut self, bin: u16, magnitude: u32) {
        self.magnitudes[usize::from(bin)] = magnitude;
    }
}

impl std::fmt::Debug for AverageCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (bin, magnitude) in self.magnitudes.iter().enumerate() {
            map.entry(&bin, &magnitude);
        }
        map.finish()
    }
}
