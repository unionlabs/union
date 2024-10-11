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

use alloc::string::{String, ToString};
use alloc::vec::Vec;

// Read what you wrote.
use flexbuffers::*;
#[cfg(not(miri))]  // slow.
use quickcheck;
use serde::{Deserialize, Serialize};

// TODO(cneo): Upstream this to the quickcheck crate. Also, write a macro to derive Arbitrary.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct NonNullString(String);
impl quickcheck::Arbitrary for NonNullString {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
        let size = core::cmp::min(1, usize::arbitrary(g));
        NonNullString(
            (0..)
                .map(|_| <char>::arbitrary(g))
                .filter(|&b| b != '\0')
                .take(size)
                .collect(),
        )
    }
}

#[cfg(not(miri))]  // slow.
quickcheck! {
    fn qc_vec_bool(xs: Vec<bool>) -> bool {
        let mut builder = Builder::default();
        let mut v = builder.start_vector();
        for &x in &xs {
            v.push(x);
        }
        v.end_vector();
        let r = Reader::get_root(builder.view()).unwrap().as_vector();
        xs.iter().enumerate().all(|(i, &x)| r.index(i).unwrap().get_bool().unwrap() == x)
    }
    fn qc_vec_uint(xs: Vec<u64>) -> bool {
        let mut builder = Builder::default();
        let mut v = builder.start_vector();
        for &x in &xs {
            v.push(x);
        }
        v.end_vector();
        let r = Reader::get_root(builder.view()).unwrap().as_vector();
        xs.iter().enumerate().all(|(i, &x)| r.idx(i).as_u64() == x)
    }
    fn qc_vec_int(xs: Vec<i64>) -> bool {
        let mut builder = Builder::default();
        let mut v = builder.start_vector();
        for &x in &xs {
            v.push(x);
        }
        v.end_vector();
        let r = Reader::get_root(builder.view()).unwrap().as_vector();
        xs.iter().enumerate().all(|(i, &x)| r.idx(i).as_i64() == x)
    }
    fn qc_vec_float(xs: Vec<f64>) -> bool {
        let mut builder = Builder::default();
        let mut v = builder.start_vector();
        for &x in &xs {
            v.push(x);
        }
        v.end_vector();
        let r = Reader::get_root(builder.view()).unwrap().as_vector();
        xs.iter().enumerate().all(|(i, &x)| (r.idx(i).as_f64() - x).abs() < core::f64::EPSILON)
    }
    fn qc_vec_string(xs: Vec<String>) -> bool {
        let mut builder = Builder::default();
        let mut v = builder.start_vector();
        for x in &xs {
            v.push(x as &str);
        }
        v.end_vector();
        let r = Reader::get_root(builder.view()).unwrap().as_vector();
        xs.iter().enumerate().all(|(i, x)| (r.idx(i).as_str() == x))
    }
    #[cfg(not(feature = "no_std"))]
    fn qc_map_int(xs: std::collections::BTreeMap<NonNullString, i64>) -> bool {
        let mut builder = Builder::default();
        let mut m = builder.start_map();
        for (k, &v) in &xs {
            m.push(&k.0, v);
        }
        m.end_map();
        let r = Reader::get_root(builder.view()).unwrap().as_map();
        xs.iter().enumerate().all(|(i, (k, &v))| {
            r.idx(i).as_i64() == v && r.idx(k.0.as_str()).as_i64() == v
        })
    }
    #[cfg(not(feature = "no_std"))]
    fn qc_map_string(xs: std::collections::BTreeMap<NonNullString, String>) -> bool {
        let mut builder = Builder::default();
        let mut m = builder.start_map();
        for (k, v) in &xs {
            m.push(&k.0, v as &str);
        }
        m.end_map();
        let r = Reader::get_root(builder.view()).unwrap().as_map();
        xs.iter().enumerate().all(|(i, (k, v))| {
            r.idx(i).as_str() == v && r.idx(k.0.as_str()).as_str() == v
        })
    }
    fn qc_blob(xs: Vec<Vec<u8>>) -> bool {
        let mut builder = Builder::default();
        let mut v = builder.start_vector();
        for x in &xs {
            v.push(Blob(x.as_ref()));
        }
        v.end_vector();
        let r = Reader::get_root(builder.view()).unwrap().as_vector();
        xs.iter().enumerate().all(
            |(i, x)| r.idx(i).get_blob().unwrap().0.iter().eq(x.iter())
        )
    }
    fn qc_serde_ints(
        u8s: Vec<u8>,
        u16s: Vec<u16>,
        u32s: Vec<u32>,
        u64s: Vec<u64>,
        i8s: Vec<i8>,
        i16s: Vec<i16>,
        i32s: Vec<i32>,
        i64s: Vec<i64>
    ) -> bool {
        #[derive(Serialize, Deserialize, PartialEq)]
        struct Foo {
            u8s: Vec<u8>,
            u16s: Vec<u16>,
            u32s: Vec<u32>,
            u64s: Vec<u64>,
            i8s: Vec<i8>,
            i16s: Vec<i16>,
            i32s: Vec<i32>,
            i64s: Vec<i64>,
        }
        let mut ser = FlexbufferSerializer::new();
        let foo1 = Foo { u8s, u16s, u32s, u64s, i8s, i16s, i32s, i64s };
        foo1.serialize(&mut ser).unwrap();
        let r = Reader::get_root(ser.view()).unwrap();
        let foo2 = Foo::deserialize(r).unwrap();
        foo1 == foo2
    }
    fn qc_serde_others(
        bools: Vec<bool>,
        strings: Vec<String>,
        f32s: Vec<f32>,
        f64s: Vec<f64>
    ) -> bool {
        #[derive(Serialize, Deserialize, PartialEq)]
        struct Foo {
            bools: Vec<bool>,
            strings: Vec<String>,
            f32s: Vec<f32>,
            f64s: Vec<f64>,
        }
        let mut ser = FlexbufferSerializer::new();
        let foo1 = Foo { bools, strings, f32s, f64s };
        foo1.serialize(&mut ser).unwrap();
        let r = Reader::get_root(ser.view()).unwrap();
        let foo2 = Foo::deserialize(r).unwrap();
        foo1 == foo2
    }
    fn qc_serde_others2(
        bools: Vec<bool>,
        strings: Vec<String>,
        f32s: Vec<f32>,
        f64s: Vec<f64>
    ) -> bool {
        #[derive(Serialize, Deserialize, PartialEq)]
        struct Foo (Vec<bool>, Vec<String>, Vec<f32>, Vec<f64>);
        let mut ser = FlexbufferSerializer::new();
        let foo1 = Foo(bools, strings, f32s, f64s);
        foo1.serialize(&mut ser).unwrap();
        let r = Reader::get_root(ser.view()).unwrap();
        let foo2 = Foo::deserialize(r).unwrap();
        foo1 == foo2
    }

}

