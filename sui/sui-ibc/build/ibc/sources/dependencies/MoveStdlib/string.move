// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// The `string` module defines the `String` type which represents UTF8 encoded
/// strings.
module std::string;

use std::ascii;

/// An invalid UTF8 encoding.
const EInvalidUTF8: u64 = 1;

/// Index out of range.
const EInvalidIndex: u64 = 2;

/// A `String` holds a sequence of bytes which is guaranteed to be in utf8
/// format.
public struct String has copy, drop, store {
    bytes: vector<u8>,
}

/// Creates a new string from a sequence of bytes. Aborts if the bytes do
/// not represent valid utf8.
public fun utf8(bytes: vector<u8>): String {
    assert!(internal_check_utf8(&bytes), EInvalidUTF8);
    String { bytes }
}

/// Convert an ASCII string to a UTF8 string
public fun from_ascii(s: ascii::String): String {
    String { bytes: s.into_bytes() }
}

/// Convert an UTF8 string to an ASCII string.
/// Aborts if `s` is not valid ASCII
public fun to_ascii(s: String): ascii::String {
    let String { bytes } = s;
    bytes.to_ascii_string()
}

/// Tries to create a new string from a sequence of bytes.
public fun try_utf8(bytes: vector<u8>): Option<String> {
    if (internal_check_utf8(&bytes)) option::some(String { bytes })
    else option::none()
}

/// Returns a reference to the underlying byte vector.
public fun as_bytes(s: &String): &vector<u8> {
    &s.bytes
}

/// Unpack the `string` to get its underlying bytes.
public fun into_bytes(s: String): vector<u8> {
    let String { bytes } = s;
    bytes
}

/// Checks whether this string is empty.
public fun is_empty(s: &String): bool {
    s.bytes.is_empty()
}

/// Returns the length of this string, in bytes.
public fun length(s: &String): u64 {
    s.bytes.length()
}

/// Appends a string.
public fun append(s: &mut String, r: String) {
    s.bytes.append(r.bytes)
}

/// Appends bytes which must be in valid utf8 format.
public fun append_utf8(s: &mut String, bytes: vector<u8>) {
    s.append(utf8(bytes))
}

/// Insert the other string at the byte index in given string. The index
/// must be at a valid utf8 char boundary.
public fun insert(s: &mut String, at: u64, o: String) {
    let bytes = &s.bytes;
    assert!(at <= bytes.length() && internal_is_char_boundary(bytes, at), EInvalidIndex);
    let l = s.length();
    let mut front = s.substring(0, at);
    let end = s.substring(at, l);
    front.append(o);
    front.append(end);
    *s = front;
}

/// Returns a sub-string using the given byte indices, where `i` is the first
/// byte position and `j` is the start of the first byte not included (or the
/// length of the string). The indices must be at valid utf8 char boundaries,
/// guaranteeing that the result is valid utf8.
public fun substring(s: &String, i: u64, j: u64): String {
    let bytes = &s.bytes;
    let l = bytes.length();
    assert!(
        j <= l &&
            i <= j &&
            internal_is_char_boundary(bytes, i) &&
            internal_is_char_boundary(bytes, j),
        EInvalidIndex,
    );
    String { bytes: internal_sub_string(bytes, i, j) }
}

/// Computes the index of the first occurrence of a string. Returns `s.length()`
/// if no occurrence found.
public fun index_of(s: &String, r: &String): u64 {
    internal_index_of(&s.bytes, &r.bytes)
}

// Native API

native fun internal_check_utf8(v: &vector<u8>): bool;
native fun internal_is_char_boundary(v: &vector<u8>, i: u64): bool;
native fun internal_sub_string(v: &vector<u8>, i: u64, j: u64): vector<u8>;
native fun internal_index_of(v: &vector<u8>, r: &vector<u8>): u64;

// === Deprecated ===

#[deprecated(note = b"Use `std::string::as_bytes` instead.")]
public fun bytes(s: &String): &vector<u8> { s.as_bytes() }

#[deprecated(note = b"Use `std::string::substring` instead.")]
public fun sub_string(s: &String, i: u64, j: u64): String {
    s.substring(i, j)
}
