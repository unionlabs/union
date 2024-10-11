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

use crate::builder::Builder;
use crate::private::Sealed;
use crate::{Blob, Buffer, IndirectFloat, IndirectInt, IndirectUInt};

impl<B: Buffer> Sealed for Blob<B> {}
impl Sealed for () {}

// TODO: String interning
// TODO: Pushable for Map types?

/// Types that implement the Pushable trait can be written into a Flexbuffer.
///
/// All Rust's standard numbers, `u8, u16, u32, u64, i8, i16, i32, i64, f32, f64`,
/// can all be pushed. They are  `FlexBufferType::{UInt, Int, Float}`.
/// Flexbuffers chooses the smallest width that can represent the given number.
/// Strings can pe pushed, they become `FlexBufferType::String` and are stored
/// with both a length and null terminator.
///
/// * For convenience and speed push typed vectors using rust arrays and slices.
/// Doing so will immediately serialize the data, skipping the `Builder`'s
/// internal cache.
///
/// * Pushable cannot not be implemented by any downstream crates.
pub trait Pushable: Sealed + Sized {
    fn push_to_builder(self, _: &mut Builder) {}
}

impl Pushable for () {
    fn push_to_builder(self, builder: &mut Builder) {
        builder.push_null();
    }
}

impl<B: Buffer> Pushable for Blob<B> {
    fn push_to_builder(self, builder: &mut Builder) {
        builder.push_blob(&self.0);
    }
}

macro_rules! forward_to_builder {
    ($T: ty, $method: ident) => {
        impl Sealed for $T {}
        impl Pushable for $T {
            fn push_to_builder(self, builder: &mut Builder) {
                builder.$method(self);
            }
        }
    };
    ($T: ty, $method: ident, $asT: ty) => {
        impl Sealed for $T {}
        impl Pushable for $T {
            fn push_to_builder(self, builder: &mut Builder) {
                builder.$method(self as $asT);
            }
        }
    };
}
forward_to_builder!(&str, push_str);
forward_to_builder!(bool, push_bool);
forward_to_builder!(u8, push_uint);
forward_to_builder!(u16, push_uint);
forward_to_builder!(u32, push_uint);
forward_to_builder!(u64, push_uint);
forward_to_builder!(i8, push_int);
forward_to_builder!(i16, push_int);
forward_to_builder!(i32, push_int);
forward_to_builder!(i64, push_int);
forward_to_builder!(f32, push_float);
forward_to_builder!(f64, push_float);
forward_to_builder!(&[u8], push_uints);
forward_to_builder!(&[u16], push_uints);
forward_to_builder!(&[u32], push_uints);
forward_to_builder!(&[u64], push_uints);
forward_to_builder!(&[i8], push_ints);
forward_to_builder!(&[i16], push_ints);
forward_to_builder!(&[i32], push_ints);
forward_to_builder!(&[i64], push_ints);
forward_to_builder!(&[f32], push_floats);
forward_to_builder!(&[f64], push_floats);
forward_to_builder!(&[bool], push_bools);
forward_to_builder!(&Vec<u8>, push_uints);
forward_to_builder!(&Vec<u16>, push_uints);
forward_to_builder!(&Vec<u32>, push_uints);
forward_to_builder!(&Vec<u64>, push_uints);
forward_to_builder!(&Vec<i8>, push_ints);
forward_to_builder!(&Vec<i16>, push_ints);
forward_to_builder!(&Vec<i32>, push_ints);
forward_to_builder!(&Vec<i64>, push_ints);
forward_to_builder!(&Vec<f32>, push_floats);
forward_to_builder!(&Vec<f64>, push_floats);
forward_to_builder!(&Vec<bool>, push_bools);

macro_rules! impl_indirects {
    ($Indirect: ident, $method: ident) => {
        impl Sealed for $Indirect {}
        impl Pushable for $Indirect {
            fn push_to_builder(self, builder: &mut Builder) {
                builder.$method(self.0);
            }
        }
    };
}
impl_indirects!(IndirectInt, push_indirect_int);
impl_indirects!(IndirectUInt, push_indirect_uint);
impl_indirects!(IndirectFloat, push_indirect_float);

macro_rules! impl_arrays {
    ($num: expr) => {
        forward_to_builder!(&[u8; $num], push_uints, &[u8]);
        forward_to_builder!(&[u16; $num], push_uints, &[u16]);
        forward_to_builder!(&[u32; $num], push_uints, &[u32]);
        forward_to_builder!(&[u64; $num], push_uints, &[u64]);
        forward_to_builder!(&[i8; $num], push_ints, &[i8]);
        forward_to_builder!(&[i16; $num], push_ints, &[i16]);
        forward_to_builder!(&[i32; $num], push_ints, &[i32]);
        forward_to_builder!(&[i64; $num], push_ints, &[i64]);
        forward_to_builder!(&[f32; $num], push_floats, &[f32]);
        forward_to_builder!(&[f64; $num], push_floats, &[f64]);
        forward_to_builder!(&[bool; $num], push_bools, &[bool]);
    };
}
impl_arrays!(0);
impl_arrays!(1);
impl_arrays!(2);
impl_arrays!(3);
impl_arrays!(4);
impl_arrays!(5);
impl_arrays!(6);
// impl_arrays!(7);
// impl_arrays!(8);
// impl_arrays!(9);
// impl_arrays!(10);
// impl_arrays!(11);
// impl_arrays!(12);
// impl_arrays!(13);
// impl_arrays!(14);
// impl_arrays!(15);
// impl_arrays!(16);
// impl_arrays!(17);
// impl_arrays!(18);
// impl_arrays!(19);
// impl_arrays!(20);
// impl_arrays!(21);
// impl_arrays!(22);
// impl_arrays!(23);
// impl_arrays!(24);
// impl_arrays!(25);
// impl_arrays!(26);
// impl_arrays!(27);
// impl_arrays!(28);
// impl_arrays!(29);
// impl_arrays!(30);
// impl_arrays!(31);
// impl_arrays!(32);
