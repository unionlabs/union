#![allow(dead_code)]

use core::ffi::c_void;
use core::mem::{align_of, size_of};
use core::ptr::{null, null_mut, NonNull};

/// Convert a `&T` into a `*const T` without using an `as`.
#[inline]
pub(crate) const fn as_ptr<T>(t: &T) -> *const T {
    t
}

/// Convert a `&mut T` into a `*mut T` without using an `as`.
#[inline]
pub(crate) fn as_mut_ptr<T>(t: &mut T) -> *mut T {
    t
}

/// Convert an `Option<&T>` into a possibly-null `*const T`.
#[inline]
pub(crate) const fn optional_as_ptr<T>(t: Option<&T>) -> *const T {
    match t {
        Some(t) => t,
        None => null(),
    }
}

/// Convert an `Option<&mut T>` into a possibly-null `*mut T`.
#[inline]
pub(crate) fn optional_as_mut_ptr<T>(t: Option<&mut T>) -> *mut T {
    match t {
        Some(t) => t,
        None => null_mut(),
    }
}

/// Convert a `*mut c_void` to a `*mut T`, checking that it is not null,
/// misaligned, or pointing to a region of memory that wraps around the address
/// space.
pub(crate) fn check_raw_pointer<T>(value: *mut c_void) -> Option<NonNull<T>> {
    if (value as usize).checked_add(size_of::<T>()).is_none()
        || (value as usize) % align_of::<T>() != 0
    {
        return None;
    }

    NonNull::new(value.cast())
}