#[test]
fn empty_vectors() {
    #[derive(PartialEq, Serialize, Deserialize, Default, Debug)]
    struct Foo(Vec<u8>, Vec<i8>);
    let foo1 = Foo::default();
    let mut s = FlexbufferSerializer::new();
    foo1.serialize(&mut s).unwrap();
    let r = Reader::get_root(s.view()).unwrap();
    let foo2 = Foo::deserialize(r).unwrap();
    assert_eq!(foo1, foo2);
}

#[test]
fn string() {
    let mut builder = Builder::default();
    let mut v = builder.start_vector();
    v.push("foo");
    v.push("barrr");
    v.push("bazzzzzz");
    v.end_vector();
    let r = Reader::get_root(builder.view()).unwrap().as_vector();
    assert_eq!(r.idx(0).as_str(), "foo");
    assert_eq!(r.idx(1).as_str(), "barrr");
    assert_eq!(r.idx(2).as_str(), "bazzzzzz");
}

#[test]
fn store_13() {
    let finished = singleton::<i32>(13);
    let r = Reader::get_root(finished.as_ref()).unwrap();
    assert_eq!(r.as_i32(), 13);
}
#[test]
fn singleton_vector_uint_4_16bit() {
    let mut builder = Builder::default();
    let mut v = builder.start_vector();
    v.push(2u8);
    v.push(3u8);
    v.push(5u8);
    v.end_vector();
    let buf1 = builder.view();
    let buf2 = singleton(&[2u8, 3, 5]);
    assert_eq!(buf1, buf2.as_slice());

    let r = Reader::get_root(buf1).unwrap().as_vector();
    assert_eq!(r.idx(0).get_u64(), Ok(2));
    assert_eq!(r.idx(1).get_u64(), Ok(3));
    assert_eq!(r.idx(2).get_u64(), Ok(5));
    assert_eq!(r.index(3).unwrap_err(), ReaderError::IndexOutOfBounds);
}
#[test]
fn vector_uint4() {
    let mut fxb = Builder::default();
    let mut v = fxb.start_vector();
    v.push(2u8);
    v.push(3u8);
    v.push(5u8);
    v.push(7u8);
    v.end_vector();
    let r = Reader::get_root(fxb.view()).unwrap();
    let v = r.as_vector();
    assert_eq!(v.idx(0).get_u64(), Ok(2));
    assert_eq!(v.idx(1).get_u64(), Ok(3));
    assert_eq!(v.idx(2).get_u64(), Ok(5));
    assert_eq!(v.idx(3).get_u64(), Ok(7));
    assert!(v.index(4).is_err());
    #[allow(deprecated)]
    #[cfg(target_endian = "little")]
    {
        assert_eq!(r.get_slice::<u8>().unwrap(), [2, 3, 5, 7]);
    }
}
#[test]
fn store_and_read_blob() {
    let mut fxb = Builder::default();
    let mut v = fxb.start_vector();
    v.push(Blob([1, 2, 3, 4].as_ref()));
    v.push(Blob([5, 6, 7].as_ref()));
    v.end_vector();

    let r = Reader::get_root(fxb.view()).unwrap().as_vector();
    assert_eq!(r.idx(0).get_blob(), Ok(Blob([1, 2, 3, 4].as_ref())));
    assert_eq!(r.idx(1).get_blob(), Ok(Blob([5, 6, 7].as_ref())));
}
#[test]
fn map_64bit() {
    let mut fxb = Builder::default();
    let mut m = fxb.start_map();
    m.push("a", 257u16);
    m.push("b", u64::max_value() - 3);
    m.end_map();

    let r = Reader::get_root(fxb.view()).unwrap().as_map();
    assert_eq!(r.idx("a").as_u16(), 257);
    assert_eq!(r.idx("b").as_u64(), u64::max_value() - 3);
}
#[test]
fn index_map() {
    let mut fxb = Builder::default();
    let mut m = fxb.start_map();
    m.push("foo", 17u8);
    m.push("bar", 33u16);
    m.push("baz", 41u32);
    m.end_map();

    let r = Reader::get_root(fxb.view()).unwrap().as_map();
    assert_eq!(r.idx(0).get_u64(), Ok(33));
    assert_eq!(r.idx(1).get_u64(), Ok(41));
    assert_eq!(r.idx(2).as_u8(), 17);
    assert_eq!(r.index(3).unwrap_err(), ReaderError::IndexOutOfBounds);

    assert_eq!(r.idx("bar").as_u64(), 33);
    assert_eq!(r.idx("baz").as_u32(), 41);
    assert_eq!(r.idx("foo").as_u16(), 17);
    assert_eq!(r.index("???").unwrap_err(), ReaderError::KeyNotFound);
}

