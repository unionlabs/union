//! Convert closures into wakers.
//!
//! A [`Waker`] is just a fancy callback. This crate converts regular closures into wakers.

#![no_std]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

extern crate alloc;

use alloc::sync::Arc;
use core::mem::{self, ManuallyDrop};
use core::task::{RawWaker, RawWakerVTable, Waker};

/// Converts a closure into a [`Waker`].
///
/// The closure gets called every time the waker is woken.
///
/// # Examples
///
/// ```
/// use waker_fn::waker_fn;
///
/// let waker = waker_fn(|| println!("woken"));
///
/// waker.wake_by_ref(); // Prints "woken".
/// waker.wake();        // Prints "woken".
/// ```
pub fn waker_fn<F: Fn() + Send + Sync + 'static>(f: F) -> Waker {
    let raw = Arc::into_raw(Arc::new(f)) as *const ();
    let vtable = &Helper::<F>::VTABLE;
    unsafe { Waker::from_raw(RawWaker::new(raw, vtable)) }
}

struct Helper<F>(F);

impl<F: Fn() + Send + Sync + 'static> Helper<F> {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        Self::clone_waker,
        Self::wake,
        Self::wake_by_ref,
        Self::drop_waker,
    );

    unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        mem::forget(arc.clone());
        RawWaker::new(ptr, &Self::VTABLE)
    }

    unsafe fn wake(ptr: *const ()) {
        let arc = Arc::from_raw(ptr as *const F);
        (arc)();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        let arc = ManuallyDrop::new(Arc::from_raw(ptr as *const F));
        (arc)();
    }

    unsafe fn drop_waker(ptr: *const ()) {
        drop(Arc::from_raw(ptr as *const F));
    }
}
