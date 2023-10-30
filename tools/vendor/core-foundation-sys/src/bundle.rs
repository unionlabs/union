// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::c_void;

use base::{CFTypeID, CFAllocatorRef};
use url::CFURLRef;
use dictionary::CFDictionaryRef;
use string::CFStringRef;

#[repr(C)]
pub struct __CFBundle(c_void);

pub type CFBundleRef = *mut __CFBundle;

extern {
    /*
     * CFBundle.h
     */
    pub fn CFBundleCreate(allocator: CFAllocatorRef, bundleURL: CFURLRef) -> CFBundleRef;

    pub fn CFBundleGetBundleWithIdentifier(bundleID: CFStringRef) -> CFBundleRef;
    pub fn CFBundleGetFunctionPointerForName(bundle: CFBundleRef, function_name: CFStringRef) -> *const c_void;
    pub fn CFBundleGetMainBundle() -> CFBundleRef;
    pub fn CFBundleGetInfoDictionary(bundle: CFBundleRef) -> CFDictionaryRef;

    pub fn CFBundleGetTypeID() -> CFTypeID;
    pub fn CFBundleCopyExecutableURL(bundle: CFBundleRef) -> CFURLRef;
    pub fn CFBundleCopyPrivateFrameworksURL(bundle: CFBundleRef) -> CFURLRef;
    pub fn CFBundleCopySharedSupportURL(bundle: CFBundleRef) -> CFURLRef;
    pub fn CFBundleCopyBundleURL(bundle: CFBundleRef) -> CFURLRef;
    pub fn CFBundleCopyResourcesDirectoryURL(bundle: CFBundleRef) -> CFURLRef;
}
