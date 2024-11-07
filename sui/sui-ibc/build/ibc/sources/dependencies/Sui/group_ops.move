// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Generic Move and native functions for group operations.
module sui::group_ops;

use sui::bcs;

#[allow(unused_const)]
const ENotSupported: u64 = 0; // Operation is not supported by the network.
const EInvalidInput: u64 = 1;
#[allow(unused_const)]
const EInputTooLong: u64 = 2;
const EInvalidBufferLength: u64 = 3;

/////////////////////////////////////////////////////
////// Generic functions for group operations. //////

// The caller provides a type identifier that should match the types of enum [Groups] in group_ops.rs.

// General wrapper for all group elements.
public struct Element<phantom T> has store, copy, drop {
    bytes: vector<u8>,
}

public fun bytes<G>(e: &Element<G>): &vector<u8> {
    &e.bytes
}

public fun equal<G>(e1: &Element<G>, e2: &Element<G>): bool {
    &e1.bytes == &e2.bytes
}

// Fails if the bytes are not a valid group element and 'is_trusted' is false.
public(package) fun from_bytes<G>(type_: u8, bytes: &vector<u8>, is_trusted: bool): Element<G> {
    assert!(is_trusted || internal_validate(type_, bytes), EInvalidInput);
    Element<G> { bytes: *bytes }
}

public(package) fun add<G>(type_: u8, e1: &Element<G>, e2: &Element<G>): Element<G> {
    Element<G> { bytes: internal_add(type_, &e1.bytes, &e2.bytes) }
}

public(package) fun sub<G>(type_: u8, e1: &Element<G>, e2: &Element<G>): Element<G> {
    Element<G> { bytes: internal_sub(type_, &e1.bytes, &e2.bytes) }
}

public(package) fun mul<S, G>(type_: u8, scalar: &Element<S>, e: &Element<G>): Element<G> {
    Element<G> { bytes: internal_mul(type_, &scalar.bytes, &e.bytes) }
}

/// Fails if scalar = 0. Else returns 1/scalar * e.
public(package) fun div<S, G>(type_: u8, scalar: &Element<S>, e: &Element<G>): Element<G> {
    Element<G> { bytes: internal_div(type_, &scalar.bytes, &e.bytes) }
}

public(package) fun hash_to<G>(type_: u8, m: &vector<u8>): Element<G> {
    Element<G> { bytes: internal_hash_to(type_, m) }
}

/// Aborts with `EInputTooLong` if the vectors are too long.
public(package) fun multi_scalar_multiplication<S, G>(
    type_: u8,
    scalars: &vector<Element<S>>,
    elements: &vector<Element<G>>,
): Element<G> {
    assert!(scalars.length() > 0, EInvalidInput);
    assert!(scalars.length() == elements.length(), EInvalidInput);

    let mut scalars_bytes: vector<u8> = vector[];
    let mut elements_bytes: vector<u8> = vector[];
    let mut i = 0;
    while (i < scalars.length()) {
        let scalar_vec = scalars[i];
        scalars_bytes.append(scalar_vec.bytes);
        let element_vec = elements[i];
        elements_bytes.append(element_vec.bytes);
        i = i + 1;
    };
    Element<G> { bytes: internal_multi_scalar_mul(type_, &scalars_bytes, &elements_bytes) }
}

public(package) fun pairing<G1, G2, G3>(
    type_: u8,
    e1: &Element<G1>,
    e2: &Element<G2>,
): Element<G3> {
    Element<G3> { bytes: internal_pairing(type_, &e1.bytes, &e2.bytes) }
}

public(package) fun convert<From, To>(from_type_: u8, to_type_: u8, e: &Element<From>): Element<To> {
    Element<To> { bytes: internal_convert(from_type_, to_type_, &e.bytes) }
}

public(package) fun sum<G>(type_: u8, terms: &vector<Element<G>>): Element<G> {
    Element<G> { bytes: internal_sum(type_, &(*terms).map!(|x| x.bytes)) }
}

//////////////////////////////
////// Native functions //////

// The following functions do *not* check whether the right types are used (e.g., Risretto255's scalar is used with
// Ristrertto255's G). The caller to the above functions is responsible for that.

// 'type' specifies the type of all elements.
native fun internal_validate(type_: u8, bytes: &vector<u8>): bool;
native fun internal_add(type_: u8, e1: &vector<u8>, e2: &vector<u8>): vector<u8>;
native fun internal_sub(type_: u8, e1: &vector<u8>, e2: &vector<u8>): vector<u8>;

// 'type' represents the type of e2, and the type of e1 is determined automatically from e2. e1 is a scalar
// and e2 is a group/scalar element.
native fun internal_mul(type_: u8, e1: &vector<u8>, e2: &vector<u8>): vector<u8>;
native fun internal_div(type_: u8, e1: &vector<u8>, e2: &vector<u8>): vector<u8>;

native fun internal_hash_to(type_: u8, m: &vector<u8>): vector<u8>;
native fun internal_multi_scalar_mul(
    type_: u8,
    scalars: &vector<u8>,
    elements: &vector<u8>,
): vector<u8>;

// 'type' represents the type of e1, and the rest are determined automatically from e1.
native fun internal_pairing(type_: u8, e1: &vector<u8>, e2: &vector<u8>): vector<u8>;

native fun internal_convert(from_type_: u8, to_type_: u8, e: &vector<u8>): vector<u8>;
native fun internal_sum(type_: u8, e: &vector<vector<u8>>): vector<u8>;

// Helper function for encoding a given u64 number as bytes in a given buffer.
public(package) fun set_as_prefix(x: u64, big_endian: bool, buffer: &mut vector<u8>) {
    let buffer_len = buffer.length();
    assert!(buffer_len > 7, EInvalidBufferLength);
    let x_as_bytes = bcs::to_bytes(&x); // little endian
    let mut i = 0;
    while (i < 8) {
        let position = if (big_endian) {
            buffer_len - i - 1
        } else {
            i
        };
        *(&mut buffer[position]) = x_as_bytes[i];
        i = i + 1;
    };
}
