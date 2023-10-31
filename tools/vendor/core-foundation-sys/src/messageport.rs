// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::c_void;

use base::{CFAllocatorRef, CFIndex, CFTypeID, Boolean};
use data::CFDataRef;
use date::CFTimeInterval;
use runloop::CFRunLoopSourceRef;
use string::CFStringRef;

#[repr(C)]
#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct CFMessagePortContext {
    pub version: CFIndex,
    pub info: *mut c_void,
    pub retain: Option<unsafe extern fn(info: *const c_void) -> *const c_void>,
    pub release: Option<unsafe extern fn(info: *const c_void)>,
    pub copyDescription: Option<unsafe extern fn(info: *const c_void)
        -> CFStringRef>,
}

pub type CFMessagePortCallBack = Option<
    unsafe extern fn(local: CFMessagePortRef,
                     msgid: i32,
                     data: CFDataRef,
                     info: *mut c_void) -> CFDataRef>;

pub type CFMessagePortInvalidationCallBack = Option<
    unsafe extern "C" fn(ms: CFMessagePortRef, info: *mut c_void)>;

#[repr(C)]
pub struct __CFMessagePort(c_void);
pub type CFMessagePortRef = *mut __CFMessagePort;

extern {
    /*
     * CFMessagePort.h
     */
    pub fn CFMessagePortGetTypeID() -> CFTypeID;
    pub fn CFMessagePortCreateLocal(allocator: CFAllocatorRef,
                                    name: CFStringRef,
                                    callout: CFMessagePortCallBack,
                                    context: *const CFMessagePortContext,
                                    shouldFreeInfo: *mut Boolean)
        -> CFMessagePortRef;
    pub fn CFMessagePortCreateRemote(allocator: CFAllocatorRef,
                                     name: CFStringRef) -> CFMessagePortRef;
    pub fn CFMessagePortIsRemote(ms: CFMessagePortRef) -> Boolean;
    pub fn CFMessagePortGetName(ms: CFMessagePortRef) -> CFStringRef;
    pub fn CFMessagePortSetName(ms: CFMessagePortRef, newName: CFStringRef)
        -> Boolean;
    pub fn CFMessagePortGetContext(ms: CFMessagePortRef,
                                   context: *mut CFMessagePortContext);
    pub fn CFMessagePortInvalidate(ms: CFMessagePortRef);
    pub fn CFMessagePortIsValid(ms: CFMessagePortRef) -> Boolean;
    pub fn CFMessagePortGetInvalidationCallBack(ms: CFMessagePortRef)
        -> CFMessagePortInvalidationCallBack;
    pub fn CFMessagePortSetInvalidationCallBack(ms: CFMessagePortRef,
                                                callout: CFMessagePortInvalidationCallBack);
    pub fn CFMessagePortSendRequest(remote: CFMessagePortRef, msgid: i32,
                                    data: CFDataRef,
                                    sendTimeout: CFTimeInterval,
                                    rcvTimeout: CFTimeInterval,
                                    replyMode: CFStringRef,
                                    returnData: *mut CFDataRef) -> i32;
    pub fn CFMessagePortCreateRunLoopSource(allocator: CFAllocatorRef,
                                            local: CFMessagePortRef,
                                            order: CFIndex)
        -> CFRunLoopSourceRef;
    // CFMessagePortSetDispatchQueue
}
