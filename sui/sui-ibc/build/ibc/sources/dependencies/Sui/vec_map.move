// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module sui::vec_map;

/// This key already exists in the map
const EKeyAlreadyExists: u64 = 0;

/// This key does not exist in the map
const EKeyDoesNotExist: u64 = 1;

/// Trying to destroy a map that is not empty
const EMapNotEmpty: u64 = 2;

/// Trying to access an element of the map at an invalid index
const EIndexOutOfBounds: u64 = 3;

/// Trying to pop from a map that is empty
const EMapEmpty: u64 = 4;

/// Trying to construct a map from keys and values of different lengths
const EUnequalLengths: u64 = 5;

/// A map data structure backed by a vector. The map is guaranteed not to contain duplicate keys, but entries
/// are *not* sorted by key--entries are included in insertion order.
/// All operations are O(N) in the size of the map--the intention of this data structure is only to provide
/// the convenience of programming against a map API.
/// Large maps should use handwritten parent/child relationships instead.
/// Maps that need sorted iteration rather than insertion order iteration should also be handwritten.
public struct VecMap<K: copy, V> has copy, drop, store {
    contents: vector<Entry<K, V>>,
}

/// An entry in the map
public struct Entry<K: copy, V> has copy, drop, store {
    key: K,
    value: V,
}

/// Create an empty `VecMap`
public fun empty<K: copy, V>(): VecMap<K, V> {
    VecMap { contents: vector[] }
}

/// Insert the entry `key` |-> `value` into `self`.
/// Aborts if `key` is already bound in `self`.
public fun insert<K: copy, V>(self: &mut VecMap<K, V>, key: K, value: V) {
    assert!(!self.contains(&key), EKeyAlreadyExists);
    self.contents.push_back(Entry { key, value })
}

/// Remove the entry `key` |-> `value` from self. Aborts if `key` is not bound in `self`.
public fun remove<K: copy, V>(self: &mut VecMap<K, V>, key: &K): (K, V) {
    let idx = self.get_idx(key);
    let Entry { key, value } = self.contents.remove(idx);
    (key, value)
}

/// Pop the most recently inserted entry from the map. Aborts if the map is empty.
public fun pop<K: copy, V>(self: &mut VecMap<K, V>): (K, V) {
    assert!(self.contents.length() != 0, EMapEmpty);
    let Entry { key, value } = self.contents.pop_back();
    (key, value)
}

#[syntax(index)]
/// Get a mutable reference to the value bound to `key` in `self`.
/// Aborts if `key` is not bound in `self`.
public fun get_mut<K: copy, V>(self: &mut VecMap<K, V>, key: &K): &mut V {
    let idx = self.get_idx(key);
    let entry = &mut self.contents[idx];
    &mut entry.value
}

#[syntax(index)]
/// Get a reference to the value bound to `key` in `self`.
/// Aborts if `key` is not bound in `self`.
public fun get<K: copy, V>(self: &VecMap<K, V>, key: &K): &V {
    let idx = self.get_idx(key);
    let entry = &self.contents[idx];
    &entry.value
}

/// Safely try borrow a value bound to `key` in `self`.
/// Return Some(V) if the value exists, None otherwise.
/// Only works for a "copyable" value as references cannot be stored in `vector`.
public fun try_get<K: copy, V: copy>(self: &VecMap<K, V>, key: &K): Option<V> {
    if (self.contains(key)) {
        option::some(*get(self, key))
    } else {
        option::none()
    }
}

/// Return true if `self` contains an entry for `key`, false otherwise
public fun contains<K: copy, V>(self: &VecMap<K, V>, key: &K): bool {
    get_idx_opt(self, key).is_some()
}

/// Return the number of entries in `self`
public fun size<K: copy, V>(self: &VecMap<K, V>): u64 {
    self.contents.length()
}

/// Return true if `self` has 0 elements, false otherwise
public fun is_empty<K: copy, V>(self: &VecMap<K, V>): bool {
    self.size() == 0
}

/// Destroy an empty map. Aborts if `self` is not empty
public fun destroy_empty<K: copy, V>(self: VecMap<K, V>) {
    let VecMap { contents } = self;
    assert!(contents.is_empty(), EMapNotEmpty);
    contents.destroy_empty()
}

