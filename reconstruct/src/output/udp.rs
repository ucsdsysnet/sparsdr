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
//! Output over a UDP socket
//!

use super::WriteOutput;
use byteorder::{ByteOrder, WriteBytesExt};
use num_complex::Complex32;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::error::Error;
use std::marker::PhantomData;
use std::net::{SocketAddr, UdpSocket};
use std::{io, mem};

/// Size of a sample in bytes
const SAMPLE_SIZE: usize = mem::size_of::<Complex32>();

/// Writes samples to a UDP socket, with configurable headers
///
/// T should be a byte order (BigEndian/LittleEndian/NativeEndian/NetworkEndian). When T matches
/// the native byte order of the computer receiving the packets, the packet format is compatible
/// with a GNU Radio UDP source.
pub struct UdpOutput<T, H>
where
    T: ByteOrder,
    H: HeaderGenerator,
{
    /// Logic to generate headers
    header_generator: H,
    /// Actual socket
    socket: UdpSocket,
    /// Maximum number of bytes to write for each UDP packet (this includes any headers)
    mtu: usize,
    /// Queue of samples that need to be put into packets and sent
    sample_queue: VecDeque<Complex32>,
    /// Buffer of self.mtu bytes, used to assemble packets
    buffer: Box<[u8]>,
    /// Phantom byteorder type
    order_phantom: PhantomData<T>,
}

impl<T, H> UdpOutput<T, H>
where
    T: ByteOrder,
    H: HeaderGenerator,
{
    /// Creates a UDP output
    ///
    /// local_addr: The local address and port to bind to
    ///
    /// remote_addr: The remote address and port to send packets to
    ///
    /// mtu: The maximum number of bytes to send in each packet (including any headers added by
    /// the header generator). This is normally 1472 for a standard network link. The provided
    /// value will be rounded down so that (mtu - H::LENGTH) is a multiple of the size of a sample.
    ///
    /// header_generator: The logic that generates headers
    ///
    /// This function returns an error if a socket operation fails, or if (mtu - H::LENGTH) is
    /// less than the size of a sample.
    pub fn new(
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
        mtu: usize,
        header_generator: H,
    ) -> Result<Self, io::Error> {
        // Check that the MTU is large enough to fit at least one sample in each packet
        let bytes_for_samples = mtu.saturating_sub(H::LENGTH);
        if bytes_for_samples < SAMPLE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MTU not large enough for headers and one sample",
            ));
        }
        // Round down MTU
        let rounded_mtu = H::LENGTH + (bytes_for_samples / SAMPLE_SIZE) * SAMPLE_SIZE;

        let socket = UdpSocket::bind(local_addr)?;
        socket.connect(remote_addr)?;
        Ok(UdpOutput {
            header_generator,
            socket,
            mtu: rounded_mtu,
            sample_queue: VecDeque::new(),
            buffer: vec![0u8; rounded_mtu].into_boxed_slice(),
            order_phantom: PhantomData,
        })
    }

    /// Takes `samples` samples from self.sample_queue, writes them to a packet, and sends the
    /// packet
    fn write_some_samples(&mut self, samples: usize) -> Result<(), io::Error> {
        let packet_length = H::LENGTH + samples * SAMPLE_SIZE;
        debug_assert!(
            self.buffer.len() >= packet_length,
            "buffer not large enough"
        );

        let buffer = &mut self.buffer[..packet_length];
        self.header_generator
            .write_header(packet_length.try_into().unwrap(), &mut buffer[..H::LENGTH]);
        let mut sample_buffer = &mut buffer[H::LENGTH..];
        for sample in self.sample_queue.drain(..samples) {
            sample_buffer
                .write_f32::<T>(sample.re)
                .expect("incorrect buffer length");
            sample_buffer
                .write_f32::<T>(sample.im)
                .expect("incorrect buffer length");
        }

        let bytes_sent = self.socket.send(buffer)?;
        if bytes_sent == packet_length {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Tried to send {} bytes as a UDP packet, but only {} were sent",
                    packet_length, bytes_sent
                ),
            ))
        }
    }

    fn max_samples_per_packet(&self) -> usize {
        debug_assert_eq!(
            (self.mtu - H::LENGTH) % SAMPLE_SIZE,
            0,
            "MTU not rounded down"
        );
        (self.mtu - H::LENGTH) / SAMPLE_SIZE
    }
}

