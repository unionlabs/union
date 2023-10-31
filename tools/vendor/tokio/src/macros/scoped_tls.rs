use crate::loom::thread::LocalKey;

use std::cell::Cell;
use std::marker;

/// Sets a reference as a thread-local.
macro_rules! scoped_thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty) => (
        $(#[$attrs])*
        $vis static $name: $crate::macros::scoped_tls::ScopedKey<$ty>
            = $crate::macros::scoped_tls::ScopedKey {
                inner: {
                    thread_local!(static FOO: ::std::cell::Cell<*const ()> = const {
                        std::cell::Cell::new(::std::ptr::null())
                    });
                    &FOO
                },
                _marker: ::std::marker::PhantomData,
            };
    )
}

/// Type representing a thread local storage key corresponding to a reference
/// to the type parameter `T`.
pub(crate) struct ScopedKey<T> {
    pub(crate) inner: &'static LocalKey<Cell<*const ()>>,
    pub(crate) _marker: marker::PhantomData<T>,
}

unsafe impl<T> Sync for ScopedKey<T> {}

impl<T> ScopedKey<T> {
    /// Inserts a value into this scoped thread local storage slot for a
    /// duration of a closure.
    pub(crate) fn set<F, R>(&'static self, t: &T, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        struct Reset {
            key: &'static LocalKey<Cell<*const ()>>,
            val: *const (),
        }

        impl Drop for Reset {
            fn drop(&mut self) {
                self.key.with(|c| c.set(self.val));
            }
        }

        let prev = self.inner.with(|c| {
            let prev = c.get();
            c.set(t as *const _ as *const ());
            prev
        });

        let _reset = Reset {
            key: self.inner,
            val: prev,
        };

        f()
    }

    /// Gets a value out of this scoped variable.
    pub(crate) fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(Option<&T>) -> R,
    {
        let val = self.inner.with(|c| c.get());

        if val.is_null() {
            f(None)
        } else {
            unsafe { f(Some(&*(val as *const T))) }
        }
    }
}
