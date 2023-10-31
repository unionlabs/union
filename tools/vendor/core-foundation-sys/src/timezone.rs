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
use date::{CFTimeInterval, CFAbsoluteTime};
use string::CFStringRef;

#[repr(C)]
pub struct __CFTimeZone(c_void);

pub type CFTimeZoneRef = *const __CFTimeZone;

extern {
    pub fn CFTimeZoneCopySystem() -> CFTimeZoneRef;
    pub fn CFTimeZoneCopyDefault() -> CFTimeZoneRef;
    pub fn CFTimeZoneCreateWithTimeIntervalFromGMT(allocator: CFAllocatorRef, interval: CFTimeInterval) -> CFTimeZoneRef;
    pub fn CFTimeZoneGetSecondsFromGMT(tz: CFTimeZoneRef, time: CFAbsoluteTime) -> CFTimeInterval;

    pub fn CFTimeZoneGetTypeID() -> CFTypeID;
    pub fn CFTimeZoneGetName(tz: CFTimeZoneRef) -> CFStringRef;
}
