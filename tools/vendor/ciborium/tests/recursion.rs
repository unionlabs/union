// SPDX-License-Identifier: Apache-2.0

//! This test validates that we don't get stack overflows.
//!
//! If container types cause recursion, then a long list of prefixes which
//! indicate nested container types could cause the stack to overflow. We
//! test each of these types here to ensure there is no stack overflow.

use ciborium::{
    de::{from_reader, Error},
    value::Value,
};

#[test]
fn array() {
    let bytes = [0x9f; 128 * 1024];
    match from_reader::<Value, _>(&bytes[..]).unwrap_err() {
        Error::RecursionLimitExceeded => (),
        e => panic!("incorrect error: {:?}", e),
    }
}

#[test]
fn map() {
    let bytes = [0xbf; 128 * 1024];
    match from_reader::<Value, _>(&bytes[..]).unwrap_err() {
        Error::RecursionLimitExceeded => (),
        e => panic!("incorrect error: {:?}", e),
    }
}

#[test]
fn bytes() {
    let bytes = [0x5f; 128 * 1024];
    match from_reader::<Value, _>(&bytes[..]).unwrap_err() {
        Error::Io(..) => (),
        e => panic!("incorrect error: {:?}", e),
    }
}

#[test]
fn text() {
    let bytes = [0x7f; 128 * 1024];
    match from_reader::<Value, _>(&bytes[..]).unwrap_err() {
        Error::Io(..) => (),
        e => panic!("incorrect error: {:?}", e),
    }
}
