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

use crate::follow::Follow;
use crate::primitives::*;
use crate::vtable::VTable;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Table<'a> {
    buf: &'a [u8],
    loc: usize,
}

impl<'a> Table<'a> {
    #[inline]
    pub fn buf(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub fn loc(&self) -> usize {
        self.loc
    }

    /// # Safety
    ///
    /// `buf` must contain a `soffset_t` at `loc`, which points to a valid vtable
    #[inline]
    pub unsafe fn new(buf: &'a [u8], loc: usize) -> Self {
        Table { buf, loc }
    }

    #[inline]
    pub fn vtable(&self) -> VTable<'a> {
        // Safety:
        // Table::new is created with a valid buf and location
        unsafe { <BackwardsSOffset<VTable<'a>>>::follow(self.buf, self.loc) }
    }

    /// Retrieves the value at the provided `slot_byte_loc` returning `default`
    /// if no value present
    ///
    /// # Safety
    ///
    /// The value of the corresponding slot must have type T
    #[inline]
    pub unsafe fn get<T: Follow<'a> + 'a>(
        &self,
        slot_byte_loc: VOffsetT,
        default: Option<T::Inner>,
    ) -> Option<T::Inner> {
        let o = self.vtable().get(slot_byte_loc) as usize;
        if o == 0 {
            return default;
        }
        Some(<T>::follow(self.buf, self.loc + o))
    }
}

impl<'a> Follow<'a> for Table<'a> {
    type Inner = Table<'a>;
    #[inline]
    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Table { buf, loc }
    }
}

/// Returns true if data contains a prefix of `ident`
#[inline]
pub fn buffer_has_identifier(data: &[u8], ident: &str, size_prefixed: bool) -> bool {
    assert_eq!(ident.len(), FILE_IDENTIFIER_LENGTH);

    let got = if size_prefixed {
        assert!(data.len() >= SIZE_SIZEPREFIX + SIZE_UOFFSET + FILE_IDENTIFIER_LENGTH);
        // Safety:
        // Verified data has sufficient bytes
        unsafe { <SkipSizePrefix<SkipRootOffset<FileIdentifier>>>::follow(data, 0) }
    } else {
        assert!(data.len() >= SIZE_UOFFSET + FILE_IDENTIFIER_LENGTH);
        // Safety:
        // Verified data has sufficient bytes
        unsafe { <SkipRootOffset<FileIdentifier>>::follow(data, 0) }
    };

    ident.as_bytes() == got
}
