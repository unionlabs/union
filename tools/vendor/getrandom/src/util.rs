// Copyright 2019 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(dead_code)]
use core::{
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

// This structure represents a lazily initialized static usize value. Useful
// when it is preferable to just rerun initialization instead of locking.
// Both unsync_init and sync_init will invoke an init() function until it
// succeeds, then return the cached value for future calls.
//
// Both methods support init() "failing". If the init() method returns UNINIT,
// that value will be returned as normal, but will not be cached.
//
// Users should only depend on the _value_ returned by init() functions.
// Specifically, for the following init() function:
//      fn init() -> usize {
//          a();
//          let v = b();
//          c();
//          v
//      }
// the effects of c() or writes to shared memory will not necessarily be
// observed and additional synchronization methods with be needed.
pub struct LazyUsize(AtomicUsize);

impl LazyUsize {
    pub const fn new() -> Self {
        Self(AtomicUsize::new(Self::UNINIT))
    }

    // The initialization is not completed.
    pub const UNINIT: usize = usize::max_value();

    // Runs the init() function at least once, returning the value of some run
    // of init(). Multiple callers can run their init() functions in parallel.
    // init() should always return the same value, if it succeeds.
    pub fn unsync_init(&self, init: impl FnOnce() -> usize) -> usize {
        // Relaxed ordering is fine, as we only have a single atomic variable.
        let mut val = self.0.load(Relaxed);
        if val == Self::UNINIT {
            val = init();
            self.0.store(val, Relaxed);
        }
        val
    }
}

// Identical to LazyUsize except with bool instead of usize.
pub struct LazyBool(LazyUsize);

impl LazyBool {
    pub const fn new() -> Self {
        Self(LazyUsize::new())
    }

    pub fn unsync_init(&self, init: impl FnOnce() -> bool) -> bool {
        self.0.unsync_init(|| init() as usize) != 0
    }
}

/// Polyfill for `maybe_uninit_slice` feature's
/// `MaybeUninit::slice_assume_init_mut`. Every element of `slice` must have
/// been initialized.
#[inline(always)]
pub unsafe fn slice_assume_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    // SAFETY: `MaybeUninit<T>` is guaranteed to be layout-compatible with `T`.
    &mut *(slice as *mut [MaybeUninit<T>] as *mut [T])
}

#[inline]
pub fn uninit_slice_fill_zero(slice: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    unsafe { ptr::write_bytes(slice.as_mut_ptr(), 0, slice.len()) };
    unsafe { slice_assume_init_mut(slice) }
}

#[inline(always)]
pub fn slice_as_uninit<T>(slice: &[T]) -> &[MaybeUninit<T>] {
    // SAFETY: `MaybeUninit<T>` is guaranteed to be layout-compatible with `T`.
    // There is no risk of writing a `MaybeUninit<T>` into the result since
    // the result isn't mutable.
    unsafe { &*(slice as *const [T] as *const [MaybeUninit<T>]) }
}

/// View an mutable initialized array as potentially-uninitialized.
///
/// This is unsafe because it allows assigning uninitialized values into
/// `slice`, which would be undefined behavior.
#[inline(always)]
pub unsafe fn slice_as_uninit_mut<T>(slice: &mut [T]) -> &mut [MaybeUninit<T>] {
    // SAFETY: `MaybeUninit<T>` is guaranteed to be layout-compatible with `T`.
    &mut *(slice as *mut [T] as *mut [MaybeUninit<T>])
}
