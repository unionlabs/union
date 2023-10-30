// Copyright 2016 lazy-static.rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate core;
extern crate std;

use self::std::prelude::v1::*;
use self::std::cell::Cell;
use self::std::hint::unreachable_unchecked;
use self::std::sync::Once;
#[allow(deprecated)]
pub use self::std::sync::ONCE_INIT;

// FIXME: Replace Option<T> with MaybeUninit<T> (stable since 1.36.0)
pub struct Lazy<T: Sync>(Cell<Option<T>>, Once);

impl<T: Sync> Lazy<T> {
    #[allow(deprecated)]
    pub const INIT: Self = Lazy(Cell::new(None), ONCE_INIT);

    #[inline(always)]
    pub fn get<F>(&'static self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.1.call_once(|| {
            self.0.set(Some(f()));
        });

        // `self.0` is guaranteed to be `Some` by this point
        // The `Once` will catch and propagate panics
        unsafe {
            match *self.0.as_ptr() {
                Some(ref x) => x,
                None => {
                    debug_assert!(false, "attempted to derefence an uninitialized lazy static. This is a bug");

                    unreachable_unchecked()
                },
            }
        }
    }
}

unsafe impl<T: Sync> Sync for Lazy<T> {}

#[macro_export]
#[doc(hidden)]
macro_rules! __lazy_static_create {
    ($NAME:ident, $T:ty) => {
        static $NAME: $crate::lazy::Lazy<$T> = $crate::lazy::Lazy::INIT;
    };
}
