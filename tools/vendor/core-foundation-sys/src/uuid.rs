// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::c_void;

use base::{CFAllocatorRef, CFTypeID};

#[repr(C)]
pub struct __CFUUID(c_void);

pub type CFUUIDRef = *const __CFUUID;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CFUUIDBytes {
    pub byte0:  u8,
    pub byte1:  u8,
    pub byte2:  u8,
    pub byte3:  u8,
    pub byte4:  u8,
    pub byte5:  u8,
    pub byte6:  u8,
    pub byte7:  u8,
    pub byte8:  u8,
    pub byte9:  u8,
    pub byte10: u8,
    pub byte11: u8,
    pub byte12: u8,
    pub byte13: u8,
    pub byte14: u8,
    pub byte15: u8
}

extern {
    /*
     * CFUUID.h
     */
    pub fn CFUUIDCreate(allocator: CFAllocatorRef) -> CFUUIDRef;
    pub fn CFUUIDCreateFromUUIDBytes(allocator: CFAllocatorRef, bytes: CFUUIDBytes) -> CFUUIDRef;
    pub fn CFUUIDGetUUIDBytes(uuid: CFUUIDRef) -> CFUUIDBytes;

    pub fn CFUUIDGetTypeID() -> CFTypeID;
}
