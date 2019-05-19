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
//! Recording of blocking operations
//!

use std::cell::Cell;
use std::time::{Duration, Instant};

/// Logs time spent blocked
#[derive(Debug, Default)]
pub struct BlockLogger {
    /// Number of times blocking has happened
    block_count: Cell<u64>,
    /// Total time spent blocking
    block_duration: Cell<Duration>,
}

impl BlockLogger {
    /// Creates a new default logger with zero block count and zero block duration
    pub fn new() -> Self {
        Self::default()
    }

    /// Logs a blocking operation, incrementing the block count and duration
    pub fn log_blocking<F, R>(&self, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start_time = Instant::now();
        let result = operation();
        let block_duration = Instant::now().duration_since(start_time);
        self.block_duration
            .set(self.block_duration.get() + block_duration);
        self.block_count
            .set(self.block_count.get().saturating_add(1));
        result
    }

    /// Returns the logs recorded by this logger
    pub fn logs(&self) -> BlockLogs {
        BlockLogs {
            block_count: self.block_count.get(),
            block_duration: self.block_duration.get(),
        }
    }
}

/// Logs recorded by a LoggingSender
#[derive(Debug)]
pub struct BlockLogs {
    /// Number of times blocking occurred
    block_count: u64,
    /// Total time spent blocked
    block_duration: Duration,
}

impl BlockLogs {
    /// Returns the number of times blocking occurred
    pub fn block_count(&self) -> u64 {
        self.block_count
    }
    /// Returns the total time spent blocked
    pub fn block_duration(&self) -> Duration {
        self.block_duration
    }
}
