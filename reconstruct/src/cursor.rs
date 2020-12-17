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
//! Cursor over slices of values
//!

use std::mem;
use std::slice::IterMut;

/// A cursor over a slice of values
///
/// This is useful for reading input buffers.
pub struct ReadCursor<'a, T> {
    /// Values remaining to be read
    slice: &'a [T],
    /// Number of values read
    count: usize,
}

impl<'a, T> ReadCursor<'a, T> {
    /// Creates a cursor around a slice
    pub fn new(slice: &'a [T]) -> Self {
        ReadCursor { slice, count: 0 }
    }
    /// Returns true if no more items are available to read
    pub fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }
    /// Returns the number of items that have been read
    /// (through calls to <ReadCursor as Iterator>::next() )
    pub fn read_count(&self) -> usize {
        self.count
    }
}

impl<'a, T> Iterator for ReadCursor<'a, T> {
    type Item = &'a T;

    /// Reads the next item from the slice and increments the read count
    fn next(&mut self) -> Option<Self::Item> {
        match self.slice.first() {
            Some(first) => {
                self.count += 1;
                self.slice = &self.slice[1..];
                Some(first)
            }
            None => None,
        }
    }
}

/// A cursor over a slice of values
///
/// This is useful for writing output buffers.
pub struct WriteCursor<'a, T> {
    /// Iterator over values remaining that can be written
    slice: IterMut<'a, T>,
    /// Number of values written
    count: usize,
}

impl<'a, T> WriteCursor<'a, T> {
    /// Creates a cursor around a slice
    pub fn new(slice: &'a mut [T]) -> Self {
        WriteCursor {
            slice: slice.iter_mut(),
            count: 0,
        }
    }
    /// Returns true if there is no mre space to write items
    pub fn is_empty(&self) -> bool {
        self.slice.len() == 0
    }
    /// Returns a mutable reference to the current item
    ///
    /// To prevent mutable aliasing, each call to this function will return a reference to a
    /// different item in the slice. The advance() function must be called to increment the
    /// write count.
    pub fn current(&mut self) -> Option<&mut T> {
        self.slice.next()
    }
    /// Advances to the next item
    ///
    /// This function will panic if this cursor is empty.
    pub fn advance(&mut self) {
        self.count += 1;
    }

    /// Writes a value to the current position of this cursor, increments the write count,
    /// and returns the value that was replaced
    ///
    /// If this cursor is empty, this function panics.
    pub fn write(&mut self, value: T) -> T {
        let entry = self.slice.next().expect("write() on empty cursor");
        self.count += 1;
        let replaced = mem::replace(entry, value);
        replaced
    }

    /// Returns the number of times advance() has been called
    pub fn write_count(&self) -> usize {
        self.count
    }
}

pub trait ReadCursorExt<'a, T> {
    fn read_cursor(self) -> ReadCursor<'a, T>;
}
impl<'a, T> ReadCursorExt<'a, T> for &'a [T] {
    fn read_cursor(self) -> ReadCursor<'a, T> {
        ReadCursor::new(self)
    }
}
pub trait WriteCursorExt<'a, T> {
    fn write_cursor(self) -> WriteCursor<'a, T>;
}
impl<'a, T> WriteCursorExt<'a, T> for &'a mut [T] {
    fn write_cursor(self) -> WriteCursor<'a, T> {
        WriteCursor::new(self)
    }
}

#[cfg(test)]
mod read_test {
    use super::ReadCursor;

    #[test]
    fn basic_read() {
        let mut cursor = ReadCursor::new(&[1, 2, 3, 4]);
        assert!(!cursor.is_empty());
        assert_eq!(cursor.read_count(), 0);

        assert_eq!(Some(&1), cursor.next());
        assert!(!cursor.is_empty());
        assert_eq!(cursor.read_count(), 1);

        assert_eq!(Some(&2), cursor.next());
        assert!(!cursor.is_empty());
        assert_eq!(cursor.read_count(), 2);

        assert_eq!(Some(&3), cursor.next());
        assert!(!cursor.is_empty());
        assert_eq!(cursor.read_count(), 3);

        assert_eq!(Some(&4), cursor.next());
        assert!(cursor.is_empty());
        assert_eq!(cursor.read_count(), 4);
    }
}

#[cfg(test)]
mod write_test {
    use super::WriteCursor;

    #[test]
    fn basic_write() {
        let mut buffer = [0; 4];
        let mut cursor = WriteCursor::new(&mut buffer);

        assert!(!cursor.is_empty());
        assert_eq!(0, cursor.write_count());

        *cursor.current().unwrap() = 7;
        cursor.advance();
        assert!(!cursor.is_empty());
        assert_eq!(1, cursor.write_count());

        *cursor.current().unwrap() = 8;
        cursor.advance();
        assert!(!cursor.is_empty());
        assert_eq!(2, cursor.write_count());

        cursor.write(9);
        assert!(!cursor.is_empty());
        assert_eq!(3, cursor.write_count());

        *cursor.current().unwrap() = 10;
        cursor.advance();
        assert!(cursor.is_empty());
        assert_eq!(4, cursor.write_count());

        assert_eq!(buffer, [7, 8, 9, 10]);
    }
}
