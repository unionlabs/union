// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::c_void;

use base::{CFRange, CFIndex, CFAllocatorRef, CFTypeID, Boolean};
use string::CFStringRef;

pub type CFArrayRetainCallBack = extern "C" fn(allocator: CFAllocatorRef, value: *const c_void) -> *const c_void;
pub type CFArrayReleaseCallBack = extern "C" fn(allocator: CFAllocatorRef, value: *const c_void);
pub type CFArrayCopyDescriptionCallBack = extern "C" fn(value: *const c_void) -> CFStringRef;
pub type CFArrayEqualCallBack = extern "C" fn(value1: *const c_void, value2: *const c_void) -> Boolean;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CFArrayCallBacks {
    pub version: CFIndex,
    pub retain: CFArrayRetainCallBack,
    pub release: CFArrayReleaseCallBack,
    pub copyDescription: CFArrayCopyDescriptionCallBack,
    pub equal: CFArrayEqualCallBack,
}

#[repr(C)]
pub struct __CFArray(c_void);

pub type CFArrayRef = *const __CFArray;

extern {
    /*
     * CFArray.h
     */
    pub static kCFTypeArrayCallBacks: CFArrayCallBacks;

    pub fn CFArrayCreate(allocator: CFAllocatorRef, values: *const *const c_void,
                     numValues: CFIndex, callBacks: *const CFArrayCallBacks) -> CFArrayRef;
    pub fn CFArrayCreateCopy(allocator: CFAllocatorRef , theArray: CFArrayRef) -> CFArrayRef;
    
    // CFArrayBSearchValues
    // CFArrayContainsValue
    pub fn CFArrayGetCount(theArray: CFArrayRef) -> CFIndex;
    // CFArrayGetCountOfValue
    // CFArrayGetFirstIndexOfValue
    // CFArrayGetLastIndexOfValue
    pub fn CFArrayGetValues(theArray: CFArrayRef, range: CFRange, values: *mut *const c_void);
    pub fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: CFIndex) -> *const c_void;
    // CFArrayApplyFunction
    pub fn CFArrayGetTypeID() -> CFTypeID;
}
