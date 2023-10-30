// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::c_void;
use base::{CFAllocatorRef, CFTypeRef, CFIndex, CFRange, CFTypeID};
use string::CFStringRef;
use dictionary::CFDictionaryRef;

#[repr(C)]
pub struct __CFAttributedString(c_void);

pub type CFAttributedStringRef = *const __CFAttributedString;
pub type CFMutableAttributedStringRef = *const __CFAttributedString;

extern {
    /* CFAttributedString */

    pub fn CFAttributedStringCreate(
        allocator: CFAllocatorRef,
        str: CFStringRef,
        attributes: CFDictionaryRef,
    ) -> CFAttributedStringRef;

    pub fn CFAttributedStringGetLength(astr: CFAttributedStringRef) -> CFIndex;

    pub fn CFAttributedStringGetTypeID() -> CFTypeID;

    /* CFMutableAttributedString */

    pub fn CFAttributedStringCreateMutableCopy(
        allocator: CFAllocatorRef, max_length: CFIndex, astr: CFAttributedStringRef
    ) -> CFMutableAttributedStringRef;

    pub fn CFAttributedStringCreateMutable(
        allocator: CFAllocatorRef,
        max_length: CFIndex,
    ) -> CFMutableAttributedStringRef;

    pub fn CFAttributedStringReplaceString(
        astr: CFMutableAttributedStringRef, range: CFRange, replacement: CFStringRef);

    pub fn CFAttributedStringSetAttribute(
        astr: CFMutableAttributedStringRef,
        range: CFRange,
        attr_name: CFStringRef,
        value: CFTypeRef,
    );

}
