/*
 * Copyright 2019-2022 The Regents of the University of California
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

//! The overlap step

use crate::steps::overlap::overlap_flush::OverlapFlush;
use crate::steps::overlap::overlap_gaps::OverlapGaps;
use crate::window::Status;
use crate::window::TimeWindow;

mod overlap_flush;
mod overlap_gaps;

/// An overlap step that may either insert gaps or support flushing
pub struct Overlap<I> {
    implementation: OverlapImpl<I>,
}

impl<I> Overlap<I>
where
    I: Iterator<Item = Status<TimeWindow>>,
{
    /// Creates a new overlap step that puts zero samples in the gaps between periods with active
    /// signals
    pub fn new_gaps(inner: I, window_size: usize) -> Self {
        Overlap {
            implementation: OverlapImpl::Gaps(OverlapGaps::new(inner, window_size)),
        }
    }

    /// Creates a new overlap step that produces flushed windows
    pub fn new_flush(inner: I, window_size: usize) -> Self {
        Overlap {
            implementation: OverlapImpl::Flush(OverlapFlush::new(inner, window_size)),
        }
    }
}

enum OverlapImpl<I> {
    Gaps(OverlapGaps<I>),
    Flush(OverlapFlush<I>),
}

impl<I> Iterator for Overlap<I>
where
    I: Iterator<Item = Status<TimeWindow>>,
{
    type Item = FlushWindow;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.implementation {
            OverlapImpl::Gaps(inner) => inner.next(),
            OverlapImpl::Flush(inner) => inner.next(),
        }
    }
}

/// A time window that may have been flushed due to a timeout status
#[derive(Debug, Clone)]
pub struct FlushWindow {
    /// The window of samples
    pub window: TimeWindow,
    /// True if this window was flushed due to a timeout
    pub flushed: bool,
}
impl FlushWindow {
    fn not_flushed(window: TimeWindow) -> Self {
        FlushWindow {
            window,
            flushed: false,
        }
    }
    fn flushed(window: TimeWindow) -> Self {
        FlushWindow {
            window,
            flushed: true,
        }
    }
}

/// A mode for the overlap steps
#[derive(Debug, Clone)]
pub enum OverlapMode {
    /// Put zero samples in gaps
    Gaps,
    /// Flush samples (the enclosed value is the number of samples to insert)
    Flush(u32),
}
