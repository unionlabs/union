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
#![allow(deprecated)]
/// Represents all the valid types in a flexbuffer.
///
/// Flexbuffers supports
/// heterogenous maps, heterogenous vectors, typed vectors, and fixed length
/// typed vectors for some lengths and types. Rust types are converted into
/// Flexbuffers via the [Pushable](trait.Pushable.html) trait.
///
/// For exact details see the [internals document](
/// https://google.github.io/flatbuffers/flatbuffers_internals.html)
///
/// ### Notes:
/// * In the binary format, Each element of a `Map` or (heterogenous) `Vector`
/// is stored with a byte describing its FlexBufferType and BitWidth.
///
/// * Typed vectors do not store this extra type information and fixed length
/// typed vectors do not store length. Whether a vector is stored as a typed
/// vector or fixed length typed vector is determined dymaically from the
/// given data.
///
/// * Indirect numbers are stored as an offset instead of inline. Using
/// indirect numbers instead of their inline counterparts in maps and typed
/// vectors can reduce the minimum element width and therefore bytes used.

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, num_enum::TryFromPrimitive)]
pub enum FlexBufferType {
    /// Nulls are represented with `()` in Rust.
    Null = 0,
    /// Variable width signed integer: `i8, i16, i32, i64`
    Int = 1,
    /// Variable width unsigned integer: `u8, u16, u32, u64`
    UInt = 2,
    /// Variable width floating point: `f32, f64`
    Float = 3,
    Bool = 26,
    /// Null termintated, utf8 string. Typically used with `Map`s.
    Key = 4,
    /// Stored with a unsigned integer length, then UTF-8 bytes, and an extra null terminator that
    /// is not counted with the length.
    String = 5,
    /// An Int, stored by offset rather than inline. Indirect types can keep the bitwidth of a
    /// vector or map small when the inline value would have increased the bitwidth.
    IndirectInt = 6,
    /// A UInt, stored by offset rather than inline. Indirect types can keep the bitwidth of a
    /// vector or map small when the inline value would have increased the bitwidth.
    IndirectUInt = 7,
    /// A Float, stored by offset rather than inline. Indirect types can keep the bitwidth of a
    /// vector or map small when the inline value would have increased the bitwidth.
    IndirectFloat = 8,
    /// Maps are like Vectors except elements are associated with, and sorted by, keys.
    Map = 9,
    /// Heterogenous Vector (stored with a type table).
    Vector = 10,
    /// Homogenous Vector of Ints.
    VectorInt = 11,
    /// Homogenous Vector of UInts.
    VectorUInt = 12,
    /// Homogenous Vector of Floats.
    VectorFloat = 13,
    /// Homogenous Vector of Keys.
    VectorKey = 14,
    /// Homogenous Vector of Strings.
    #[deprecated(
        note = "Please use Vector or VectorKey instead. See https://github.com/google/flatbuffers/issues/5627"
    )]
    VectorString = 15,
    /// Since the elements of a vector use the same `BitWidth` as the length,
    /// Blob is more efficient for >255 element boolean vectors.
    VectorBool = 36,
    /// Homogenous vector of two Ints
    VectorInt2 = 16,
    /// Homogenous vector of two UInts
    VectorUInt2 = 17,
    /// Homogenous vector of two Floats
    VectorFloat2 = 18,
    /// Homogenous vector of three Ints
    VectorInt3 = 19,
    /// Homogenous vector of three UInts
    VectorUInt3 = 20,
    /// Homogenous vector of three Floats
    VectorFloat3 = 21,
    /// Homogenous vector of four Ints
    VectorInt4 = 22,
    /// Homogenous vector of four UInts
    VectorUInt4 = 23,
    /// Homogenous vector of four Floats
    VectorFloat4 = 24,
    /// An array of bytes. Stored with a variable width length.
    Blob = 25,
}
use FlexBufferType::*;

impl Default for FlexBufferType {
    fn default() -> Self {
        Null
    }
}

macro_rules! is_ty {
    ($is_T: ident, $FTy: ident) => {
        #[inline(always)]
        pub fn $is_T(self) -> bool {
            self == $FTy
        }
    };
}

