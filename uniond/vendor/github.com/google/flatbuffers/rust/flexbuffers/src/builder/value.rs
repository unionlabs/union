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

use byteorder::{LittleEndian, WriteBytesExt};

use crate::bitwidth::BitWidth;
use crate::bitwidth::BitWidth::*;
use crate::flexbuffer_type::FlexBufferType;
use crate::flexbuffer_type::FlexBufferType::*;

/// Internal representation of FlexBuffer Types and Data before writing.
/// These get placed on the builder's stack and are eventually commited.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    // Inline types
    Null,
    Int(i64),
    UInt(u64),
    Float(f64),
    Bool(bool),
    /// Null termintated, c_string. Only used with `Map`s.
    Key(usize),
    /// The other ~20 or so types.
    Reference {
        address: usize,
        child_width: BitWidth,
        fxb_type: FlexBufferType,
    },
}

macro_rules! new_typed_vector {
    ($name: ident, $v2: ident, $v3: ident, $v4: ident, $vn: ident) => {
        /// Returns a typed vector, fixed length if possible.
        /// Address and child width are zero initialized and must be set.
        pub fn $name(n: usize) -> Value {
            let address = 0;
            let child_width = W8;
            match n {
                2 => Value::Reference {
                    address,
                    child_width,
                    fxb_type: $v2,
                },
                3 => Value::Reference {
                    address,
                    child_width,
                    fxb_type: $v3,
                },
                4 => Value::Reference {
                    address,
                    child_width,
                    fxb_type: $v4,
                },
                _ => Value::Reference {
                    address,
                    child_width,
                    fxb_type: $vn,
                },
            }
        }
    };
}

