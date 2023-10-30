// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cmp::Ordering;
use std::os::raw::{c_uint, c_void, c_int};
use string::CFStringRef;

pub type Boolean = u8;
pub type mach_port_t = c_uint;
pub type CFAllocatorRef = *const c_void;
pub type CFNullRef = *const c_void;
pub type CFTypeRef = *const c_void;
pub type OSStatus = i32;
pub type SInt32 = c_int;
pub type CFTypeID = usize;
pub type CFOptionFlags = usize;
pub type CFHashCode = usize;
pub type CFIndex = isize;

#[repr(isize)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CFComparisonResult {
    LessThan = -1,
    EqualTo = 0,
    GreaterThan = 1,
}

impl Into<Ordering> for CFComparisonResult {
    fn into(self) -> Ordering {
        match self {
            CFComparisonResult::LessThan => Ordering::Less,
            CFComparisonResult::EqualTo => Ordering::Equal,
            CFComparisonResult::GreaterThan => Ordering::Greater
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CFRange {
    pub location: CFIndex,
    pub length: CFIndex
}

// for back-compat
impl CFRange {
    pub fn init(location: CFIndex, length: CFIndex) -> CFRange {
        CFRange {
            location: location,
            length: length,
        }
    }
}

pub type CFAllocatorRetainCallBack = extern "C" fn(info: *mut c_void) -> *mut c_void;
pub type CFAllocatorReleaseCallBack = extern "C" fn(info: *mut c_void);
pub type CFAllocatorCopyDescriptionCallBack = extern "C" fn(info: *mut c_void) -> CFStringRef;
pub type CFAllocatorAllocateCallBack = extern "C" fn(allocSize: CFIndex, hint: CFOptionFlags, info: *mut c_void) -> *mut c_void;
pub type CFAllocatorReallocateCallBack = extern "C" fn(ptr: *mut c_void, newsize: CFIndex, hint: CFOptionFlags, info: *mut c_void) -> *mut c_void;
pub type CFAllocatorDeallocateCallBack = extern "C" fn(ptr: *mut c_void, info: *mut c_void);
pub type CFAllocatorPreferredSizeCallBack = extern "C" fn(size: CFIndex, hint: CFOptionFlags, info: *mut c_void) -> CFIndex;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CFAllocatorContext {
    pub version: CFIndex,
    pub info: *mut c_void,
    pub retain: Option<CFAllocatorRetainCallBack>,
    pub release: Option<CFAllocatorReleaseCallBack>,
    pub copyDescription: Option<CFAllocatorCopyDescriptionCallBack>,
    pub allocate: Option<CFAllocatorAllocateCallBack>,
    pub reallocate: Option<CFAllocatorReallocateCallBack>,
    pub deallocate: Option<CFAllocatorDeallocateCallBack>,
    pub preferredSize: Option<CFAllocatorPreferredSizeCallBack>
}

/// Trait for all types which are Core Foundation reference types.
pub trait TCFTypeRef {
    fn as_void_ptr(&self) -> *const c_void;

    unsafe fn from_void_ptr(ptr: *const c_void) -> Self;
}

impl<T> TCFTypeRef for *const T {
    fn as_void_ptr(&self) -> *const c_void {
        (*self) as *const c_void
    }

    unsafe fn from_void_ptr(ptr: *const c_void) -> Self {
        ptr as *const T
    }
}

impl<T> TCFTypeRef for *mut T {
    fn as_void_ptr(&self) -> *const c_void {
        (*self) as *const T as *const c_void
    }

    unsafe fn from_void_ptr(ptr: *const c_void) -> Self {
        ptr as *const T as *mut T
    }
}

/// Constant used by some functions to indicate failed searches.
pub static kCFNotFound: CFIndex = -1;

extern {
    /*
     * CFBase.h
     */

    /* CFAllocator Reference */

    pub static kCFAllocatorDefault: CFAllocatorRef;
    pub static kCFAllocatorSystemDefault: CFAllocatorRef;
    pub static kCFAllocatorMalloc: CFAllocatorRef;
    pub static kCFAllocatorMallocZone: CFAllocatorRef;
    pub static kCFAllocatorNull: CFAllocatorRef;
    pub static kCFAllocatorUseContext: CFAllocatorRef;

    pub fn CFAllocatorCreate(allocator: CFAllocatorRef, context: *mut CFAllocatorContext) -> CFAllocatorRef;
    pub fn CFAllocatorAllocate(allocator: CFAllocatorRef, size: CFIndex, hint: CFOptionFlags) -> *mut c_void;
    pub fn CFAllocatorDeallocate(allocator: CFAllocatorRef, ptr: *mut c_void);
    pub fn CFAllocatorGetPreferredSizeForSize(allocator: CFAllocatorRef, size: CFIndex, hint: CFOptionFlags) -> CFIndex;
    pub fn CFAllocatorReallocate(allocator: CFAllocatorRef, ptr: *mut c_void, newsize: CFIndex, hint: CFOptionFlags) -> *mut c_void;
    pub fn CFAllocatorGetDefault() -> CFAllocatorRef;
    pub fn CFAllocatorSetDefault(allocator: CFAllocatorRef);
    pub fn CFAllocatorGetContext(allocator: CFAllocatorRef, context: *mut CFAllocatorContext);
    pub fn CFAllocatorGetTypeID() -> CFTypeID;

    /* CFNull Reference */

    pub static kCFNull: CFNullRef;

    /* CFType Reference */

    //fn CFCopyTypeIDDescription
    //fn CFGetAllocator
    pub fn CFCopyDescription(cf: CFTypeRef) -> CFStringRef;
    pub fn CFEqual(cf1: CFTypeRef, cf2: CFTypeRef) -> Boolean;
    pub fn CFGetRetainCount(cf: CFTypeRef) -> CFIndex;
    pub fn CFGetTypeID(cf: CFTypeRef) -> CFTypeID;
    pub fn CFHash(cf: CFTypeRef) -> CFHashCode;
    //fn CFMakeCollectable
    pub fn CFRelease(cf: CFTypeRef);
    pub fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    pub fn CFShow(obj: CFTypeRef);

    /* Base Utilities Reference */
    // N.B. Some things missing here.
}
