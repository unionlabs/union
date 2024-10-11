// Copyright 2020 Google LLC
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

use alloc::vec::Vec;

use flexbuffers::*;
#[cfg(not(miri))]  // slow.
use quickcheck::QuickCheck;

#[test]
#[cfg(not(miri))]  // slow.
fn qc_reader_no_crash() {
    fn no_crash(xs: Vec<u8>) -> bool {
        let r = Reader::get_root(xs.as_ref());
        r.is_err() || r.is_ok()
    }
    QuickCheck::new()
        .min_tests_passed(10_000_000)
        .quicktest(no_crash as fn(Vec<u8>) -> bool)
        .unwrap();

    no_crash(vec![0, 10 << 2 | 2, 0]);
}
#[test]
fn as_num() {
    let mut fxb = Builder::default();
    let mut m = fxb.start_map();
    m.push("a", &[-1i8, -2, -3, -4]);
    m.push("b", 250i64);
    m.push("c", 5000u16);
    m.end_map();

    let r = Reader::get_root(fxb.view()).unwrap();
    assert_eq!(r.as_i8(), 3); // length.
    assert_eq!(r.as_i16(), 3);
    assert_eq!(r.as_i32(), 3);
    assert_eq!(r.as_i64(), 3);
    assert_eq!(r.as_u8(), 3);
    assert_eq!(r.as_u16(), 3);
    assert_eq!(r.as_u32(), 3);
    assert_eq!(r.as_u64(), 3);
    assert_eq!(r.as_f32(), 3.0);
    assert_eq!(r.as_f64(), 3.0);

    let m = r.as_map();
    let a = m.index("a").unwrap();
    assert_eq!(a.as_f32(), 4.0); // length.
    assert_eq!(a.as_f64(), 4.0); // length.
    assert_eq!(a.as_vector().idx(0).as_i8(), -1);
    assert_eq!(a.as_vector().idx(1).as_i16(), -2);
    assert_eq!(a.as_vector().idx(2).as_i32(), -3);
    assert_eq!(a.as_vector().idx(3).as_i64(), -4);

    let b = m.index("b").unwrap();
    assert_eq!(b.as_u8(), 250);
    assert_eq!(b.as_u16(), 250);
    assert_eq!(b.as_u32(), 250);
    assert_eq!(b.as_u64(), 250);
    assert_eq!(b.as_i8(), 0); // overflow
    assert_eq!(b.as_i16(), 250);
    assert_eq!(b.as_i32(), 250);
    assert_eq!(b.as_i64(), 250);

    let c = m.index("c").unwrap();
    assert_eq!(c.as_i64(), 5000);
    assert_eq!(c.as_u64(), 5000);
    assert_eq!(c.as_f32(), 5000.0);
    assert_eq!(c.as_u8(), 0); // overflow
    assert_eq!(c.as_u16(), 5000);
    assert_eq!(c.as_u32(), 5000);
    assert_eq!(c.as_u64(), 5000);
    assert_eq!(c.as_i8(), 0); // overflow
    assert_eq!(c.as_i16(), 5000);
    assert_eq!(c.as_i32(), 5000);
    assert_eq!(c.as_i64(), 5000);
}
#[test]
fn string_as_num() {
    let mut fxb = Builder::default();
    let mut v = fxb.start_vector();
    v.push("3.1415");
    v.push("9.001e3");
    v.push("42");
    v.end_vector();
    let r = Reader::get_root(fxb.view()).unwrap();

    let v0 = r.as_vector().idx(0);
    assert_eq!(v0.as_f64(), 3.1415);
    assert_eq!(v0.as_f32(), 3.1415);
    assert_eq!(v0.as_u8(), 0);
    assert_eq!(v0.as_u16(), 0);
    assert_eq!(v0.as_u32(), 0);
    assert_eq!(v0.as_u64(), 0);
    assert_eq!(v0.as_i8(), 0);
    assert_eq!(v0.as_i16(), 0);
    assert_eq!(v0.as_i32(), 0);
    assert_eq!(v0.as_i64(), 0);

    let v1 = r.as_vector().idx(1);
    assert_eq!(v1.as_f64(), 9001.0);
    assert_eq!(v1.as_f32(), 9001.0);
    assert_eq!(v1.as_u8(), 0);
    assert_eq!(v1.as_u16(), 0);
    assert_eq!(v1.as_u32(), 0);
    assert_eq!(v1.as_u64(), 0);
    assert_eq!(v1.as_i8(), 0);
    assert_eq!(v1.as_i16(), 0);
    assert_eq!(v1.as_i32(), 0);
    assert_eq!(v1.as_i64(), 0);
    assert_eq!(v1.as_i32(), 0);

    let v2 = r.as_vector().idx(2);
    assert_eq!(v2.as_f64(), 42.0);
    assert_eq!(v2.as_f32(), 42.0);
    assert_eq!(v2.as_u8(), 42);
    assert_eq!(v2.as_u16(), 42);
    assert_eq!(v2.as_u32(), 42);
    assert_eq!(v2.as_u64(), 42);
    assert_eq!(v2.as_i8(), 42);
    assert_eq!(v2.as_i16(), 42);
    assert_eq!(v2.as_i32(), 42);
    assert_eq!(v2.as_i64(), 42);
    assert_eq!(v2.as_i32(), 42);
}
#[test]
fn null_reader() {
    let n = Reader::<&[u8]>::default();
    assert_eq!(n.as_i8(), 0);
    assert_eq!(n.as_i16(), 0);
    assert_eq!(n.as_i32(), 0);
    assert_eq!(n.as_i64(), 0);
    assert_eq!(n.as_u8(), 0);
    assert_eq!(n.as_u16(), 0);
    assert_eq!(n.as_u32(), 0);
    assert_eq!(n.as_u64(), 0);
    assert_eq!(n.as_f32(), 0.0);
    assert_eq!(n.as_f64(), 0.0);
    assert!(n.get_i64().is_err());
    assert!(n.get_u64().is_err());
    assert!(n.get_f64().is_err());
    assert!(n.as_vector().is_empty());
    assert!(n.as_map().is_empty());
    assert_eq!(n.as_vector().idx(1).flexbuffer_type(), FlexBufferType::Null);
    assert_eq!(n.as_map().idx("1").flexbuffer_type(), FlexBufferType::Null);
}
#[test]
fn get_root_deref_oob() {
    let s = &[
        4, // Deref out of bounds
        (FlexBufferType::Vector as u8) << 2 | BitWidth::W8 as u8,
        1,
    ];
    assert!(Reader::get_root(s.as_ref()).is_err());
}
#[test]
fn get_root_deref_u64() {
    let s = &[
        0,
        0,
        (FlexBufferType::IndirectUInt as u8) << 2 | BitWidth::W64 as u8,
        1,
    ];
    // The risk of crashing is reading 8 bytes from index 0.
    assert_eq!(Reader::get_root(s.as_ref()).unwrap().as_u64(), 0);
}

/// Verifies that the clone operation is shallow / zero copy.
#[test]
fn clone_is_shallow() {
    let mut fxb = Builder::default();
    let mut m = fxb.start_map();
    m.push("a", &[-1i8, -2, -3, -4]);
    m.push("b", 250i64);
    m.push("c", 5000u16);
    m.end_map();

    let r = Reader::get_root(fxb.view()).unwrap();

    let r2 = r.clone();

    assert_eq!(r.buffer().as_ptr(), r2.buffer().as_ptr());
}

#[test]
#[should_panic]
fn build_map_panic_on_repeated_key() {
    let mut b = Builder::default();
    let mut m = b.start_map();
    m.push("foo", 5u8);
    m.push("foo", 6u8);
    m.end_map();
}
#[test]
#[should_panic]
fn build_map_panic_on_internal_null() {
    let mut b = Builder::default();
    let mut m = b.start_map();
    m.push("foo\0", 5u8);
    m.end_map();
}
