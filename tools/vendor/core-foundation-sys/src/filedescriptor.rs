// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::{c_int, c_void};

use base::{Boolean, CFIndex, CFTypeID, CFOptionFlags, CFAllocatorRef};
use string::CFStringRef;
use runloop::CFRunLoopSourceRef;

pub type CFFileDescriptorNativeDescriptor = c_int;

#[repr(C)]
pub struct __CFFileDescriptor(c_void);

pub type CFFileDescriptorRef = *mut __CFFileDescriptor;

/* Callback Reason Types */
pub const kCFFileDescriptorReadCallBack: CFOptionFlags  = 1 << 0;
pub const kCFFileDescriptorWriteCallBack: CFOptionFlags = 1 << 1;

pub type CFFileDescriptorCallBack = extern "C" fn (f: CFFileDescriptorRef, callBackTypes: CFOptionFlags, info: *mut c_void);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CFFileDescriptorContext {
    pub version: CFIndex,
    pub info: *mut c_void,
    pub retain: Option<extern "C" fn (info: *const c_void) -> *const c_void>,
    pub release: Option<extern "C" fn (info: *const c_void)>,
    pub copyDescription: Option<extern "C" fn (info: *const c_void) -> CFStringRef>,
}

extern {
    /*
     * CFFileDescriptor.h
     */
    pub fn CFFileDescriptorGetTypeID() -> CFTypeID;

    pub fn CFFileDescriptorCreate(allocator: CFAllocatorRef, fd: CFFileDescriptorNativeDescriptor, closeOnInvalidate: Boolean, callout: CFFileDescriptorCallBack, context: *const CFFileDescriptorContext) -> CFFileDescriptorRef;

    pub fn CFFileDescriptorGetNativeDescriptor(f: CFFileDescriptorRef) -> CFFileDescriptorNativeDescriptor;

    pub fn CFFileDescriptorGetContext(f: CFFileDescriptorRef, context: *mut CFFileDescriptorContext);

    pub fn CFFileDescriptorEnableCallBacks(f: CFFileDescriptorRef, callBackTypes: CFOptionFlags);
    pub fn CFFileDescriptorDisableCallBacks(f: CFFileDescriptorRef, callBackTypes: CFOptionFlags);

    pub fn CFFileDescriptorInvalidate(f: CFFileDescriptorRef);
    pub fn CFFileDescriptorIsValid(f: CFFileDescriptorRef) -> Boolean;

    pub fn CFFileDescriptorCreateRunLoopSource(allocator: CFAllocatorRef, f: CFFileDescriptorRef, order: CFIndex) -> CFRunLoopSourceRef;
}
