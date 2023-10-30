// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::c_void;

use base::{CFOptionFlags, CFIndex, CFAllocatorRef, Boolean, CFTypeID, CFTypeRef, SInt32};
use data::CFDataRef;
use array::CFArrayRef;
use dictionary::CFDictionaryRef;
use string::{CFStringRef, CFStringEncoding};
use error::CFErrorRef;

#[repr(C)]
pub struct __CFURL(c_void);

pub type CFURLRef = *const __CFURL;

pub type CFURLBookmarkCreationOptions = CFOptionFlags;
pub type CFURLBookmarkResolutionOptions = CFOptionFlags;
pub type CFURLBookmarkFileCreationOptions = CFOptionFlags;

pub type CFURLPathStyle = CFIndex;

/* typedef CF_ENUM(CFIndex, CFURLPathStyle) */
pub const kCFURLPOSIXPathStyle: CFURLPathStyle   = 0;
pub const kCFURLHFSPathStyle: CFURLPathStyle     = 1;
pub const kCFURLWindowsPathStyle: CFURLPathStyle = 2;

pub static kCFURLBookmarkCreationPreferFileIDResolutionMask: CFURLBookmarkCreationOptions =
    (1u32 << 8) as usize;
pub static kCFURLBookmarkCreationMinimalBookmarkMask: CFURLBookmarkCreationOptions =
    (1u32 << 9) as usize;
pub static kCFURLBookmarkCreationSuitableForBookmarkFile: CFURLBookmarkCreationOptions =
    (1u32 << 10) as usize;
pub static kCFURLBookmarkCreationWithSecurityScope: CFURLBookmarkCreationOptions =
    (1u32 << 11) as usize;
pub static kCFURLBookmarkCreationSecurityScopeAllowOnlyReadAccess: CFURLBookmarkCreationOptions =
    (1u32 << 12) as usize;

// TODO: there are a lot of missing keys and constants. Add if you are bored or need them.

