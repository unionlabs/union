// This module defines safe wrappers around memchr (POSIX) and memrchr (GNU
// extension).

#![allow(dead_code)]

use libc::{c_int, c_void, size_t};

pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    // SAFETY: This is safe to call since all pointers are valid.
    let p = unsafe {
        libc::memchr(
            haystack.as_ptr() as *const c_void,
            needle as c_int,
            haystack.len() as size_t,
        )
    };
    if p.is_null() {
        None
    } else {
        Some(p as usize - (haystack.as_ptr() as usize))
    }
}

// memrchr is a GNU extension. We know it's available on Linux at least.
#[cfg(target_os = "linux")]
pub fn memrchr(needle: u8, haystack: &[u8]) -> Option<usize> {
    // GNU's memrchr() will - unlike memchr() - error if haystack is empty.
    if haystack.is_empty() {
        return None;
    }
    // SAFETY: This is safe to call since all pointers are valid.
    let p = unsafe {
        libc::memrchr(
            haystack.as_ptr() as *const c_void,
            needle as c_int,
            haystack.len() as size_t,
        )
    };
    if p.is_null() {
        None
    } else {
        Some(p as usize - (haystack.as_ptr() as usize))
    }
}