impl Value {
    pub fn new_vector() -> Self {
        Value::Reference {
            address: 0,
            child_width: W8,
            fxb_type: Vector,
        }
    }
    pub fn new_map() -> Self {
        Value::Reference {
            address: 0,
            child_width: W8,
            fxb_type: Map,
        }
    }
    new_typed_vector!(
        new_int_vector,
        VectorInt2,
        VectorInt3,
        VectorInt4,
        VectorInt
    );
    new_typed_vector!(
        new_uint_vector,
        VectorUInt2,
        VectorUInt3,
        VectorUInt4,
        VectorUInt
    );
    new_typed_vector!(
        new_float_vector,
        VectorFloat2,
        VectorFloat3,
        VectorFloat4,
        VectorFloat
    );
    pub fn fxb_type(&self) -> FlexBufferType {
        match *self {
            Value::Null => Null,
            Value::Int(_) => Int,
            Value::UInt(_) => UInt,
            Value::Float(_) => Float,
            Value::Bool(_) => Bool,
            Value::Key(_) => Key,
            Value::Reference { fxb_type, .. } => fxb_type,
        }
    }
    pub fn is_fixed_length_vector(&self) -> bool {
        self.fxb_type().is_fixed_length_vector()
    }
    pub fn is_inline(&self) -> bool {
        self.fxb_type().is_inline()
    }
    pub fn is_reference(&self) -> bool {
        !self.is_inline()
    }
    pub fn is_key(&self) -> bool {
        if let Value::Key(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_typed_vector_or_map(&self) -> bool {
        if let Value::Reference { fxb_type, .. } = self {
            fxb_type.is_heterogenous()
        } else {
            false
        }
    }
    pub fn prefix_length(&self) -> usize {
        if self.is_fixed_length_vector() || self.is_inline() {
            return 0;
        }
        if let Value::Reference { fxb_type, .. } = self {
            if *fxb_type == Map {
                return 3;
            }
        }
        1
    }
    pub fn set_fxb_type_or_panic(&mut self, new_type: FlexBufferType) {
        if let Value::Reference { fxb_type, .. } = self {
            *fxb_type = new_type;
        } else {
            panic!("`set_fxb_type_or_panic` called on {:?}", self)
        }
    }
    pub fn set_child_width_or_panic(&mut self, new_width: BitWidth) {
        if let Value::Reference { child_width, .. } = self {
            *child_width = new_width;
        } else {
            panic!("`set_child_width_or_panic` called on {:?}", self);
        }
    }
    pub fn get_address(&self) -> Option<usize> {
        if let Value::Reference { address, .. } | Value::Key(address) = self {
            Some(*address)
        } else {
            None
        }
    }
    pub fn set_address_or_panic(&mut self, new_address: usize) {
        if let Value::Reference { address, .. } | Value::Key(address) = self {
            *address = new_address;
        } else {
            panic!("`set_address_or_panic` called on {:?}", self);
        }
    }
    /// For inline types - the width of the value to be stored.
    /// For reference types, the width of the referred.
    /// Note Key types always refer to 8 bit data.
    pub fn width_or_child_width(&self) -> BitWidth {
        match *self {
            Value::Int(x) => x.into(),
            Value::UInt(x) => x.into(),
            Value::Float(x) => x.into(),
            Value::Key(_) | Value::Bool(_) | Value::Null => W8,
            Value::Reference { child_width, .. } => child_width,
        }
    }
    pub fn relative_address(self, written_at: usize) -> Option<Value> {
        self.get_address().map(|address| {
            let offset = written_at
                .checked_sub(address)
                .expect("Error: References may only refer backwards in buffer.");
            Value::UInt(offset as u64)
        })
    }
    /// Computes the minimum required width of `value` when stored in a vector
    /// starting at `vector_start` at index `idx` (this index includes the prefix).
    /// `Value::Reference{..}` variants require location information because
    /// offsets are relative.
    pub fn width_in_vector(self, vector_start: usize, idx: usize) -> BitWidth {
        match self {
            Value::Bool(_) => W8,
            Value::Null => W8,
            Value::Int(x) => x.into(),
            Value::UInt(x) => x.into(),
            Value::Float(x) => x.into(),
            _ => {
                debug_assert!(self.is_reference());
                for &width in BitWidth::iter() {
                    let bytes = width as usize + 1;
                    let alignment = (bytes - vector_start % bytes) % bytes;
                    let written_at = vector_start + alignment + idx * bytes;
                    // This match must always succeed.
                    if let Some(Value::UInt(offset)) = self.relative_address(written_at) {
                        if BitWidth::from(offset) == width {
                            return width;
                        }
                    }
                }
                unreachable!()
            }
        }
    }
    pub fn packed_type(self, parent_width: BitWidth) -> u8 {
        let width = if self.is_inline() {
            std::cmp::max(parent_width, self.width_or_child_width())
        } else {
            self.width_or_child_width()
        };
        (self.fxb_type() as u8) << 2 | width as u8
    }
}

pub fn find_vector_type<'a, T>(mut values: T) -> Value
where
    T: std::iter::Iterator<Item = &'a Value>,
{
    let first = values.next();
    if first.is_none() {
        return Value::new_vector();
    }
    let mut len = 1;
    let init = first.unwrap().fxb_type();
    for v in values {
        if v.fxb_type() != init {
            return Value::new_vector();
        }
        len += 1;
    }
    let vector_type = match init {
        Bool => VectorBool,
        UInt => return Value::new_uint_vector(len),
        Int => return Value::new_int_vector(len),
        Float => return Value::new_float_vector(len),
        Key => VectorKey,
        // Note that VectorString is deprecated for writing
        _ => return Value::new_vector(),
    };
    Value::Reference {
        address: 0,
        child_width: W8,
        fxb_type: vector_type,
    }
}

#[inline]
pub fn store_value(buffer: &mut Vec<u8>, mut value: Value, width: BitWidth) {
    // Remap to number types.
    use Value::*;
    if let Some(offset) = value.relative_address(buffer.len()) {
        value = offset;
    } else {
        value = match value {
            Bool(x) => UInt(x.into()),
            Null => UInt(0), // Should this be 0 bytes?
            _ => value,
        }
    }
    let write_result = match (value, width) {
        (UInt(x), W8) => buffer.write_u8(x as u8),
        (UInt(x), W16) => buffer.write_u16::<LittleEndian>(x as u16),
        (UInt(x), W32) => buffer.write_u32::<LittleEndian>(x as u32),
        (UInt(x), W64) => buffer.write_u64::<LittleEndian>(x),
        (Int(x), W8) => buffer.write_i8(x as i8),
        (Int(x), W16) => buffer.write_i16::<LittleEndian>(x as i16),
        (Int(x), W32) => buffer.write_i32::<LittleEndian>(x as i32),
        (Int(x), W64) => buffer.write_i64::<LittleEndian>(x),
        (Float(x), W32) => buffer.write_f32::<LittleEndian>(x as f32),
        (Float(x), W64) => buffer.write_f64::<LittleEndian>(x),
        (Float(_), _) => unreachable!("Error: Flatbuffers does not support 8 and 16 bit floats."),
        _ => unreachable!("Variant not considered: {:?}", value),
    };
    write_result.unwrap_or_else(|err| {
        panic!(
            "Error writing value {:?} with width {:?}: {:?}",
            value, width, err
        )
    });
}
