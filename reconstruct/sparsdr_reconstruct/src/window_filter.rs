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

//! Things that can filter frequency-domain windows to determine if they should be reconstructed

pub mod correlation_template;

use crate::window::{Fft, Window};

/// A filter for frequency-domain windows
pub trait WindowFilter {
    /// Returns true if this window (and all consecutive following windows) should be
    /// reconstructed and decoded
    fn accept(&mut self, window: &Window<Fft>) -> bool;
}