impl FlexBufferType {
    /// Returns true for flexbuffer types that are stored inline.
    pub fn is_inline(self) -> bool {
        match self {
            Null | Int | UInt | Float | Bool => true,
            _ => false,
        }
    }
    /// Returns true for flexbuffer types that are stored by offset.
    pub fn is_reference(self) -> bool {
        !self.is_inline()
    }
    /// Returns true if called on a map, vector, typed vector, or fixed length typed vector.
    pub fn is_vector(self) -> bool {
        let d = self as u8;
        (9..25).contains(&d) || self == VectorBool
    }
    /// True iff the binary format stores the length.
    /// This applies to Blob, String, Maps, and Vectors of variable length.
    pub fn has_length_slot(self) -> bool {
        !self.is_fixed_length_vector() && self.is_vector() || self == String || self == Blob
    }
    /// Returns true if called on a fixed length typed vector.
    pub fn is_fixed_length_vector(self) -> bool {
        self.fixed_length_vector_length().is_some()
    }
    /// If called on a fixed type vector, returns the type of the elements.
    pub fn typed_vector_type(self) -> Option<FlexBufferType> {
        match self {
            VectorInt | VectorInt2 | VectorInt3 | VectorInt4 => Some(Int),
            VectorUInt | VectorUInt2 | VectorUInt3 | VectorUInt4 => Some(UInt),
            VectorFloat | VectorFloat2 | VectorFloat3 | VectorFloat4 => Some(Float),
            VectorKey => Some(Key),
            // Treat them as keys because we do not know width of length slot.
            // see deprecation link.
            VectorString => Some(Key),
            VectorBool => Some(Bool),
            _ => None,
        }
    }
    /// Return the length of the fixed length vector or None.
    pub fn fixed_length_vector_length(self) -> Option<usize> {
        match self {
            VectorInt2 | VectorUInt2 | VectorFloat2 => Some(2),
            VectorInt3 | VectorUInt3 | VectorFloat3 => Some(3),
            VectorInt4 | VectorUInt4 | VectorFloat4 => Some(4),
            _ => None,
        }
    }
    /// Returns true if self is a Map or Vector. Typed vectors are not heterogenous.
    pub fn is_heterogenous(self) -> bool {
        self == Map || self == Vector
    }
    /// If `self` is an indirect scalar, remap it to the scalar. Otherwise do nothing.
    pub fn to_direct(self) -> Option<Self> {
        match self {
            IndirectInt => Some(Int),
            IndirectUInt => Some(UInt),
            IndirectFloat => Some(Float),
            _ => None,
        }
    }
    // returns true if and only if the flexbuffer type is `Null`.
    is_ty!(is_null, Null);
    // returns true if and only if the flexbuffer type is `Int`.
    is_ty!(is_int, Int);
    // returns true if and only if the flexbuffer type is `UInt`.
    is_ty!(is_uint, UInt);
    // returns true if and only if the flexbuffer type is `Float`.
    is_ty!(is_float, Float);
    // returns true if and only if the flexbuffer type is `Bool`.
    is_ty!(is_bool, Bool);
    // returns true if and only if the flexbuffer type is `Key`.
    is_ty!(is_key, Key);
    // returns true if and only if the flexbuffer type is `String`.
    is_ty!(is_string, String);
    // returns true if and only if the flexbuffer type is `IndirectInt`.
    is_ty!(is_indirect_int, IndirectInt);
    // returns true if and only if the flexbuffer type is `IndirectUInt`.
    is_ty!(is_indirect_uint, IndirectUInt);
    // returns true if and only if the flexbuffer type is `IndirectFloat`.
    is_ty!(is_indirect_float, IndirectFloat);
    // returns true if and only if the flexbuffer type is `Map`.
    is_ty!(is_map, Map);
    // returns true if and only if the flexbuffer type is `Vector`.
    is_ty!(is_heterogenous_vector, Vector);
    // returns true if and only if the flexbuffer type is `VectorInt`.
    is_ty!(is_vector_int, VectorInt);
    // returns true if and only if the flexbuffer type is `VectorUInt`.
    is_ty!(is_vector_uint, VectorUInt);
    // returns true if and only if the flexbuffer type is `VectorFloat`.
    is_ty!(is_vector_float, VectorFloat);
    // returns true if and only if the flexbuffer type is `VectorKey`.
    is_ty!(is_vector_key, VectorKey);
    // returns true if and only if the flexbuffer type is `VectorString`.
    is_ty!(is_vector_string, VectorString);
    // returns true if and only if the flexbuffer type is `VectorBool`.
    is_ty!(is_vector_bool, VectorBool);
    // returns true if and only if the flexbuffer type is `VectorInt2`.
    is_ty!(is_vector_int2, VectorInt2);
    // returns true if and only if the flexbuffer type is `VectorUInt2`.
    is_ty!(is_vector_uint2, VectorUInt2);
    // returns true if and only if the flexbuffer type is `VectorFloat2`.
    is_ty!(is_vector_float2, VectorFloat2);
    // returns true if and only if the flexbuffer type is `VectorInt3`.
    is_ty!(is_vector_int3, VectorInt3);
    // returns true if and only if the flexbuffer type is `VectorUInt3`.
    is_ty!(is_vector_uint3, VectorUInt3);
    // returns true if and only if the flexbuffer type is `VectorFloat3`.
    is_ty!(is_vector_float3, VectorFloat3);
    // returns true if and only if the flexbuffer type is `VectorInt4`.
    is_ty!(is_vector_int4, VectorInt4);
    // returns true if and only if the flexbuffer type is `VectorUInt4`.
    is_ty!(is_vector_uint4, VectorUInt4);
    // returns true if and only if the flexbuffer type is `VectorFloat4`.
    is_ty!(is_vector_float4, VectorFloat4);
    // returns true if and only if the flexbuffer type is `Blob`.
    is_ty!(is_blob, Blob);
}
