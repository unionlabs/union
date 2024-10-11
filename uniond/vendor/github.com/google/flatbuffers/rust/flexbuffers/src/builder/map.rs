// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{Builder, Pushable, Value, VectorBuilder};

/// Builds a Flexbuffer map, returned by a [Builder](struct.Builder.html).
///
/// ## Side effect when dropped:
/// When this is dropped, or `end_map` is called, the map is
/// commited to the buffer. If this map is the root of the flexbuffer, then the
/// root is written and the flexbuffer is complete.
/// ## Panics:
/// -  Duplicate keys will result in a panic in both debug and release mode.
/// -  Keys with internal nulls results in a panic in debug mode and result in silent truncaction
///    in release mode.
pub struct MapBuilder<'a> {
    pub(super) builder: &'a mut Builder,
    // If the root is this map then start == None. Otherwise start is the
    // number of values in the 'values stack' before adding this map.
    pub(super) start: Option<usize>,
}
impl<'a> MapBuilder<'a> {
    /// Push `p` onto this map with key `key`.
    /// This will panic (in debug mode) if `key` contains internal nulls.
    #[inline]
    pub fn push<P: Pushable>(&mut self, key: &str, p: P) {
        self.builder.push_key(key);
        self.builder.push(p);
    }
    /// Starts a nested vector that will be pushed onto this map
    /// with key `key` when it is dropped.
    ///
    /// This will panic (in debug mode) if `key` contains internal nulls.
    #[inline]
    pub fn start_vector(&mut self, key: &str) -> VectorBuilder {
        // Push the key that refers to this nested vector.
        self.builder.push_key(key);
        // Nested vector.
        let start = Some(self.builder.values.len());
        VectorBuilder {
            builder: self.builder,
            start,
        }
    }
    /// Starts a nested map which that will be pushed onto this map
    /// with key `key` when it is dropped.
    ///
    /// This will panic (in debug mode) if `key` contains internal nulls.
    #[inline]
    pub fn start_map(&mut self, key: &str) -> MapBuilder {
        // Push the key that refers to this nested vector.
        self.builder.push_key(key);
        // Nested map.
        let start = Some(self.builder.values.len());
        MapBuilder {
            builder: self.builder,
            start,
        }
    }
    /// `end_map` sorts the map by key and writes it to the buffer. This happens anyway
    /// when the map builder is dropped.
    #[inline]
    pub fn end_map(self) {}
}
impl<'a> Drop for MapBuilder<'a> {
    #[inline]
    fn drop(&mut self) {
        self.builder.end_map_or_vector(true, self.start);
    }
}

// Read known keys / strings as iterators over bytes -- skipping utf8 validation and strlen.
pub(super) fn get_key(buffer: &[u8], address: usize) -> impl Iterator<Item = &u8> {
    buffer[address..].iter().take_while(|&&b| b != b'\0')
}

// `values` is assumed to be of the format [key1, value1, ..., keyN, valueN].
// The keys refer to cstrings in `buffer`. When this function returns,
// `values` is sorted in place by key.
pub(super) fn sort_map_by_keys(values: &mut [Value], buffer: &[u8]) {
    debug_assert_eq!(values.len() % 2, 0);
    debug_assert!(values.iter().step_by(2).all(Value::is_key));
    let raw_pairs = values.as_mut_ptr() as *mut [Value; 2];
    let pairs_len = values.len() / 2;
    // Unsafe code needed to treat the slice as key-value pairs when sorting in place. This is
    // preferred over custom sorting or adding another dependency. By construction, this part
    // of the values stack must be alternating (key, value) pairs. The public API must not be
    // able to trigger the above debug_assets that protect this unsafe usage.
    let pairs: &mut [[Value; 2]] = unsafe { std::slice::from_raw_parts_mut(raw_pairs, pairs_len) };
    #[rustfmt::skip]
    pairs.sort_unstable_by(|[key1, _], [key2, _]| {
        if let Value::Key(a1) = *key1 {
            if let Value::Key(a2) = *key2 {
                let s1 = get_key(buffer, a1);
                let s2 = get_key(buffer, a2);
                let ord = s1.cmp(s2);
                if ord == std::cmp::Ordering::Equal {
                    let dup: String = get_key(buffer, a1).map(|&b| b as char).collect();
                    panic!("Duplicated key in map {:?}", dup);
                }
                return ord;
            }
        }
        unreachable!();
    });
}
