// WhoAmI
// Copyright © 2017-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
//  - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
//  - MIT License (https://mit-license.org/)
//  - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{DesktopEnv, Platform};

use std::convert::TryInto;
use std::ffi::OsString;
use std::os::raw::{c_char, c_int, c_uchar, c_ulong, c_ushort};
use std::os::windows::ffi::OsStringExt;
use std::ptr;

#[repr(C)]
struct OsVersionInfoEx {
    os_version_info_size: c_ulong,
    major_version: c_ulong,
    minor_version: c_ulong,
    build_number: c_ulong,
    platform_id: c_ulong,
    sz_csd_version: [u16; 128],
    service_pack_major: c_ushort,
    service_pack_minor: c_ushort,
    suite_mask: c_ushort,
    product_type: c_uchar,
    reserved: c_uchar,
}

#[allow(unused)]
#[repr(C)]
enum ExtendedNameFormat {
    Unknown,          // Nothing
    FullyQualifiedDN, // Nothing
    SamCompatible,    // Hostname Followed By Username
    Display,          // Full Name
    UniqueId,         // Nothing
    Canonical,        // Nothing
    UserPrincipal,    // Nothing
    CanonicalEx,      // Nothing
    ServicePrincipal, // Nothing
    DnsDomain,        // Nothing
    GivenName,        // Nothing
    Surname,          // Nothing
}

#[allow(unused)]
#[repr(C)]
enum ComputerNameFormat {
    NetBIOS,                   // Same as GetComputerNameW
    DnsHostname,               // Fancy Name
    DnsDomain,                 // Nothing
    DnsFullyQualified,         // Fancy Name with, for example, .com
    PhysicalNetBIOS,           // Same as GetComputerNameW
    PhysicalDnsHostname,       // Same as GetComputerNameW
    PhysicalDnsDomain,         // Nothing
    PhysicalDnsFullyQualified, // Fancy Name with, for example, .com
    Max,
}

#[link(name = "secur32")]
extern "system" {
    fn GetLastError() -> c_ulong;
    fn GetUserNameExW(
        a: ExtendedNameFormat,
        b: *mut c_char,
        c: *mut c_ulong,
    ) -> c_uchar;
    fn GetUserNameW(a: *mut c_char, b: *mut c_ulong) -> c_int;
    fn GetComputerNameExW(
        a: ComputerNameFormat,
        b: *mut c_char,
        c: *mut c_ulong,
    ) -> c_int;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetUserPreferredUILanguages(
        dw_flags: c_ulong,
        pul_num_languages: *mut c_ulong,
        pwsz_languages_buffer: *mut u16,
        pcch_languages_buffer: *mut c_ulong,
    ) -> c_int;
}

// Convert an OsString into a String
fn string_from_os(string: OsString) -> String {
    match string.into_string() {
        Ok(string) => string,
        Err(string) => string.to_string_lossy().to_string(),
    }
}

pub fn username() -> String {
    string_from_os(username_os())
}

pub fn username_os() -> OsString {
    // Step 1. Retreive the entire length of the username
    let mut size = 0;
    let success = unsafe {
        // Ignore error, we know that it will be ERROR_INSUFFICIENT_BUFFER
        GetUserNameW(ptr::null_mut(), &mut size) == 0
    };
    assert!(success);

    // Step 2. Allocate memory to put the Windows (UTF-16) string.
    let mut name: Vec<u16> =
        Vec::with_capacity(size.try_into().unwrap_or(std::usize::MAX));
    size = name.capacity().try_into().unwrap_or(std::u32::MAX);
    let orig_size = size;
    let fail =
        unsafe { GetUserNameW(name.as_mut_ptr().cast(), &mut size) == 0 };
    if fail {
        return "unknown".to_string().into();
    }
    debug_assert_eq!(orig_size, size);
    unsafe {
        name.set_len(size.try_into().unwrap_or(std::usize::MAX));
    }
    let terminator = name.pop(); // Remove Trailing Null
    debug_assert_eq!(terminator, Some(0u16));

    // Step 3. Convert to Rust String
    OsString::from_wide(&name)
}

#[inline(always)]
pub fn realname() -> String {
    string_from_os(realname_os())
}

#[inline(always)]
pub fn realname_os() -> OsString {
    // Step 1. Retrieve the entire length of the username
    let mut buf_size = 0;
    let success = unsafe {
        GetUserNameExW(
            ExtendedNameFormat::Display,
            ptr::null_mut(),
            &mut buf_size,
        ) == 0
    };
    assert!(success);
    match unsafe { GetLastError() } {
		0x00EA /* more data */ => { /* Success, continue */ }
		_ /* network error or none mapped */ => {
			// Fallback to username
			return username_os();
		}
	}

    // Step 2. Allocate memory to put the Windows (UTF-16) string.
    let mut name: Vec<u16> =
        Vec::with_capacity(buf_size.try_into().unwrap_or(std::usize::MAX));
    let mut name_len = name.capacity().try_into().unwrap_or(std::u32::MAX);
    let fail = unsafe {
        GetUserNameExW(
            ExtendedNameFormat::Display,
            name.as_mut_ptr().cast(),
            &mut name_len,
        ) == 0
    };
    if fail {
        return "Unknown".to_string().into();
    }
    debug_assert_eq!(buf_size, name_len + 1);
    unsafe {
        name.set_len(name_len.try_into().unwrap_or(std::usize::MAX));
    }

    // Step 3. Convert to Rust String
    OsString::from_wide(&name)
}

#[inline(always)]
pub fn devicename() -> String {
    string_from_os(devicename_os())
}