#[test]
fn map_strings() {
    let mut fxb = Builder::default();
    {
        let mut m = fxb.start_map();
        let mut a = m.start_vector("a");
        for &s in ["b", "c", "d", "e"].iter() {
            a.push(s);
        }
        a.end_vector();
        let mut f = m.start_vector("f");
        for &s in ["gh", "ij"].iter() {
            f.push(s);
        }
    }
    let r = Reader::get_root(fxb.view()).unwrap().as_map();
    let a = r.idx("a").as_vector();

    assert_eq!(a.idx(0).as_str(), "b");
    assert_eq!(a.idx(1).as_str(), "c");
    assert_eq!(a.idx(2).as_str(), "d");
    assert_eq!(a.idx(3).as_str(), "e");

    let f = r.idx("f").as_vector();
    assert_eq!(f.idx(0).as_str(), "gh");
    assert_eq!(f.idx(1).as_str(), "ij");

    // Defaults to empty string for index errors.
    assert_eq!(r.idx("a").as_vector().idx(4).as_str(), "");
    assert_eq!(r.idx("b").as_vector().idx(2).as_str(), "");
    assert_eq!(r.idx("c").as_str(), "");
}

#[test]
fn store_u64() {
    let finished = singleton(u64::max_value() - 10);
    let r = Reader::get_root(finished.as_ref()).unwrap();
    assert_eq!(r.get_u64(), Ok(u64::max_value() - 10));
}
#[test]
fn store_indirects() {
    let mut b = Builder::default();
    let mut v = b.start_vector();
    v.push(IndirectInt(-42));
    v.push(IndirectUInt(9000));
    v.push(IndirectFloat(3.14));
    v.end_vector();
    let r = Reader::get_root(b.view()).unwrap().as_vector();
    assert_eq!(r.idx(0).get_i64().unwrap(), -42);
    assert_eq!(r.idx(1).get_u64().unwrap(), 9000);
    assert_eq!(r.idx(2).get_f64().unwrap(), 3.14);
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Foo {
    a: i8,
    b: f64,
    c: Vec<u32>,
    d: String,
}

#[cfg(not(miri))]  // slow.
quickcheck! {
    fn serde_foo(a: i8,
    b: f64,
    c: Vec<u32>,
    d: String) -> bool {
        let mut s = FlexbufferSerializer::new();
        let data = Foo { a, b, c, d };
        data.serialize(&mut s).unwrap();

        let read = Foo::deserialize(Reader::get_root(s.view()).unwrap()).unwrap();
        data == read
    }
}

#[test]
fn serde_serious() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum MyEnum {
        Unit,
        NewType([i32; 3]),
        Tuple(f32, f64),
        Struct { a: u8, b: u16, c: u32 },
    }
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MyNewType;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MyStruct {
        a: u8,
        b: u16,
        c: u32,
        d: u64,
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MyUnitStruct(Vec<String>);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct MyTupleStruct(MyNewType, MyUnitStruct, MyStruct, Vec<MyEnum>);

    let data = MyTupleStruct(
        MyNewType,
        MyUnitStruct(vec!["Hello".to_string(), "World".to_string()]),
        MyStruct {
            a: 2,
            b: 4,
            c: 8,
            d: 16,
        },
        vec![
            MyEnum::Unit,
            MyEnum::NewType([-1, 0, 1]),
            MyEnum::Unit,
            MyEnum::Tuple(3.14, 2.71),
            MyEnum::Struct {
                a: 32,
                b: 64,
                c: 128,
            },
        ],
    );

    let mut s = FlexbufferSerializer::new();
    data.serialize(&mut s).unwrap();

    let reader = Reader::get_root(s.view()).unwrap();
    let read = MyTupleStruct::deserialize(reader).unwrap();
    assert_eq!(data, read);
}
#[test]
fn serialize_serde_with_bytes_as_blob() {
    #[derive(Serialize, Deserialize)]
    struct Foo(#[serde(with = "serde_bytes")] Vec<u8>);
    let mut s = FlexbufferSerializer::new();
    Foo(vec![5, 6, 7, 8]).serialize(&mut s).unwrap();
    let reader = Reader::get_root(s.view()).unwrap();
    assert_eq!(reader.flexbuffer_type(), FlexBufferType::Blob);
    assert_eq!(reader.as_blob(), Blob([5, 6, 7, 8].as_ref()));
}
#[test]
fn iter() {
    let mut fxb = Builder::default();
    {
        let mut m = fxb.start_map();
        m.push("a", "42");
        m.push("b", 250i64);
        m.push("c", 5000u16);
    }
    let r = Reader::get_root(fxb.view()).unwrap();

    let v: Vec<u32> = r.as_vector().iter().map(|x| x.as_u32()).collect();
    assert_eq!(&v, &[42, 250, 5000]);
}

#[test]
fn deserialize_newtype_i8() {
    #[derive(Deserialize)]
    struct Foo(u8);
    let data = [13, 4, 1];
    let r = Reader::get_root(data.as_ref()).unwrap();
    let foo = Foo::deserialize(r).unwrap();
    assert_eq!(foo.0, 13);
}
#[test]
fn deserialize_newtype_str() {
    #[derive(Deserialize)]
    struct Foo<'a>(&'a str);
    let data = [5, b'h', b'e', b'l', b'l', b'o', b'\0', 6, 5 << 2, 1];
    let r = Reader::get_root(data.as_ref()).unwrap();
    let foo = Foo::deserialize(r).unwrap();
    assert_eq!(foo.0, "hello");
}
#[test]
#[rustfmt::skip]
fn deserialize_tuple_struct_to_vec_uint4() {
    #[derive(Deserialize)]
    struct Foo(u8, u16, u32, u64);
    let data = [
        4, 0, 16, 0, 64, 0, 0, 1, // Data
        8,              // Vector offset.
        23 << 2 | 1,    // (VectorUInt4, W16 - referring to data).
        1,              // Root width W8 - referring to vector.
    ];
    let r = Reader::get_root(data.as_ref()).unwrap();
    let foo = Foo::deserialize(r).unwrap();
    assert_eq!(foo.0, 4);
    assert_eq!(foo.1, 16);
    assert_eq!(foo.2, 64);
    assert_eq!(foo.3, 256);

    let data = [
        1, 2, 3, 4, // The vector.
        4,          // Root data (offset).
        23 << 2,    // Root type: VectorUInt4, W8.
        1,          // Root width: W8.
    ];
    let r = Reader::get_root(data.as_ref()).unwrap();
    let foo = Foo::deserialize(r).unwrap();
    assert_eq!(foo.0, 1);
    assert_eq!(foo.1, 2);
    assert_eq!(foo.2, 3);
    assert_eq!(foo.3, 4);
}
