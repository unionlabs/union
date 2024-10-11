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

use core::ptr::write_bytes;

use crate::endian_scalar::emplace_scalar;
use crate::primitives::*;

/// VTableWriter compartmentalizes actions needed to create a vtable.
#[derive(Debug)]
pub struct VTableWriter<'a> {
    buf: &'a mut [u8],
}

impl<'a> VTableWriter<'a> {
    #[inline(always)]
    pub fn init(buf: &'a mut [u8]) -> Self {
        VTableWriter { buf }
    }

    /// Writes the vtable length (in bytes) into the vtable.
    ///
    /// Note that callers already need to have computed this to initialize
    /// a VTableWriter.
    ///
    /// In debug mode, asserts that the length of the underlying data is equal
    /// to the provided value.
    #[inline(always)]
    pub fn write_vtable_byte_length(&mut self, n: VOffsetT) {
        let buf = &mut self.buf[..SIZE_VOFFSET];
        // Safety:
        // Validated range above
        unsafe {
            emplace_scalar::<VOffsetT>(buf, n);
        }
        debug_assert_eq!(n as usize, self.buf.len());
    }

    /// Writes an object length (in bytes) into the vtable.
    #[inline(always)]
    pub fn write_object_inline_size(&mut self, n: VOffsetT) {
        let buf = &mut self.buf[SIZE_VOFFSET..2 * SIZE_VOFFSET];
        // Safety:
        // Validated range above
        unsafe {
            emplace_scalar::<VOffsetT>(buf, n);
        }
    }

    /// Writes an object field offset into the vtable.
    ///
    /// Note that this expects field offsets (which are like pointers), not
    /// field ids (which are like array indices).
    #[inline(always)]
    pub fn write_field_offset(&mut self, vtable_offset: VOffsetT, object_data_offset: VOffsetT) {
        let idx = vtable_offset as usize;
        let buf = &mut self.buf[idx..idx + SIZE_VOFFSET];
        // Safety:
        // Validated range above
        unsafe {
            emplace_scalar::<VOffsetT>(buf, object_data_offset);
        }
    }

    /// Clears all data in this VTableWriter. Used to cleanly undo a
    /// vtable write.
    #[inline(always)]
    pub fn clear(&mut self) {
        // This is the closest thing to memset in Rust right now.
        let len = self.buf.len();
        let p = self.buf.as_mut_ptr() as *mut u8;

        // Safety:
        // p is byte aligned and of length `len`
        unsafe {
            write_bytes(p, 0, len);
        }
    }
}