/// Unpack `self` into vectors of its keys and values.
/// The output keys and values are stored in insertion order, *not* sorted by key.
public fun into_keys_values<K: copy, V>(self: VecMap<K, V>): (vector<K>, vector<V>) {
    let VecMap { mut contents } = self;
    // reverse the vector so the output keys and values will appear in insertion order
    contents.reverse();
    let mut i = 0;
    let n = contents.length();
    let mut keys = vector[];
    let mut values = vector[];
    while (i < n) {
        let Entry { key, value } = contents.pop_back();
        keys.push_back(key);
        values.push_back(value);
        i = i + 1;
    };
    contents.destroy_empty();
    (keys, values)
}

/// Construct a new `VecMap` from two vectors, one for keys and one for values.
/// The key value pairs are associated via their indices in the vectors, e.g. the key at index i
/// in `keys` is associated with the value at index i in `values`.
/// The key value pairs are stored in insertion order (the original vectors ordering)
/// and are *not* sorted.
public fun from_keys_values<K: copy, V>(mut keys: vector<K>, mut values: vector<V>): VecMap<K, V> {
    assert!(keys.length() == values.length(), EUnequalLengths);
    keys.reverse();
    values.reverse();
    let mut map = empty();
    while (keys.length() != 0) map.insert(keys.pop_back(), values.pop_back());
    keys.destroy_empty();
    values.destroy_empty();
    map
}

/// Returns a list of keys in the map.
/// Do not assume any particular ordering.
public fun keys<K: copy, V>(self: &VecMap<K, V>): vector<K> {
    let mut i = 0;
    let n = self.contents.length();
    let mut keys = vector[];
    while (i < n) {
        let entry = self.contents.borrow(i);
        keys.push_back(entry.key);
        i = i + 1;
    };
    keys
}

/// Find the index of `key` in `self`. Return `None` if `key` is not in `self`.
/// Note that map entries are stored in insertion order, *not* sorted by key.
public fun get_idx_opt<K: copy, V>(self: &VecMap<K, V>, key: &K): Option<u64> {
    let mut i = 0;
    let n = size(self);
    while (i < n) {
        if (&self.contents[i].key == key) {
            return option::some(i)
        };
        i = i + 1;
    };
    option::none()
}

/// Find the index of `key` in `self`. Aborts if `key` is not in `self`.
/// Note that map entries are stored in insertion order, *not* sorted by key.
public fun get_idx<K: copy, V>(self: &VecMap<K, V>, key: &K): u64 {
    let idx_opt = self.get_idx_opt(key);
    assert!(idx_opt.is_some(), EKeyDoesNotExist);
    idx_opt.destroy_some()
}

/// Return a reference to the `idx`th entry of `self`. This gives direct access into the backing array of the map--use with caution.
/// Note that map entries are stored in insertion order, *not* sorted by key.
/// Aborts if `idx` is greater than or equal to `size(self)`
public fun get_entry_by_idx<K: copy, V>(self: &VecMap<K, V>, idx: u64): (&K, &V) {
    assert!(idx < size(self), EIndexOutOfBounds);
    let entry = &self.contents[idx];
    (&entry.key, &entry.value)
}

/// Return a mutable reference to the `idx`th entry of `self`. This gives direct access into the backing array of the map--use with caution.
/// Note that map entries are stored in insertion order, *not* sorted by key.
/// Aborts if `idx` is greater than or equal to `size(self)`
public fun get_entry_by_idx_mut<K: copy, V>(self: &mut VecMap<K, V>, idx: u64): (&K, &mut V) {
    assert!(idx < size(self), EIndexOutOfBounds);
    let entry = &mut self.contents[idx];
    (&entry.key, &mut entry.value)
}

/// Remove the entry at index `idx` from self.
/// Aborts if `idx` is greater than or equal to `size(self)`
public fun remove_entry_by_idx<K: copy, V>(self: &mut VecMap<K, V>, idx: u64): (K, V) {
    assert!(idx < size(self), EIndexOutOfBounds);
    let Entry { key, value } = self.contents.remove(idx);
    (key, value)
}
