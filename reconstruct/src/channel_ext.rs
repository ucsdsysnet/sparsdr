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
//! Logging extensions for Crossbeam channels
//!

use std::time::Duration;

use crossbeam_channel::{
    Receiver, RecvTimeoutError, SendError, Sender, TryRecvError, TrySendError,
};

use crate::blocking::{BlockLogger, BlockLogs};

/// A Sender wrapper that records information about blocking
pub struct LoggingSender<T> {
    /// Inner sender
    sender: Sender<T>,
    /// Blocking logger
    logger: BlockLogger,
}

impl<T> LoggingSender<T> {
    pub fn new(sender: Sender<T>) -> Self {
        LoggingSender {
            sender,
            logger: BlockLogger::new(),
        }
    }

    /// Sends a value over the channel, possibly blocking
    ///
    /// This function also records if blocking occurs
    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        match self.sender.try_send(msg) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(msg)) => self.logger.log_blocking(|| self.sender.send(msg)),
            Err(TrySendError::Disconnected(msg)) => Err(SendError(msg)),
        }
    }

    /// Returns the logs recorded by this sender
    pub fn logs(&self) -> BlockLogs {
        self.logger.logs()
    }
}

/// A Receiver wrapper that records information about blocking
pub struct LoggingReceiver<T> {
    /// Inner receiver
    receiver: Receiver<T>,
    /// Blocking logger
    logger: BlockLogger,
}

impl<T> LoggingReceiver<T> {
    /// Creates a logging receiver
    pub fn new(receiver: Receiver<T>) -> Self {
        LoggingReceiver {
            receiver,
            logger: BlockLogger::new(),
        }
    }

    /// Attempts to receive an item, possibly blocking with a timeout
    ///
    /// This function also records if blocking occurs
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        match self.receiver.try_recv() {
            Ok(item) => Ok(item),
            Err(TryRecvError::Empty) => {
                // Block
                self.logger
                    .log_blocking(|| self.receiver.recv_timeout(timeout))
            }
            Err(TryRecvError::Disconnected) => Err(RecvTimeoutError::Disconnected),
        }
    }

    /// Returns the logs recorded by this sender
    pub fn logs(&self) -> BlockLogs {
        self.logger.logs()
    }
}
