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

use std::ffi::{c_void, OsString};
use std::mem;
use std::os::unix::ffi::OsStringExt;

#[cfg(target_os = "macos")]
use std::{
    os::{
        raw::{c_long, c_uchar},
        unix::ffi::OsStrExt,
    },
    ptr::null_mut,
};

#[repr(C)]
struct PassWd {
    pw_name: *const c_void,
    pw_passwd: *const c_void,
    pw_uid: u32,
    pw_gid: u32,
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "bitrig",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    pw_change: isize,
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "bitrig",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    pw_class: *const c_void,
    pw_gecos: *const c_void,
    pw_dir: *const c_void,
    pw_shell: *const c_void,
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "bitrig",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    pw_expire: isize,
    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "bitrig",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    pw_fields: i32,
}

extern "system" {
    fn getpwuid_r(
        uid: u32,
        pwd: *mut PassWd,
        buf: *mut c_void,
        buflen: usize,
        result: *mut *mut PassWd,
    ) -> i32;
    fn geteuid() -> u32;
    fn gethostname(name: *mut c_void, len: usize) -> i32;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
#[link(name = "SystemConfiguration", kind = "framework")]
extern "system" {
    fn CFStringGetCString(
        the_string: *mut c_void,
        buffer: *mut u8,
        buffer_size: c_long,
        encoding: u32,
    ) -> c_uchar;
    fn CFStringGetLength(the_string: *mut c_void) -> c_long;
    fn CFStringGetMaximumSizeForEncoding(
        length: c_long,
        encoding: u32,
    ) -> c_long;
    fn SCDynamicStoreCopyComputerName(
        store: *mut c_void,
        encoding: *mut u32,
    ) -> *mut c_void;
    fn CFRelease(cf: *const c_void);
}

unsafe fn strlen(cs: *const c_void) -> usize {
    let mut len = 0;
    let mut cs: *const u8 = cs.cast();
    while *cs != 0 {
        len += 1;
        cs = cs.offset(1);
    }
    len
}

unsafe fn strlen_gecos(cs: *const c_void) -> usize {
    let mut len = 0;
    let mut cs: *const u8 = cs.cast();
    while *cs != 0 && *cs != b',' {
        len += 1;
        cs = cs.offset(1);
    }
    len
}

// Convert an OsString into a String
fn string_from_os(string: OsString) -> String {
    match string.into_string() {
        Ok(string) => string,
        Err(string) => string.to_string_lossy().to_string(),
    }
}

fn os_from_cstring_gecos(string: *const c_void) -> Option<OsString> {
    if string.is_null() {
        return None;
    }

    // Get a byte slice of the c string.
    let slice = unsafe {
        let length = strlen_gecos(string);
        if length == 0 {
            return None;
        }
        std::slice::from_raw_parts(string as *const u8, length)
    };

    // Turn byte slice into Rust String.
    Some(OsString::from_vec(slice.to_vec()))
}

fn os_from_cstring(string: *const c_void) -> OsString {
    if string.is_null() {
        return "".to_string().into();
    }

    // Get a byte slice of the c string.
    let slice = unsafe {
        let length = strlen(string);
        std::slice::from_raw_parts(string as *const u8, length)
    };

    // Turn byte slice into Rust String.
    OsString::from_vec(slice.to_vec())
}

#[cfg(target_os = "macos")]
fn os_from_cfstring(string: *mut c_void) -> OsString {
    if string.is_null() {
        return "".to_string().into();
    }

    unsafe {
        let len = CFStringGetLength(string);
        let capacity =
            CFStringGetMaximumSizeForEncoding(len, 134_217_984 /*UTF8*/) + 1;
        let mut out = Vec::with_capacity(capacity as usize);
        if CFStringGetCString(
            string,
            out.as_mut_ptr(),
            capacity,
            134_217_984, /*UTF8*/
        ) != 0
        {
            out.set_len(strlen(out.as_ptr().cast())); // Remove trailing NUL byte
            out.shrink_to_fit();
            CFRelease(string);
            OsString::from_vec(out)
        } else {
            CFRelease(string);
            "".to_string().into()
        }
    }
}

// This function must allocate, because a slice or Cow<OsStr> would still
// reference `passwd` which is dropped when this function returns.
#[inline(always)]
fn getpwuid(real: bool) -> OsString {
    const BUF_SIZE: usize = 16_384; // size from the man page
    let mut buffer = mem::MaybeUninit::<[u8; BUF_SIZE]>::uninit();
    let mut passwd = mem::MaybeUninit::<PassWd>::uninit();
    let mut _passwd = mem::MaybeUninit::<*mut PassWd>::uninit();

    // Get PassWd `struct`.
    let passwd = unsafe {
        let ret = getpwuid_r(
            geteuid(),
            passwd.as_mut_ptr(),
            buffer.as_mut_ptr() as *mut c_void,
            BUF_SIZE,
            _passwd.as_mut_ptr(),
        );

        if ret != 0 {
            return "Unknown".to_string().into();
        }

        let _passwd = _passwd.assume_init();

        if _passwd.is_null() {
            return "Unknown".to_string().into();
        }

        passwd.assume_init()
    };

    // Extract names.
    if real {
        let string = os_from_cstring_gecos(passwd.pw_gecos);
        let result = if let Some(string) = string {
            Ok(string)
        } else {
            Err(os_from_cstring(passwd.pw_name))
        };
        fancy_fallback_os(result)
    } else {
        os_from_cstring(passwd.pw_name)
    }
}

pub fn username() -> String {
    string_from_os(username_os())
}

pub fn username_os() -> OsString {
    getpwuid(false)
}

fn fancy_fallback(result: Result<&str, String>) -> String {
    let mut cap = true;
    let iter = match result {
        Ok(a) => a.chars(),
        Err(ref b) => b.chars(),
    };
    let mut new = String::new();
    for c in iter {
        match c {
            '.' | '-' | '_' => {
                new.push(' ');
                cap = true;
            }
            a => {
                if cap {
                    cap = false;
                    for i in a.to_uppercase() {
                        new.push(i);
                    }
                } else {
                    new.push(a);
                }
            }
        }
    }
    new
}

fn fancy_fallback_os(result: Result<OsString, OsString>) -> OsString {
    match result {
        Ok(success) => success,
        Err(fallback) => {
            let cs = match fallback.to_str() {
                Some(a) => Ok(a),
                None => Err(fallback.to_string_lossy().to_string()),
            };

            fancy_fallback(cs).into()
        }
    }
}

pub fn realname() -> String {
    string_from_os(realname_os())
}

pub fn realname_os() -> OsString {
    getpwuid(true)
}

#[cfg(not(target_os = "macos"))]
pub fn devicename_os() -> OsString {
    devicename().into()
}

#[cfg(not(target_os = "macos"))]
pub fn devicename() -> String {
    let mut distro = String::new();

    if let Ok(program) = std::fs::read_to_string("/etc/machine-info") {
        let program = program.into_bytes();

        distro.push_str(&String::from_utf8_lossy(&program));

        for i in distro.split('\n') {
            let mut j = i.split('=');

            if j.next() == Some("PRETTY_HOSTNAME") {
                if let Some(value) = j.next() {
                    return value.trim_matches('"').to_string();
                }
            }
        }
    }
    fancy_fallback(Err(hostname()))
}

#[cfg(target_os = "macos")]
pub fn devicename() -> String {
    string_from_os(devicename_os())
}

#[cfg(target_os = "macos")]
pub fn devicename_os() -> OsString {
    let out = os_from_cfstring(unsafe {
        SCDynamicStoreCopyComputerName(null_mut(), null_mut())
    });

    let computer = if out.as_bytes().is_empty() {
        Err(hostname_os())
    } else {
        Ok(out)
    };
    fancy_fallback_os(computer)
}

pub fn hostname() -> String {
    string_from_os(hostname_os())
}

pub fn hostname_os() -> OsString {
    // Maximum hostname length = 255, plus a NULL byte.
    let mut string = Vec::<u8>::with_capacity(256);
    unsafe {
        gethostname(string.as_mut_ptr() as *mut c_void, 255);
        string.set_len(strlen(string.as_ptr() as *const c_void));
    };
    OsString::from_vec(string)
}

#[cfg(target_os = "macos")]
fn distro_xml(data: String) -> Option<String> {
    let mut product_name = None;
    let mut user_visible_version = None;
    if let Some(start) = data.find("<dict>") {
        if let Some(end) = data.find("</dict>") {
            let mut set_product_name = false;
            let mut set_user_visible_version = false;
            for line in data[start + "<dict>".len()..end].lines() {
                let line = line.trim();
                if line.starts_with("<key>") {
                    match line["<key>".len()..].trim_end_matches("</key>") {
                        "ProductName" => set_product_name = true,
                        "ProductUserVisibleVersion" => {
                            set_user_visible_version = true
                        }
                        "ProductVersion" => {
                            if user_visible_version.is_none() {
                                set_user_visible_version = true
                            }
                        }
                        _ => {}
                    }
                } else if line.starts_with("<string>") {
                    if set_product_name {
                        product_name = Some(
                            line["<string>".len()..]
                                .trim_end_matches("</string>"),
                        );
                        set_product_name = false;
                    } else if set_user_visible_version {
                        user_visible_version = Some(
                            line["<string>".len()..]
                                .trim_end_matches("</string>"),
                        );
                        set_user_visible_version = false;
                    }
                }
            }
        }
    }
    if let Some(product_name) = product_name {
        if let Some(user_visible_version) = user_visible_version {
            Some(format!("{} {}", product_name, user_visible_version))
        } else {
            Some(product_name.to_string())
        }
    } else if let Some(user_visible_version) = user_visible_version {
        Some(format!("Mac OS (Unknown) {}", user_visible_version))
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
pub fn distro_os() -> Option<OsString> {
    distro().map(|a| a.into())
}

#[cfg(target_os = "macos")]
pub fn distro() -> Option<String> {
    if let Ok(data) = std::fs::read_to_string(
        "/System/Library/CoreServices/ServerVersion.plist",
    ) {
        distro_xml(data)
    } else if let Ok(data) = std::fs::read_to_string(
        "/System/Library/CoreServices/SystemVersion.plist",
    ) {
        distro_xml(data)
    } else {
        None
    }
}

#[cfg(not(target_os = "macos"))]
pub fn distro_os() -> Option<OsString> {
    distro().map(|a| a.into())
}

#[cfg(not(target_os = "macos"))]
pub fn distro() -> Option<String> {
    let mut distro = String::new();

    let program = std::fs::read_to_string("/etc/os-release")
        .ok()?
        .into_bytes();

    distro.push_str(&String::from_utf8_lossy(&program));

    let mut fallback = None;

    for i in distro.split('\n') {
        let mut j = i.split('=');

        match j.next()? {
            "PRETTY_NAME" => {
                return Some(j.next()?.trim_matches('"').to_string())
            }
            "NAME" => fallback = Some(j.next()?.trim_matches('"').to_string()),
            _ => {}
        }
    }

    fallback
}

#[cfg(target_os = "macos")]
#[inline(always)]
pub const fn desktop_env() -> DesktopEnv {
    DesktopEnv::Aqua
}

#[cfg(not(target_os = "macos"))]
#[inline(always)]
pub fn desktop_env() -> DesktopEnv {
    match std::env::var_os("DESKTOP_SESSION")
        .map(|env| env.to_string_lossy().to_string())
    {
        Some(env_orig) => {
            let env = env_orig.to_uppercase();

            if env.contains("GNOME") {
                DesktopEnv::Gnome
            } else if env.contains("LXDE") {
                DesktopEnv::Lxde
            } else if env.contains("OPENBOX") {
                DesktopEnv::Openbox
            } else if env.contains("I3") {
                DesktopEnv::I3
            } else if env.contains("UBUNTU") {
                DesktopEnv::Ubuntu
            } else if env.contains("PLASMA5") {
                DesktopEnv::Kde
            } else {
                DesktopEnv::Unknown(env_orig)
            }
        }
        // TODO: Other Linux Desktop Environments
        None => DesktopEnv::Unknown("Unknown".to_string()),
    }
}

#[cfg(target_os = "macos")]
#[inline(always)]
pub const fn platform() -> Platform {
    Platform::MacOS
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "bitrig",
    target_os = "openbsd",
    target_os = "netbsd"
)))]
#[inline(always)]
pub const fn platform() -> Platform {
    Platform::Linux
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "bitrig",
    target_os = "openbsd",
    target_os = "netbsd"
))]
#[inline(always)]
pub const fn platform() -> Platform {
    Platform::Bsd
}

struct LangIter {
    array: String,
    index: Option<bool>,
}

impl Iterator for LangIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index? {
            self.index = Some(false);
            let mut temp = self.array.split('-').next()?.to_string();
            std::mem::swap(&mut temp, &mut self.array);
            Some(temp)
        } else {
            self.index = None;
            let mut temp = String::new();
            std::mem::swap(&mut temp, &mut self.array);
            Some(temp)
        }
    }
}

#[inline(always)]
pub fn lang() -> impl Iterator<Item = String> {
    let array = std::env::var("LANG")
        .unwrap_or_default()
        .split('.')
        .next()
        .unwrap_or("en_US")
        .to_string()
        .replace("_", "-");
    LangIter {
        array,
        index: Some(true),
    }
}