impl<T, H> WriteOutput for UdpOutput<T, H>
where
    T: ByteOrder,
    H: HeaderGenerator,
{
    fn write_samples(&mut self, samples: &[Complex32]) -> Result<(), Box<dyn Error + Send>> {
        // Consider optimizing this later
        self.sample_queue.extend(samples);
        let max_samples_per_packet = self.max_samples_per_packet();
        // Write full packets until there are not enough samples to fill a packet.
        // The remaining samples will be written once more samples arrive, or when flush() is called
        while self.sample_queue.len() >= max_samples_per_packet {
            self.write_some_samples(max_samples_per_packet)
                .map_err(box_err)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Box<dyn Error + Send>> {
        let max_samples_per_packet = self.max_samples_per_packet();
        // Write full packets until there are not enough samples to fill a packet
        while self.sample_queue.len() >= max_samples_per_packet {
            self.write_some_samples(max_samples_per_packet)
                .map_err(box_err)?;
        }
        // Write remaining samples
        self.write_some_samples(self.sample_queue.len())
            .map_err(box_err)?;
        Ok(())
    }
}

impl<T, H> Drop for UdpOutput<T, H>
where
    T: ByteOrder,
    H: HeaderGenerator,
{
    /// Calls self.flush() and ignores any errors
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Trait for things that can generate headers
pub trait HeaderGenerator {
    /// Number of bytes required for this header
    const LENGTH: usize;

    /// Writes a header to the beginning of a buffer
    ///
    /// packet_length is the length in bytes of this packet, including headers added by this
    /// header generator.
    ///
    /// The buffer length will be at least Self::LENGTH.
    fn write_header(&mut self, packet_length: usize, destination: &mut [u8]);
}

/// Does not add headers
pub struct NoHeaders;

impl HeaderGenerator for NoHeaders {
    const LENGTH: usize = 0;

    fn write_header(&mut self, _packet_length: usize, _destination: &mut [u8]) {
        // Nothing
    }
}

/// Adds a 64-bit sequence number (starting at 1)
///
/// T should be a byte order (BigEndian/LittleEndian/NativeEndian/NetworkEndian). When T matches
/// the native byte order of the computer receiving the packets, this header format is compatible
/// with GNU Radio HEADERTYPE_SEQNUM.
pub struct SequenceHeaders<T> {
    /// Sequence number to apply to the next packet
    next: u64,
    /// Byte order phantom data
    order_phantom: PhantomData<T>,
}

impl<T> SequenceHeaders<T> {
    /// Creates a header generator
    pub fn new() -> Self {
        SequenceHeaders {
            next: 0,
            order_phantom: PhantomData,
        }
    }
}

impl<T> HeaderGenerator for SequenceHeaders<T>
where
    T: ByteOrder,
{
    const LENGTH: usize = 8;

    fn write_header(&mut self, _packet_length: usize, destination: &mut [u8]) {
        T::write_u64(destination, self.next);
        self.next = self.next.wrapping_add(1);
    }
}

/// Writes a 64-bit sequence number, followed by a 16-bit length and 6 zero bytes
///
/// T should be a byte order (BigEndian/LittleEndian/NativeEndian/NetworkEndian). When T matches
/// the native byte order of the computer receiving the packets, this header format is compatible
/// with GNU Radio HEADERTYPE_SEQPLUSSIZE.
pub struct SequenceAndSizeHeaders<T> {
    /// Sequence number to apply to the next packet
    next: u64,
    /// Byte order phantom data
    order_phantom: PhantomData<T>,
}

impl<T> SequenceAndSizeHeaders<T> {
    /// Creates a header generator
    ///
    /// mtu: The number of bytes in each packet, including the sequence and size headers
    pub fn new() -> Self {
        SequenceAndSizeHeaders {
            next: 0,
            order_phantom: PhantomData,
        }
    }
}

impl<T> HeaderGenerator for SequenceAndSizeHeaders<T>
where
    T: ByteOrder,
{
    // Although there are only 10 bytes of data, we need to allocate 16 bytes for alignment
    // because the GNU Radio code uses sizeof. See:
    // https://github.com/gnuradio/gnuradio/blob/4849af9ad83bc16d21e2c63ab95bab1755289c2f/gr-network/include/gnuradio/network/packet_headers.h
    // https://github.com/gnuradio/gnuradio/blob/4849af9ad83bc16d21e2c63ab95bab1755289c2f/gr-network/lib/udp_source_impl.cc
    const LENGTH: usize = 16;

    fn write_header(&mut self, packet_length: usize, destination: &mut [u8]) {
        assert!(destination.len() >= Self::LENGTH);
        let packet_length: u16 = packet_length
            .try_into()
            .expect("packet length too large for u16");

        T::write_u64(destination, self.next);
        T::write_u16(&mut destination[8..10], packet_length);
        // Fill in with zero
        destination[10..16].copy_from_slice(&[0; 6]);

        self.next = self.next.wrapping_add(1);
    }
}

fn box_err(e: std::io::Error) -> Box<dyn Error + Send> {
    Box::new(e)
}
