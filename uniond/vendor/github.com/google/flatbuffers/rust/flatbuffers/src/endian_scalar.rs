/*
 * Copyright 2018 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
#![allow(clippy::wrong_self_convention)]

use core::mem::size_of;

mod private {
    /// Types that are trivially transmutable are those where any combination of bits
    /// represents a valid value of that type
    ///
    /// For example integral types are TriviallyTransmutable as all bit patterns are valid,
    /// however, `bool` is not trivially transmutable as only `0` and `1` are valid
    pub trait TriviallyTransmutable {}

    impl TriviallyTransmutable for i8 {}
    impl TriviallyTransmutable for i16 {}
    impl TriviallyTransmutable for i32 {}
    impl TriviallyTransmutable for i64 {}
    impl TriviallyTransmutable for u8 {}
    impl TriviallyTransmutable for u16 {}
    impl TriviallyTransmutable for u32 {}
    impl TriviallyTransmutable for u64 {}
}

/// Trait for values that must be stored in little-endian byte order, but
/// might be represented in memory as big-endian. Every type that implements
/// EndianScalar is a valid FlatBuffers scalar value.
///
/// The Rust stdlib does not provide a trait to represent scalars, so this trait
/// serves that purpose, too.
///
/// Note that we do not use the num-traits crate for this, because it provides
/// "too much". For example, num-traits provides i128 support, but that is an
/// invalid FlatBuffers type.
pub trait EndianScalar: Sized + PartialEq + Copy + Clone {
    type Scalar: private::TriviallyTransmutable;

    fn to_little_endian(self) -> Self::Scalar;

    fn from_little_endian(v: Self::Scalar) -> Self;
}

/// Macro for implementing an endian conversion using the stdlib `to_le` and
/// `from_le` functions. This is used for integer types. It is not used for
/// floats, because the `to_le` and `from_le` are not implemented for them in
/// the stdlib.
macro_rules! impl_endian_scalar {
    ($ty:ident) => {
        impl EndianScalar for $ty {
            type Scalar = Self;

            #[inline]
            fn to_little_endian(self) -> Self::Scalar {
                Self::to_le(self)
            }
            #[inline]
            fn from_little_endian(v: Self::Scalar) -> Self {
                Self::from_le(v)
            }
        }
    };
}

impl_endian_scalar!(u8);
impl_endian_scalar!(i8);
impl_endian_scalar!(u16);
impl_endian_scalar!(u32);
impl_endian_scalar!(u64);
impl_endian_scalar!(i16);
impl_endian_scalar!(i32);
impl_endian_scalar!(i64);

impl EndianScalar for bool {
    type Scalar = u8;

    fn to_little_endian(self) -> Self::Scalar {
        self as u8
    }

    fn from_little_endian(v: Self::Scalar) -> Self {
        v != 0
    }
}

impl EndianScalar for f32 {
    type Scalar = u32;
    /// Convert f32 from host endian-ness to little-endian.
    #[inline]
    fn to_little_endian(self) -> u32 {
        // Floats and Ints have the same endianness on all supported platforms.
        // <https://doc.rust-lang.org/std/primitive.f32.html#method.from_bits>
        self.to_bits().to_le()
    }
    /// Convert f32 from little-endian to host endian-ness.
    #[inline]
    fn from_little_endian(v: u32) -> Self {
        // Floats and Ints have the same endianness on all supported platforms.
        // <https://doc.rust-lang.org/std/primitive.f32.html#method.from_bits>
        f32::from_bits(u32::from_le(v))
    }
}

impl EndianScalar for f64 {
    type Scalar = u64;

    /// Convert f64 from host endian-ness to little-endian.
    #[inline]
    fn to_little_endian(self) -> u64 {
        // Floats and Ints have the same endianness on all supported platforms.
        // <https://doc.rust-lang.org/std/primitive.f64.html#method.from_bits>
        self.to_bits().to_le()
    }
    /// Convert f64 from little-endian to host endian-ness.
    #[inline]
    fn from_little_endian(v: u64) -> Self {
        // Floats and Ints have the same endianness on all supported platforms.
        // <https://doc.rust-lang.org/std/primitive.f64.html#method.from_bits>
        f64::from_bits(u64::from_le(v))
    }
}

/// Place an EndianScalar into the provided mutable byte slice. Performs
/// endian conversion, if necessary.
/// # Safety
/// Caller must ensure `s.len() >= size_of::<T>()`
#[inline]
pub unsafe fn emplace_scalar<T: EndianScalar>(s: &mut [u8], x: T) {
    let size = size_of::<T::Scalar>();
    debug_assert!(
        s.len() >= size,
        "insufficient capacity for emplace_scalar, needed {} got {}",
        size,
        s.len()
    );

    let x_le = x.to_little_endian();
    core::ptr::copy_nonoverlapping(
        &x_le as *const T::Scalar as *const u8,
        s.as_mut_ptr() as *mut u8,
        size,
    );
}

/// Read an EndianScalar from the provided byte slice at the specified location.
/// Performs endian conversion, if necessary.
/// # Safety
/// Caller must ensure `s.len() >= loc + size_of::<T>()`.
#[inline]
pub unsafe fn read_scalar_at<T: EndianScalar>(s: &[u8], loc: usize) -> T {
    read_scalar(&s[loc..])
}

/// Read an EndianScalar from the provided byte slice. Performs endian
/// conversion, if necessary.
/// # Safety
/// Caller must ensure `s.len() > size_of::<T>()`.
#[inline]
pub unsafe fn read_scalar<T: EndianScalar>(s: &[u8]) -> T {
    let size = size_of::<T::Scalar>();
    debug_assert!(
        s.len() >= size,
        "insufficient capacity for emplace_scalar, needed {} got {}",
        size,
        s.len()
    );

    let mut mem = core::mem::MaybeUninit::<T::Scalar>::uninit();
    // Since [u8] has alignment 1, we copy it into T which may have higher alignment.
    core::ptr::copy_nonoverlapping(s.as_ptr(), mem.as_mut_ptr() as *mut u8, size);
    T::from_little_endian(mem.assume_init())
}
