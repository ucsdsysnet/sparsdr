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

//! Hann windows

use std::f32::consts::PI;

/// An iterator that generates a Hann window
pub struct HannWindow {
    size: usize,
    next_index: usize,
}

impl HannWindow {
    /// Creates a window generator that produces the specified number of samples
    pub fn new(size: usize) -> Self {
        HannWindow {
            size,
            next_index: 0,
        }
    }
}

impl Iterator for HannWindow {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == self.size {
            None
        } else {
            let value =
                0.5 * (1.0 - f32::cos(2.0 * PI * (self.next_index as f32) / (self.size as f32)));
            self.next_index += 1;
            Some(value)
        }
    }
}
