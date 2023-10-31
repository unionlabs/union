// Copyright 2016 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::c_void;

use base::{CFTypeID, CFIndex};
use string::CFStringRef;

#[repr(C)]
pub struct __CFError(c_void);

pub type CFErrorRef = *mut __CFError;

extern "C" {
    pub fn CFErrorGetTypeID() -> CFTypeID;

    pub static kCFErrorDomainPOSIX: CFStringRef;
    pub static kCFErrorDomainOSStatus: CFStringRef;
    pub static kCFErrorDomainMach: CFStringRef;
    pub static kCFErrorDomainCocoa: CFStringRef;

    pub fn CFErrorGetDomain(err: CFErrorRef) -> CFStringRef;
    pub fn CFErrorGetCode(err: CFErrorRef) -> CFIndex;

    pub fn CFErrorCopyDescription(err: CFErrorRef) -> CFStringRef;
}
