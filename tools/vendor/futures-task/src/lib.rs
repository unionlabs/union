//! Tools for working with tasks.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
// It cannot be included in the published code because this lints have false positives in the minimum required version.
#![cfg_attr(test, warn(single_use_lifetimes))]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]

#[cfg(feature = "alloc")]
extern crate alloc;

mod spawn;
pub use crate::spawn::{LocalSpawn, Spawn, SpawnError};

#[cfg(not(futures_no_atomic_cas))]
#[cfg(feature = "alloc")]
mod arc_wake;
#[cfg(not(futures_no_atomic_cas))]
#[cfg(feature = "alloc")]
pub use crate::arc_wake::ArcWake;

#[cfg(not(futures_no_atomic_cas))]
#[cfg(feature = "alloc")]
mod waker;
#[cfg(not(futures_no_atomic_cas))]
#[cfg(feature = "alloc")]
pub use crate::waker::waker;

#[cfg(not(futures_no_atomic_cas))]
#[cfg(feature = "alloc")]
mod waker_ref;
#[cfg(not(futures_no_atomic_cas))]
#[cfg(feature = "alloc")]
pub use crate::waker_ref::{waker_ref, WakerRef};

mod future_obj;
pub use crate::future_obj::{FutureObj, LocalFutureObj, UnsafeFutureObj};

mod noop_waker;
pub use crate::noop_waker::noop_waker;
pub use crate::noop_waker::noop_waker_ref;

#[doc(no_inline)]
pub use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
