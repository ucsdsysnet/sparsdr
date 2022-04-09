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

//! Bin range data

pub mod choice;

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::ops::Range;
use std::str::FromStr;

/// A range of bins
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinRange(Range<u16>);

impl BinRange {
    /// Returns the start of this range
    pub fn start(&self) -> u16 {
        self.0.start
    }
    /// Returns the end of this range
    pub fn end(&self) -> u16 {
        self.0.end
    }
    /// Returns the middle of this range
    pub fn middle(&self) -> u16 {
        (self.0.start + self.0.end) / 2
    }
    /// Returns the number of bins in this range
    pub fn size(&self) -> u16 {
        self.0.end - self.0.start
    }
    /// Returns true if the specified index is in this range
    pub fn contains(&self, index: u16) -> bool {
        index >= self.0.start && index < self.0.end
    }
    /// Returns a Range<usize> equivalent to this bin range
    pub fn as_usize_range(&self) -> Range<usize> {
        usize::from(self.0.start)..usize::from(self.0.end)
    }
}

impl PartialOrd for BinRange {
    fn partial_cmp(&self, other: &BinRange) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BinRange {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .start
            .cmp(&other.0.start)
            .then(self.0.end.cmp(&other.0.end))
    }
}

impl fmt::Display for BinRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.0.start, self.0.end)
    }
}

/// Parses a bin range from a string
///
/// Expected format: number .. number
///
/// The first number is inclusive, and the second is exclusive.
///
impl FromStr for BinRange {
    type Err = BinRangeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Look for the first number
        let chars = s.chars();
        let start_digits = chars
            .take_while(|c| char::is_numeric(*c))
            .collect::<String>();
        let start = u16::from_str(&start_digits)?;
        // The first use of chars consumed the first character after the end of the start number.
        // Create a new iterator to get back to that character
        let mut chars = s.chars().skip(start_digits.chars().count());
        // Expect two periods
        for _ in 0..2 {
            let next = chars.next();
            if next != Some('.') {
                return Err(BinRangeParseError::Format);
            }
        }
        // Look for the second number
        let end_digits = chars
            .take_while(|c| char::is_numeric(*c))
            .collect::<String>();
        let end = u16::from_str(&end_digits)?;

        // Check start and end
        if end < start {
            return Err(BinRangeParseError::Range);
        }

        Ok(BinRange(start..end))
    }
}

/// Errors that may occur when parsing bin ranges
#[derive(Debug)]
pub enum BinRangeParseError {
    /// The format was not valid
    Format,
    /// The end was not at least as large as the start
    Range,
    /// A number part of the range could not be parsed
    Number(ParseIntError),
}

impl fmt::Display for BinRangeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BinRangeParseError::Format => {
                write!(f, "Invalid bin range format, expected number..number")
            }
            BinRangeParseError::Range => write!(f, "Range end is not at least as large as start"),
            BinRangeParseError::Number(ref inner) => write!(f, "Invalid number format: {}", inner),
        }
    }
}

impl Error for BinRangeParseError {
    fn description(&self) -> &str {
        "bin range parse error"
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            BinRangeParseError::Format | BinRangeParseError::Range => None,
            BinRangeParseError::Number(ref inner) => Some(inner),
        }
    }
}

impl From<ParseIntError> for BinRangeParseError {
    fn from(inner: ParseIntError) -> Self {
        BinRangeParseError::Number(inner)
    }
}

impl From<Range<u16>> for BinRange {
    fn from(range: Range<u16>) -> Self {
        BinRange(range)
    }
}

impl From<BinRange> for Range<u16> {
    fn from(range: BinRange) -> Self {
        range.0
    }
}

#[cfg(test)]
mod test {
    use super::BinRange;
    use std::str::FromStr;

    #[test]
    fn test_empty() {
        assert!(BinRange::from_str("").is_err());
    }
    #[test]
    fn test_starting_only() {
        assert!(BinRange::from_str("33").is_err());
    }
    #[test]
    fn test_starting_and_periods_only() {
        assert!(BinRange::from_str("33..").is_err());
    }
    #[test]
    fn test_numbers_too_large() {
        assert!(BinRange::from_str("33..65536").is_err());
    }
    #[test]
    fn test_incorrect_order() {
        assert!(BinRange::from_str("33..32").is_err());
    }
    #[test]
    fn test_limits() {
        let parsed = BinRange::from_str("0..65535").unwrap();
        let expected = BinRange::from(0..65535);
        assert_eq!(parsed, expected);
    }
    #[test]
    fn test_empty_range() {
        let parsed = BinRange::from_str("0..0").unwrap();
        let expected = BinRange::from(0..0);
        assert_eq!(parsed, expected);
    }
}
