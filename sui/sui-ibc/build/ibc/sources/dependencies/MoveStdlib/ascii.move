// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// The `ASCII` module defines basic string and char newtypes in Move that verify
/// that characters are valid ASCII, and that strings consist of only valid ASCII characters.
module std::ascii;

// Allows calling `.to_string()` to convert an `ascii::String` into as `string::String`
public use fun std::string::from_ascii as String.to_string;

/// An invalid ASCII character was encountered when creating an ASCII string.
const EInvalidASCIICharacter: u64 = 0x10000;
/// An invalid index was encountered when creating a substring.
const EInvalidIndex: u64 = 0x10001;

/// The `String` struct holds a vector of bytes that all represent
/// valid ASCII characters. Note that these ASCII characters may not all
/// be printable. To determine if a `String` contains only "printable"
/// characters you should use the `all_characters_printable` predicate
/// defined in this module.
public struct String has copy, drop, store {
    bytes: vector<u8>,
}

/// An ASCII character.
public struct Char has copy, drop, store {
    byte: u8,
}

/// Convert a `byte` into a `Char` that is checked to make sure it is valid ASCII.
public fun char(byte: u8): Char {
    assert!(is_valid_char(byte), EInvalidASCIICharacter);
    Char { byte }
}

/// Convert a vector of bytes `bytes` into an `String`. Aborts if
/// `bytes` contains non-ASCII characters.
public fun string(bytes: vector<u8>): String {
    let x = try_string(bytes);
    assert!(x.is_some(), EInvalidASCIICharacter);
    x.destroy_some()
}

/// Convert a vector of bytes `bytes` into an `String`. Returns
/// `Some(<ascii_string>)` if the `bytes` contains all valid ASCII
/// characters. Otherwise returns `None`.
public fun try_string(bytes: vector<u8>): Option<String> {
    let is_valid = bytes.all!(|byte| is_valid_char(*byte));
    if (is_valid) option::some(String { bytes })
    else option::none()
}

/// Returns `true` if all characters in `string` are printable characters
/// Returns `false` otherwise. Not all `String`s are printable strings.
public fun all_characters_printable(string: &String): bool {
    string.bytes.all!(|byte| is_printable_char(*byte))
}

/// Push a `Char` to the end of the `string`.
public fun push_char(string: &mut String, char: Char) {
    string.bytes.push_back(char.byte);
}

/// Pop a `Char` from the end of the `string`.
public fun pop_char(string: &mut String): Char {
    Char { byte: string.bytes.pop_back() }
}

/// Returns the length of the `string` in bytes.
public fun length(string: &String): u64 {
    string.as_bytes().length()
}

/// Append the `other` string to the end of `string`.
public fun append(string: &mut String, other: String) {
    string.bytes.append(other.into_bytes())
}

/// Insert the `other` string at the `at` index of `string`.
public fun insert(s: &mut String, at: u64, o: String) {
    assert!(at <= s.length(), EInvalidIndex);
    o.into_bytes().destroy!(|e| s.bytes.insert(e, at));
}

/// Copy the slice of the `string` from `i` to `j` into a new `String`.
public fun substring(string: &String, i: u64, j: u64): String {
    assert!(i <= j && j <= string.length(), EInvalidIndex);
    let mut bytes = vector[];
    i.range_do!(j, |i| bytes.push_back(string.bytes[i]));
    String { bytes }
}

/// Get the inner bytes of the `string` as a reference
public fun as_bytes(string: &String): &vector<u8> {
    &string.bytes
}

/// Unpack the `string` to get its backing bytes
public fun into_bytes(string: String): vector<u8> {
    let String { bytes } = string;
    bytes
}

/// Unpack the `char` into its underlying bytes.
public fun byte(char: Char): u8 {
    let Char { byte } = char;
    byte
}

/// Returns `true` if `b` is a valid ASCII character.
/// Returns `false` otherwise.
public fun is_valid_char(b: u8): bool {
    b <= 0x7F
}

/// Returns `true` if `byte` is an printable ASCII character.
/// Returns `false` otherwise.
public fun is_printable_char(byte: u8): bool {
    byte >= 0x20 && // Disallow metacharacters
        byte <= 0x7E // Don't allow DEL metacharacter
}

/// Returns `true` if `string` is empty.
public fun is_empty(string: &String): bool {
    string.bytes.is_empty()
}

/// Convert a `string` to its uppercase equivalent.
public fun to_uppercase(string: &String): String {
    let bytes = string.as_bytes().map_ref!(|byte| char_to_uppercase(*byte));
    String { bytes }
}

/// Convert a `string` to its lowercase equivalent.
public fun to_lowercase(string: &String): String {
    let bytes = string.as_bytes().map_ref!(|byte| char_to_lowercase(*byte));
    String { bytes }
}

/// Computes the index of the first occurrence of the `substr` in the `string`.
/// Returns the length of the `string` if the `substr` is not found.
/// Returns 0 if the `substr` is empty.
public fun index_of(string: &String, substr: &String): u64 {
    let mut i = 0;
    let (n, m) = (string.length(), substr.length());
    if (n < m) return n;
    while (i <= n - m) {
        let mut j = 0;
        while (j < m && string.bytes[i + j] == substr.bytes[j]) j = j + 1;
        if (j == m) return i;
        i = i + 1;
    };
    n
}

/// Convert a `char` to its lowercase equivalent.
fun char_to_uppercase(byte: u8): u8 {
    if (byte >= 0x61 && byte <= 0x7A) byte - 0x20
    else byte
}

/// Convert a `char` to its lowercase equivalent.
fun char_to_lowercase(byte: u8): u8 {
    if (byte >= 0x41 && byte <= 0x5A) byte + 0x20
    else byte
}
