// Copyright 2021 Google LLC
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

use crate::reader::Reader;
use crate::Buffer;
use crate::{BitWidth::*, FlexBufferType::*};
use serde::ser;
use serde::ser::{SerializeMap, SerializeSeq};

impl<B: Buffer> ser::Serialize for &Reader<B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        #[allow(deprecated)]
        match (self.flexbuffer_type(), self.bitwidth()) {
            (Null, _) => serializer.serialize_unit(),
            (Int, W8) | (IndirectInt, W8) => serializer.serialize_i8(self.as_i8()),
            (Int, W16) | (IndirectInt, W16) => serializer.serialize_i16(self.as_i16()),
            (Int, W32) | (IndirectInt, W32) => serializer.serialize_i32(self.as_i32()),
            (Int, W64) | (IndirectInt, W64) => serializer.serialize_i64(self.as_i64()),
            (UInt, W8) | (IndirectUInt, W8) => serializer.serialize_u8(self.as_u8()),
            (UInt, W16) | (IndirectUInt, W16) => serializer.serialize_u16(self.as_u16()),
            (UInt, W32) | (IndirectUInt, W32) => serializer.serialize_u32(self.as_u32()),
            (UInt, W64) | (IndirectUInt, W64) => serializer.serialize_u64(self.as_u64()),
            (Float, W32) | (IndirectFloat, W32) => serializer.serialize_f32(self.as_f32()),
            (Float, _) | (IndirectFloat, _) => serializer.serialize_f64(self.as_f64()),
            (Bool, _) => serializer.serialize_bool(self.as_bool()),
            (Key, _) | (String, _) => serializer.serialize_str(&self.as_str()),
            (Map, _) => {
                let m = self.as_map();
                let mut map_serializer = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m.iter_keys().zip(m.iter_values()) {
                    map_serializer.serialize_key(&&k)?;
                    map_serializer.serialize_value(&&v)?;
                }
                map_serializer.end()
            }
            (Vector, _)
            | (VectorInt, _)
            | (VectorUInt, _)
            | (VectorFloat, _)
            | (VectorKey, _)
            | (VectorString, _)
            | (VectorBool, _)
            | (VectorInt2, _)
            | (VectorUInt2, _)
            | (VectorFloat2, _)
            | (VectorInt3, _)
            | (VectorUInt3, _)
            | (VectorFloat3, _)
            | (VectorInt4, _)
            | (VectorUInt4, _)
            | (VectorFloat4, _) => {
                let v = self.as_vector();
                let mut seq_serializer = serializer.serialize_seq(Some(v.len()))?;
                for x in v.iter() {
                    seq_serializer.serialize_element(&&x)?;
                }
                seq_serializer.end()
            }
            (Blob, _) => serializer.serialize_bytes(&self.as_blob().0),
        }
    }
}
