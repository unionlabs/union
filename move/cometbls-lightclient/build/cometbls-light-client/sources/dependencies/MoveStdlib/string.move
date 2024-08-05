/// The `string` module defines the `String` type which represents UTF8 encoded strings.
module std::string {
    use std::vector;
    use std::option::{Self, Option};

    /// An invalid UTF8 encoding.
    const EINVALID_UTF8: u64 = 1;

    /// Index out of range.
    const EINVALID_INDEX: u64 = 2;

    /// A `String` holds a sequence of bytes which is guaranteed to be in utf8 format.
    struct String has copy, drop, store {
        bytes: vector<u8>,
    }

    /// Creates a new string from a sequence of bytes. Aborts if the bytes do not represent valid utf8.
    public fun utf8(bytes: vector<u8>): String {
        assert!(internal_check_utf8(&bytes), EINVALID_UTF8);
        String{bytes}
    }

    /// Tries to create a new string from a sequence of bytes.
    public fun try_utf8(bytes: vector<u8>): Option<String> {
        if (internal_check_utf8(&bytes)) {
            option::some(String{bytes})
        } else {
            option::none()
        }
    }

    /// Returns a reference to the underlying byte vector.
    public fun bytes(s: &String): &vector<u8> {
        &s.bytes
    }

    /// Checks whether this string is empty.
    public fun is_empty(s: &String): bool {
        vector::is_empty(&s.bytes)
    }

    /// Returns the length of this string, in bytes.
    public fun length(s: &String): u64 {
        vector::length(&s.bytes)
    }

    /// Appends a string.
    public fun append(s: &mut String, r: String) {
        vector::append(&mut s.bytes, r.bytes)
    }

    /// Appends bytes which must be in valid utf8 format.
    public fun append_utf8(s: &mut String, bytes: vector<u8>) {
        append(s, utf8(bytes))
    }

    /// Insert the other string at the byte index in given string. The index must be at a valid utf8 char
    /// boundary.
    public fun insert(s: &mut String, at: u64, o: String) {
        let bytes = &s.bytes;
        assert!(at <= vector::length(bytes) && internal_is_char_boundary(bytes, at), EINVALID_INDEX);
        let l = length(s);
        let front = sub_string(s, 0, at);
        let end = sub_string(s, at, l);
        append(&mut front, o);
        append(&mut front, end);
        *s = front;
    }

    /// Returns a sub-string using the given byte indices, where `i` is the first byte position and `j` is the start
    /// of the first byte not included (or the length of the string). The indices must be at valid utf8 char boundaries,
    /// guaranteeing that the result is valid utf8.
    public fun sub_string(s: &String, i: u64, j: u64): String {
        let bytes = &s.bytes;
        let l = vector::length(bytes);
        assert!(
            j <= l && i <= j && internal_is_char_boundary(bytes, i) && internal_is_char_boundary(bytes, j),
            EINVALID_INDEX
        );
        String { bytes: internal_sub_string(bytes, i, j) }
    }

    /// Computes the index of the first occurrence of a string. Returns `length(s)` if no occurrence found.
    public fun index_of(s: &String, r: &String): u64 {
        internal_index_of(&s.bytes, &r.bytes)
    }

    // Native API
    public native fun internal_check_utf8(v: &vector<u8>): bool;
    native fun internal_is_char_boundary(v: &vector<u8>, i: u64): bool;
    native fun internal_sub_string(v: &vector<u8>, i: u64, j: u64): vector<u8>;
    native fun internal_index_of(v: &vector<u8>, r: &vector<u8>): u64;
}
