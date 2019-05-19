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
//! Buffered output to named pipes
//!

use std::fs::File;
use std::io::{self, ErrorKind, Write};
use std::mem;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::path::Path;

use nix::fcntl::{self, FcntlArg, OFlag};
use nix::sys::stat::Mode;
use nix::unistd;
use nix::Result;

/// Default buffer capacity
const DEFAULT_CAPACITY: usize = 65536;

/// Buffer states
enum State {
    /// Buffering, pipe is in nonblocking mode
    Buffering(Vec<u8>),
    /// Writing directly to the named pipe in blocking mode
    Direct,
}

/// An open named pipe (FIFO) file with an in-memory buffer
///
/// The named pipe is originally opened in nonblocking mode, which will not block waiting for
/// another process to open the pipe. In this stage, writes to a BufferedPipe will be stored in a
/// fixed-capacity buffer.
///
/// When another process opens the pipe and reads from it, the BufferedPipe will write all buffered
/// output to the pipe. All future write operations will be blocking.
///
pub struct BufferedPipe {
    /// The named pipe
    pipe: File,
    /// The state
    state: State,
}

impl BufferedPipe {
    /// Creates a buffered pipe with the default capacity
    pub fn new<P>(path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        Self::with_capacity(path, DEFAULT_CAPACITY)
    }

    /// Creates a buffered pipe with the specified capacity
    pub fn with_capacity<P>(path: P, capacity: usize) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        Self::with_capacity_nix(path, capacity).map_err(nix_to_std)
    }

    fn with_capacity_nix<P>(path: P, capacity: usize) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        // Create the pipe
        unistd::mkfifo(path.as_ref(), Mode::S_IRUSR)?;
        // Open nonblocking
        let fd = fcntl::open(
            path.as_ref(),
            OFlag::O_CLOEXEC | OFlag::O_NONBLOCK | OFlag::O_WRONLY,
            Mode::S_IRUSR,
        )?;

        let pipe = unsafe { File::from_raw_fd(fd) };
        Ok(BufferedPipe {
            pipe,
            state: State::Buffering(Vec::with_capacity(capacity)),
        })
    }

    /// Sets self.pipe to blocking mode
    fn set_blocking(&mut self) -> Result<()> {
        let flags = fcntl::fcntl(self.pipe.as_raw_fd(), FcntlArg::F_GETFD)?;
        let mut flags = OFlag::from_bits_truncate(flags);
        flags.remove(OFlag::O_NONBLOCK);
        fcntl::fcntl(self.pipe.as_raw_fd(), FcntlArg::F_SETFL(flags))?;
        Ok(())
    }
}

fn nix_to_std(err: nix::Error) -> io::Error {
    match err {
        nix::Error::Sys(errno) => io::Error::from(errno),
        nix::Error::InvalidPath => io::Error::new(ErrorKind::InvalidData, "Invalid path"),
        nix::Error::InvalidUtf8 => io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8"),
        nix::Error::UnsupportedOperation => io::Error::new(ErrorKind::Other, "Unsupported"),
    }
}

impl Write for BufferedPipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let state = mem::replace(&mut self.state, State::Direct);
        let (new_state, result) = match state {
            State::Buffering(mut buffer) => {
                // Try a nonblocking write of the whole stored buffer
                match self.pipe.write_all(&buffer) {
                    Ok(()) => {
                        // Someone read from the pipe
                        // Switch to blocking and write the new data
                        self.set_blocking().map_err(nix_to_std)?;
                        self.pipe.write(buf)?;

                        // Switch to direct mode
                        (State::Direct, Ok(buf.len()))
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                            // Add to buffer, don't change state
                            let free_capacity = buffer.capacity() - buffer.len();
                            if buf.len() > free_capacity {
                                warn!("Output buffer full, dropping bytes");
                                (
                                    State::Buffering(buffer),
                                    Err(io::Error::new(
                                        ErrorKind::WouldBlock,
                                        "Output buffer full",
                                    )),
                                )
                            } else {
                                // Enough space to add to buffer
                                buffer.extend_from_slice(buf);
                                (State::Buffering(buffer), Ok(buf.len()))
                            }
                        } else {
                            // Forward the error, don't change state
                            (State::Buffering(buffer), Err(e))
                        }
                    }
                }
            }
            State::Direct => {
                // Blocking, just write
                (State::Direct, self.pipe.write(buf))
            }
        };
        self.state = new_state;
        result
    }

    fn flush(&mut self) -> io::Result<()> {
        // Set state to direct
        let state = mem::replace(&mut self.state, State::Direct);
        // Flush buffer if one exists
        match state {
            State::Buffering(buffer) => {
                self.pipe.write_all(&buffer)?;
                self.pipe.flush()
            }
            State::Direct => self.pipe.flush(),
        }
    }
}