extern {
    /*
     * CFURL.h
     */

    /* Common File System Resource Keys */
    pub static kCFURLAttributeModificationDateKey: CFStringRef;
    pub static kCFURLContentAccessDateKey: CFStringRef;
    pub static kCFURLContentModificationDateKey: CFStringRef;
    pub static kCFURLCreationDateKey: CFStringRef;
    pub static kCFURLFileResourceIdentifierKey: CFStringRef;
    pub static kCFURLFileSecurityKey: CFStringRef;
    pub static kCFURLHasHiddenExtensionKey: CFStringRef;
    pub static kCFURLIsDirectoryKey: CFStringRef;
    pub static kCFURLIsExecutableKey: CFStringRef;
    pub static kCFURLIsHiddenKey: CFStringRef;
    pub static kCFURLIsPackageKey: CFStringRef;
    pub static kCFURLIsReadableKey: CFStringRef;
    pub static kCFURLIsRegularFileKey: CFStringRef;
    pub static kCFURLIsSymbolicLinkKey: CFStringRef;
    pub static kCFURLIsSystemImmutableKey: CFStringRef;
    pub static kCFURLIsUserImmutableKey: CFStringRef;
    pub static kCFURLIsVolumeKey: CFStringRef;
    pub static kCFURLIsWritableKey: CFStringRef;
    pub static kCFURLLabelNumberKey: CFStringRef;
    pub static kCFURLLinkCountKey: CFStringRef;
    pub static kCFURLLocalizedLabelKey: CFStringRef;
    pub static kCFURLLocalizedNameKey: CFStringRef;
    pub static kCFURLLocalizedTypeDescriptionKey: CFStringRef;
    pub static kCFURLNameKey: CFStringRef;
    pub static kCFURLParentDirectoryURLKey: CFStringRef;
    pub static kCFURLPreferredIOBlockSizeKey: CFStringRef;
    pub static kCFURLTypeIdentifierKey: CFStringRef;
    pub static kCFURLVolumeIdentifierKey: CFStringRef;
    pub static kCFURLVolumeURLKey: CFStringRef;

    #[cfg(feature="mac_os_10_8_features")]
    #[cfg_attr(feature = "mac_os_10_7_support", linkage = "extern_weak")]
    pub static kCFURLIsExcludedFromBackupKey: CFStringRef;
    pub static kCFURLFileResourceTypeKey: CFStringRef;

    /* Creating a CFURL */
    pub fn CFURLCopyAbsoluteURL(anURL: CFURLRef) -> CFURLRef;
    pub fn CFURLCreateAbsoluteURLWithBytes(allocator: CFAllocatorRef, relativeURLBytes: *const u8, length: CFIndex, encoding: CFStringEncoding, baseURL: CFURLRef, useCompatibilityMode: Boolean) -> CFURLRef;
    pub fn CFURLCreateByResolvingBookmarkData(allocator: CFAllocatorRef, bookmark: CFDataRef, options: CFURLBookmarkResolutionOptions, relativeToURL: CFURLRef, resourcePropertiesToInclude: CFArrayRef, isStale: *mut Boolean, error: *mut CFErrorRef) -> CFURLRef;
    //fn CFURLCreateCopyAppendingPathComponent
    //fn CFURLCreateCopyAppendingPathExtension
    //fn CFURLCreateCopyDeletingLastPathComponent
    //fn CFURLCreateCopyDeletingPathExtension
    pub fn CFURLCreateFilePathURL(allocator: CFAllocatorRef, url: CFURLRef, error: *mut CFErrorRef) -> CFURLRef;
    //fn CFURLCreateFileReferenceURL
    pub fn CFURLCreateFromFileSystemRepresentation(allocator: CFAllocatorRef, buffer: *const u8, bufLen: CFIndex, isDirectory: Boolean) -> CFURLRef;
    //fn CFURLCreateFromFileSystemRepresentationRelativeToBase
    //fn CFURLCreateFromFSRef
    pub fn CFURLCreateWithBytes(allocator: CFAllocatorRef, URLBytes: *const u8, length: CFIndex, encoding: CFStringEncoding, baseURL: CFURLRef) -> CFURLRef;
    pub fn CFURLCreateWithFileSystemPath(allocator: CFAllocatorRef, filePath: CFStringRef, pathStyle: CFURLPathStyle, isDirectory: Boolean) -> CFURLRef;
    pub fn CFURLCreateWithFileSystemPathRelativeToBase(allocator: CFAllocatorRef, filePath: CFStringRef, pathStyle: CFURLPathStyle, isDirectory: Boolean, baseURL: CFURLRef) -> CFURLRef;
    //fn CFURLCreateWithString(allocator: CFAllocatorRef, urlString: CFStringRef,
    //                         baseURL: CFURLRef) -> CFURLRef;

    /* Accessing the Parts of a URL */
    pub fn CFURLCanBeDecomposed(anURL: CFURLRef) -> Boolean;
    pub fn CFURLCopyFileSystemPath(anURL: CFURLRef, pathStyle: CFURLPathStyle) -> CFStringRef;
    pub fn CFURLCopyFragment(anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef;
    pub fn CFURLCopyHostName(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLCopyLastPathComponent(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLCopyNetLocation(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLCopyParameterString(anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef;
    pub fn CFURLCopyPassword(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLCopyPath(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLCopyPathExtension(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLCopyQueryString(anURL: CFURLRef, charactersToLeaveEscaped: CFStringRef) -> CFStringRef;
    pub fn CFURLCopyResourceSpecifier(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLCopyScheme(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLCopyStrictPath(anURL: CFURLRef, isAbsolute: *mut Boolean) -> CFStringRef;
    pub fn CFURLCopyUserName(anURL: CFURLRef) -> CFStringRef;
    pub fn CFURLGetPortNumber(anURL: CFURLRef) -> SInt32;
    pub fn CFURLHasDirectoryPath(anURL: CFURLRef) -> Boolean;

    /* Converting URLs to Other Representations */
    //fn CFURLCreateData(allocator: CFAllocatorRef, url: CFURLRef,
    //                   encoding: CFStringEncoding, escapeWhitespace: bool) -> CFDataRef;
    //fn CFURLCreateStringByAddingPercentEscapes
    //fn CFURLCreateStringByReplacingPercentEscapes
    //fn CFURLCreateStringByReplacingPercentEscapesUsingEncoding
    pub fn CFURLGetFileSystemRepresentation(anURL: CFURLRef, resolveAgainstBase: Boolean, buffer: *mut u8, maxBufLen: CFIndex) -> Boolean;

    //fn CFURLGetFSRef
    pub fn CFURLGetString(anURL: CFURLRef) -> CFStringRef;

    /* Getting URL Properties */
    //fn CFURLGetBaseURL(anURL: CFURLRef) -> CFURLRef;
    pub fn CFURLGetBytes(anURL: CFURLRef, buffer: *mut u8, bufferLength: CFIndex) -> CFIndex;
    //fn CFURLGetByteRangeForComponent
    pub fn CFURLGetTypeID() -> CFTypeID;
    //fn CFURLResourceIsReachable

    /* Getting and Setting File System Resource Properties */
    pub fn CFURLClearResourcePropertyCache(url: CFURLRef);
    //fn CFURLClearResourcePropertyCacheForKey
    //fn CFURLCopyResourcePropertiesForKeys
    //fn CFURLCopyResourcePropertyForKey
    pub fn CFURLCreateResourcePropertiesForKeysFromBookmarkData(allocator: CFAllocatorRef, resourcePropertiesToReturn: CFArrayRef, bookmark: CFDataRef) -> CFDictionaryRef;
    pub fn CFURLCreateResourcePropertyForKeyFromBookmarkData(allocator: CFAllocatorRef, resourcePropertyKey: CFStringRef, bookmark: CFDataRef) -> CFTypeRef;
    //fn CFURLSetResourcePropertiesForKeys
    pub fn CFURLSetResourcePropertyForKey(url: CFURLRef, key: CFStringRef, value: CFTypeRef, error: *mut CFErrorRef) -> Boolean;
    //fn CFURLSetTemporaryResourcePropertyForKey

    /* Working with Bookmark Data */
    pub fn CFURLCreateBookmarkData(allocator: CFAllocatorRef, url: CFURLRef, options: CFURLBookmarkCreationOptions, resourcePropertiesToInclude: CFArrayRef, relativeToURL: CFURLRef, error: *mut CFErrorRef) -> CFDataRef;
    pub fn CFURLCreateBookmarkDataFromAliasRecord(allocator: CFAllocatorRef, aliasRecordDataRef: CFDataRef) -> CFDataRef;
    pub fn CFURLCreateBookmarkDataFromFile(allocator: CFAllocatorRef, fileURL: CFURLRef, errorRef: *mut CFErrorRef) -> CFDataRef;
    pub fn CFURLWriteBookmarkDataToFile(bookmarkRef: CFDataRef, fileURL: CFURLRef, options: CFURLBookmarkFileCreationOptions, errorRef: *mut CFErrorRef) -> Boolean;
    pub fn CFURLStartAccessingSecurityScopedResource(url: CFURLRef) -> Boolean;
    pub fn CFURLStopAccessingSecurityScopedResource(url: CFURLRef);
}

#[test]
#[cfg(feature="mac_os_10_8_features")]
fn can_see_excluded_from_backup_key() {
    let _ = unsafe { kCFURLIsExcludedFromBackupKey };
}