#[inline(always)]
pub fn devicename_os() -> OsString {
    // Step 1. Retreive the entire length of the device name
    let mut size = 0;
    let success = unsafe {
        // Ignore error, we know that it will be ERROR_INSUFFICIENT_BUFFER
        GetComputerNameExW(
            ComputerNameFormat::DnsHostname,
            ptr::null_mut(),
            &mut size,
        ) == 0
    };
    assert!(success);

    // Step 2. Allocate memory to put the Windows (UTF-16) string.
    let mut name: Vec<u16> =
        Vec::with_capacity(size.try_into().unwrap_or(std::usize::MAX));
    size = name.capacity().try_into().unwrap_or(std::u32::MAX);
    let fail = unsafe {
        GetComputerNameExW(
            ComputerNameFormat::DnsHostname,
            name.as_mut_ptr().cast(),
            &mut size,
        ) == 0
    };
    if fail {
        return "Unknown".to_string().into();
    }
    unsafe {
        name.set_len(size.try_into().unwrap_or(std::usize::MAX));
    }

    // Step 3. Convert to Rust String
    OsString::from_wide(&name)
}

pub fn hostname() -> String {
    string_from_os(hostname_os())
}

pub fn hostname_os() -> OsString {
    // Step 1. Retreive the entire length of the username
    let mut size = 0;
    let fail = unsafe {
        // Ignore error, we know that it will be ERROR_INSUFFICIENT_BUFFER
        GetComputerNameExW(
            ComputerNameFormat::NetBIOS,
            ptr::null_mut(),
            &mut size,
        ) == 0
    };
    debug_assert!(fail);

    // Step 2. Allocate memory to put the Windows (UTF-16) string.
    let mut name: Vec<u16> =
        Vec::with_capacity(size.try_into().unwrap_or(std::usize::MAX));
    size = name.capacity().try_into().unwrap_or(std::u32::MAX);
    let fail = unsafe {
        GetComputerNameExW(
            ComputerNameFormat::NetBIOS,
            name.as_mut_ptr().cast(),
            &mut size,
        ) == 0
    };
    if fail {
        return "localhost".to_string().into();
    }
    unsafe {
        name.set_len(size.try_into().unwrap_or(std::usize::MAX));
    }

    // Step 3. Convert to Rust String
    OsString::from_wide(&name)
}

pub fn distro_os() -> Option<OsString> {
    distro().map(|a| a.into())
}

pub fn distro() -> Option<String> {
    // Due to MingW Limitations, we must dynamically load ntdll.dll
    extern "system" {
        fn LoadLibraryExW(
            filename: *mut u16,
            hfile: isize,
            dwflags: u32,
        ) -> isize;
        fn FreeLibrary(hmodule: isize) -> i32;
        fn GetProcAddress(
            hmodule: isize,
            procname: *mut u8,
        ) -> *mut std::os::raw::c_void;
    }

    let mut path = "ntdll.dll\0".encode_utf16().collect::<Vec<u16>>();
    let path = path.as_mut_ptr();

    let inst = unsafe { LoadLibraryExW(path, 0, 0x00000800) };

    let mut path = "RtlGetVersion\0".bytes().collect::<Vec<u8>>();
    let path = path.as_mut_ptr();
    let func = unsafe { GetProcAddress(inst, path) };

    if func.is_null() {
        return Some("Windows (Unknown)".to_string());
    }

    let get_version: unsafe extern "system" fn(a: *mut OsVersionInfoEx) -> u32 =
        unsafe { std::mem::transmute(func) };

    let mut version = std::mem::MaybeUninit::<OsVersionInfoEx>::zeroed();

    let version = unsafe {
        (*version.as_mut_ptr()).os_version_info_size =
            std::mem::size_of::<OsVersionInfoEx>() as u32;
        get_version(version.as_mut_ptr());
        FreeLibrary(inst);
        version.assume_init()
    };

    let product = match version.product_type {
        1 => "Workstation",
        2 => "Domain Controller",
        3 => "Server",
        _ => "Unknown",
    };

    let out = format!(
        "Windows {}.{}.{} ({})",
        version.major_version,
        version.minor_version,
        version.build_number,
        product
    );

    Some(out)
}

#[inline(always)]
pub const fn desktop_env() -> DesktopEnv {
    DesktopEnv::Windows
}

#[inline(always)]
pub const fn platform() -> Platform {
    Platform::Windows
}

struct LangIter {
    array: Vec<String>,
    index: usize,
}

impl Iterator for LangIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.array.get(self.index) {
            self.index += 1;
            Some(value.to_string())
        } else {
            None
        }
    }
}

#[inline(always)]
pub fn lang() -> impl Iterator<Item = String> {
    let mut num_languages = 0;
    let mut buffer_size = 0;
    let mut buffer;

    unsafe {
        assert_ne!(
            GetUserPreferredUILanguages(
                0x08, /* MUI_LANGUAGE_NAME */
                &mut num_languages,
                std::ptr::null_mut(), // List of languages.
                &mut buffer_size,
            ),
            0
        );

        buffer = Vec::with_capacity(buffer_size as usize);

        assert_ne!(
            GetUserPreferredUILanguages(
                0x08, /* MUI_LANGUAGE_NAME */
                &mut num_languages,
                buffer.as_mut_ptr(), // List of languages.
                &mut buffer_size,
            ),
            0
        );

        buffer.set_len(buffer_size as usize);
    }

    // We know it ends in two null characters.
    buffer.pop();
    buffer.pop();

    //
    let array = String::from_utf16_lossy(&buffer)
        .split('\0')
        .map(|x| x.to_string())
        .collect();
    let index = 0;

    LangIter { array, index }
}
