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
//! Custom deserialization functions
//!

use serde::de::{DeserializeSeed, Error, MapAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt::Formatter;

/// Deserializes a Vec<T>, but returns an error if the returned Vec would be empty
pub fn deserialize_non_empty_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let values = Vec::<T>::deserialize(deserializer)?;
    if values.is_empty() {
        Err(D::Error::invalid_length(0, &"at least one value"))
    } else {
        Ok(values)
    }
}

/// Deserializes a BTreeMap<String, String> with values that may be strings, integers,
/// floating-point numbers, or booleans. All values will be converted to strings.
pub fn permissive_deserialize_string_map<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ToStringVisitor;

    impl<'de> Visitor<'de> for ToStringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a string, boolean, integer, or floating-point number")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.to_string())
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.to_string())
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.to_string())
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.to_string())
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v.to_owned())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(v)
        }
    }
    struct ToStringSeed;
    impl<'de> DeserializeSeed<'de> for ToStringSeed {
        type Value = String;

        fn deserialize<D>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, <D as Deserializer<'de>>::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(ToStringVisitor)
        }
    }

    struct MapVisitor;
    impl<'de> Visitor<'de> for MapVisitor {
        type Value = BTreeMap<String, String>;

        fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            formatter
                .write_str("a map with string keys and values that can be formatted as strings")
        }

        fn visit_map<A>(
            self,
            mut map_access: A,
        ) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
        where
            A: MapAccess<'de>,
        {
            let mut map = BTreeMap::new();
            while let Some(key) = map_access.next_key::<String>()? {
                let value = map_access.next_value_seed(ToStringSeed)?;
                match map.entry(key) {
                    Entry::Vacant(vacant) => {
                        vacant.insert(value);
                    }
                    Entry::Occupied(occupied) => {
                        return Err(<A as MapAccess<'de>>::Error::custom(format!(
                            "multiple values for key {}",
                            occupied.key()
                        )))
                    }
                }
            }
            Ok(map)
        }
    }
    deserializer.deserialize_map(MapVisitor)
}

/// Deserializes an f32 in the range `[0, 1]`
pub fn deserialize_0_1<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    struct Float01Visitor;
    impl<'de1> Visitor<'de1> for Float01Visitor {
        type Value = f32;

        fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
            write!(formatter, "a floating-point number in the range [0, 1]")
        }

        fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where
            E: Error,
        {
            if v >= 0.0 && v <= 1.0 {
                Ok(v)
            } else {
                Err(Error::invalid_value(Unexpected::Float(f64::from(v)), &self))
            }
        }
    }
    deserializer.deserialize_f32(Float01Visitor)
}
