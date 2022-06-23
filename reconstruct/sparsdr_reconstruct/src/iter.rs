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

#![allow(missing_docs)]

use std::ops::ControlFlow;

/// An iterator-like thing that can be used by pushing values in at the beginning, instead of
/// polling for values at the end
///
/// This is similar to `std::iter::Iterator`, but in reverse. Chains of operations can be built up
/// from the end to the beginning.
///
pub trait PushIterator<T> {
    type Error;

    /// Handles an incoming item
    ///
    /// This function should return `ControlFlow::Continue(())` if the item was successfully
    /// processed, or `ControlFlow::Break` with an error if an error occurs.
    ///
    /// After this function returns `ControlFlow::Break`, it should not be called again.
    fn push(&mut self, item: T) -> ControlFlow<Self::Error>;

    /// Flushes any items that may have been delayed out to the end of the chain
    fn flush(&mut self) -> Result<(), Self::Error>;

    // General-purpose provided methods

    /// Creates an iterator adapter that applies a function to each item
    fn map<F>(self, operation: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map {
            inner: self,
            operation,
        }
    }

    /// Creates an iterator adapter that calls a function with a reference to each item
    fn inspect<F>(self, operation: F) -> Inspect<Self, F>
    where
        Self: Sized,
    {
        Inspect {
            inner: self,
            operation,
        }
    }
}

/// Applies a function to each item
pub struct Map<I, F> {
    inner: I,
    operation: F,
}

impl<I, F, T, U> PushIterator<T> for Map<I, F>
where
    I: PushIterator<U>,
    F: FnMut(T) -> U,
{
    type Error = I::Error;

    fn push(&mut self, item: T) -> ControlFlow<Self::Error> {
        let converted: U = (self.operation)(item);
        self.inner.push(converted)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.inner.flush()
    }
}

/// Calls a function with a reference to each item
pub struct Inspect<I, F> {
    inner: I,
    operation: F,
}

impl<I, F, T> PushIterator<T> for Inspect<I, F>
where
    I: PushIterator<T>,
    F: FnMut(&T),
{
    type Error = I::Error;

    fn push(&mut self, item: T) -> ControlFlow<Self::Error> {
        (self.operation)(&item);
        self.inner.push(item)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.inner.flush()
    }
}
