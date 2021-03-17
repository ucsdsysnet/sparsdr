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
//! Available methods of writing output samples to files or other processing elements
//!

pub mod stdio;
pub mod udp;

use std::error::Error;
use std::fs::File;
use std::io;
use std::path::Path;

use self::stdio::StdioOutput;
use self::udp::{NoHeaders, SequenceAndSizeHeaders, SequenceHeaders, UdpOutput};
use byteorder::NativeEndian;
use num_complex::Complex32;
use std::net::{SocketAddr, TcpStream};

/// Trait for writing output samples to a sink
pub trait WriteOutput {
    /// Prepares to write output
    ///
    /// This function will be called shortly before signal processing starts. The default
    /// implementation does nothing.
    fn start(&mut self) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }

    /// Writes samples from a buffer to this output
    ///
    /// This function must write all the samples in the buffer.
    fn write_samples(&mut self, samples: &[Complex32]) -> Result<(), Box<dyn Error + Send>>;

    /// Flushes all buffered samples, ensuring that they have been written out
    fn flush(&mut self) -> Result<(), Box<dyn Error + Send>>;
}

// Convenience functions for creating outputs

/// Creates a file (or truncates the file if it already exists) and returns a writer that will
/// write samples to that file
pub fn create_file<P>(path: P) -> Result<StdioOutput<File>, io::Error>
where
    P: AsRef<Path>,
{
    let file = File::create(path)?;
    Ok(StdioOutput::new(file))
}

/// Connects to a TCP server (blocking until the connection is accepted) and returns a writer that
/// will write samples to the server
pub fn tcp_client(remote_addr: SocketAddr) -> Result<StdioOutput<TcpStream>, io::Error> {
    log::info!("Opening TCP connection to {}", remote_addr);
    let socket = TcpStream::connect(remote_addr)?;
    Ok(StdioOutput::new(socket))
}

/// Creates a UDP output compatible with a GNU Radio UDP source with HEADERTYPE_NONE, assuming that
/// the GNU Radio block is running on a computer with the same endianness as the computer
/// this software is running on
///
/// local_addr: The local address to bind to
///
/// remote_addr: The remote address to send to
///
/// mtu: The maximum number of bytes in a UDP packet (normally 1472)
pub fn udp_no_headers(
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    mtu: usize,
) -> Result<UdpOutput<NativeEndian, NoHeaders>, io::Error> {
    UdpOutput::new(local_addr, remote_addr, mtu, NoHeaders)
}

/// Creates a UDP output compatible with a GNU Radio UDP source with HEADERTYPE_SEQNUM, assuming that
/// the GNU Radio block is running on a computer with the same endianness as the computer
/// this software is running on
///
/// local_addr: The local address to bind to
///
/// remote_addr: The remote address to send to
///
/// mtu: The maximum number of bytes in a UDP packet (normally 1472)
pub fn udp_sequence_headers(
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    mtu: usize,
) -> Result<UdpOutput<NativeEndian, SequenceHeaders<NativeEndian>>, io::Error> {
    UdpOutput::new(local_addr, remote_addr, mtu, SequenceHeaders::new())
}

/// Creates a UDP output compatible with a GNU Radio UDP source with HEADERTYPE_SEQPLUSSIZE, assuming that
/// the GNU Radio block is running on a computer with the same endianness as the computer
/// this software is running on
///
/// local_addr: The local address to bind to
///
/// remote_addr: The remote address to send to
///
/// mtu: The maximum number of bytes in a UDP packet (normally 1472)
pub fn udp_sequence_length_headers(
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    mtu: usize,
) -> Result<UdpOutput<NativeEndian, SequenceAndSizeHeaders<NativeEndian>>, io::Error> {
    UdpOutput::new(local_addr, remote_addr, mtu, SequenceAndSizeHeaders::new())
}
