// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! Cryptographic pseudo-random number generation.
//!
//! An application should create a single `SystemRandom` and then use it for
//! all randomness generation. Functions that generate random bytes should take
//! a `&dyn SecureRandom` parameter instead of instantiating their own. Besides
//! being more efficient, this also helps document where non-deterministic
//! (random) outputs occur. Taking a reference to a `SecureRandom` also helps
//! with testing techniques like fuzzing, where it is useful to use a
//! (non-secure) deterministic implementation of `SecureRandom` so that results
//! can be replayed. Following this pattern also may help with sandboxing
//! (seccomp filters on Linux in particular). See `SystemRandom`'s
//! documentation for more details.

use crate::error;

/// A secure random number generator.
pub trait SecureRandom: sealed::SecureRandom {
    /// Fills `dest` with random bytes.
    fn fill(&self, dest: &mut [u8]) -> Result<(), error::Unspecified>;
}

impl<T> SecureRandom for T
where
    T: sealed::SecureRandom,
{
    #[inline(always)]
    fn fill(&self, dest: &mut [u8]) -> Result<(), error::Unspecified> {
        self.fill_impl(dest)
    }
}

/// A random value constructed from a `SecureRandom` that hasn't been exposed
/// through any safe Rust interface.
///
/// Intentionally does not implement any traits other than `Sized`.
pub struct Random<T: RandomlyConstructable>(T);

impl<T: RandomlyConstructable> Random<T> {
    /// Expose the random value.
    #[inline]
    pub fn expose(self) -> T {
        self.0
    }
}

/// Generate the new random value using `rng`.
#[inline]
pub fn generate<T: RandomlyConstructable>(
    rng: &dyn SecureRandom,
) -> Result<Random<T>, error::Unspecified>
where
    T: RandomlyConstructable,
{
    let mut r = T::zero();
    rng.fill(r.as_mut_bytes())?;
    Ok(Random(r))
}

pub(crate) mod sealed {
    use crate::error;

    pub trait SecureRandom: core::fmt::Debug {
        /// Fills `dest` with random bytes.
        fn fill_impl(&self, dest: &mut [u8]) -> Result<(), error::Unspecified>;
    }

    pub trait RandomlyConstructable: Sized {
        fn zero() -> Self; // `Default::default()`
        fn as_mut_bytes(&mut self) -> &mut [u8]; // `AsMut<[u8]>::as_mut`
    }

    macro_rules! impl_random_arrays {
        [ $($len:expr)+ ] => {
            $(
                impl RandomlyConstructable for [u8; $len] {
                    #[inline]
                    fn zero() -> Self { [0; $len] }

                    #[inline]
                    fn as_mut_bytes(&mut self) -> &mut [u8] { &mut self[..] }
                }
            )+
        }
    }

    impl_random_arrays![4 8 16 32 48 64 128 256];
}

/// A type that can be returned by `ring::rand::generate()`.
pub trait RandomlyConstructable: self::sealed::RandomlyConstructable {}
impl<T> RandomlyConstructable for T where T: self::sealed::RandomlyConstructable {}

/// A secure random number generator where the random values come directly
/// from the operating system.
///
/// A single `SystemRandom` may be shared across multiple threads safely.
///
/// `new()` is guaranteed to always succeed and to have low latency; it won't
/// try to open or read from a file or do similar things. The first call to
/// `fill()` may block a substantial amount of time since any and all
/// initialization is deferred to it. Therefore, it may be a good idea to call
/// `fill()` once at a non-latency-sensitive time to minimize latency for
/// future calls.
///
/// On Linux (including Android), `fill()` will use the [`getrandom`] syscall.
/// If the kernel is too old to support `getrandom` then by default `fill()`
/// falls back to reading from `/dev/urandom`. This decision is made the first
/// time `fill` *succeeds*. The fallback to `/dev/urandom` can be disabled by
/// disabling the `dev_urandom_fallback` default feature; this should be done
/// whenever the target system is known to support `getrandom`. When
/// `/dev/urandom` is used, a file handle for `/dev/urandom` won't be opened
/// until `fill` is called; `SystemRandom::new()` will not open `/dev/urandom`
/// or do other potentially-high-latency things. The file handle will never be
/// closed, until the operating system closes it at process shutdown. All
/// instances of `SystemRandom` will share a single file handle. To properly
/// implement seccomp filtering when the `dev_urandom_fallback` default feature
/// is disabled, allow `getrandom` through. When the fallback is enabled, allow
/// file opening, `getrandom`, and `read` up until the first call to `fill()`
/// succeeds; after that, allow `getrandom` and `read`.
///
/// On macOS and iOS, `fill()` is implemented using `SecRandomCopyBytes`.
///
/// On wasm32-unknown-unknown (non-WASI), `fill()` is implemented using
/// `window.crypto.getRandomValues()`. It must be used in a context where the
/// global object is a `Window`; i.e. it must not be used in a Worker or a
/// non-browser context.
///
/// On Windows, `fill` is implemented using the platform's API for secure
/// random number generation.
///
/// [`getrandom`]: http://man7.org/linux/man-pages/man2/getrandom.2.html
#[derive(Clone, Debug)]
pub struct SystemRandom(());

impl SystemRandom {
    /// Constructs a new `SystemRandom`.
    #[inline(always)]
    pub fn new() -> Self {
        Self(())
    }
}

impl sealed::SecureRandom for SystemRandom {
    #[inline(always)]
    fn fill_impl(&self, dest: &mut [u8]) -> Result<(), error::Unspecified> {
        fill_impl(dest)
    }
}

impl crate::sealed::Sealed for SystemRandom {}

#[cfg(any(
    all(
        any(target_os = "android", target_os = "linux"),
        not(feature = "dev_urandom_fallback")
    ),
    target_arch = "wasm32",
    windows
))]
use self::sysrand::fill as fill_impl;

#[cfg(all(
    any(target_os = "android", target_os = "linux"),
    feature = "dev_urandom_fallback"
))]
use self::sysrand_or_urandom::fill as fill_impl;

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris",
))]
use self::urandom::fill as fill_impl;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use self::darwin::fill as fill_impl;

#[cfg(any(target_os = "fuchsia"))]
use self::fuchsia::fill as fill_impl;

#[cfg(any(target_os = "android", target_os = "linux"))]
mod sysrand_chunk {
    use crate::{c, error};

    #[inline]
    pub fn chunk(dest: &mut [u8]) -> Result<usize, error::Unspecified> {
        use libc::c_long;

        // See `SYS_getrandom` in #include <sys/syscall.h>.

        #[cfg(target_arch = "aarch64")]
        const SYS_GETRANDOM: c_long = 278;

        #[cfg(target_arch = "arm")]
        const SYS_GETRANDOM: c_long = 384;

        #[cfg(target_arch = "x86")]
        const SYS_GETRANDOM: c_long = 355;

        #[cfg(target_arch = "x86_64")]
        const SYS_GETRANDOM: c_long = 318;

        let chunk_len: c::size_t = dest.len();
        let r = unsafe { libc::syscall(SYS_GETRANDOM, dest.as_mut_ptr(), chunk_len, 0) };
        if r < 0 {
            let errno;

            #[cfg(target_os = "linux")]
            {
                errno = unsafe { *libc::__errno_location() };
            }

            #[cfg(target_os = "android")]
            {
                errno = unsafe { *libc::__errno() };
            }

            if errno == libc::EINTR {
                // If an interrupt occurs while getrandom() is blocking to wait
                // for the entropy pool, then EINTR is returned. Returning 0
                // will cause the caller to try again.
                return Ok(0);
            }
            return Err(error::Unspecified);
        }
        Ok(r as usize)
    }
}

#[cfg(all(
    target_arch = "wasm32",
    target_vendor = "unknown",
    target_os = "unknown",
    target_env = "",
))]
mod sysrand_chunk {
    use crate::error;

    pub fn chunk(mut dest: &mut [u8]) -> Result<usize, error::Unspecified> {
        // This limit is specified in
        // https://www.w3.org/TR/WebCryptoAPI/#Crypto-method-getRandomValues.
        const MAX_LEN: usize = 65_536;
        if dest.len() > MAX_LEN {
            dest = &mut dest[..MAX_LEN];
        };

        let _ = web_sys::window()
            .ok_or(error::Unspecified)?
            .crypto()
            .map_err(|_| error::Unspecified)?
            .get_random_values_with_u8_array(dest)
            .map_err(|_| error::Unspecified)?;

        Ok(dest.len())
    }
}

#[cfg(windows)]
mod sysrand_chunk {
    use crate::{error, polyfill};

    #[inline]
    pub fn chunk(dest: &mut [u8]) -> Result<usize, error::Unspecified> {
        use winapi::shared::wtypesbase::ULONG;

        assert!(core::mem::size_of::<usize>() >= core::mem::size_of::<ULONG>());
        let len = core::cmp::min(dest.len(), polyfill::usize_from_u32(ULONG::max_value()));
        let result = unsafe {
            winapi::um::ntsecapi::RtlGenRandom(
                dest.as_mut_ptr() as *mut winapi::ctypes::c_void,
                len as ULONG,
            )
        };
        if result == 0 {
            return Err(error::Unspecified);
        }

        Ok(len)
    }
}

#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_arch = "wasm32",
    windows
))]
mod sysrand {
    use super::sysrand_chunk::chunk;
    use crate::error;

    pub fn fill(dest: &mut [u8]) -> Result<(), error::Unspecified> {
        let mut read_len = 0;
        while read_len < dest.len() {
            let chunk_len = chunk(&mut dest[read_len..])?;
            read_len += chunk_len;
        }
        Ok(())
    }
}

// Keep the `cfg` conditions in sync with the conditions in lib.rs.
#[cfg(all(
    any(target_os = "android", target_os = "linux"),
    feature = "dev_urandom_fallback"
))]
mod sysrand_or_urandom {
    use crate::error;

    enum Mechanism {
        Sysrand,
        DevURandom,
    }

    #[inline]
    pub fn fill(dest: &mut [u8]) -> Result<(), error::Unspecified> {
        use once_cell::sync::Lazy;
        static MECHANISM: Lazy<Mechanism> = Lazy::new(|| {
            let mut dummy = [0u8; 1];
            if super::sysrand_chunk::chunk(&mut dummy[..]).is_err() {
                Mechanism::DevURandom
            } else {
                Mechanism::Sysrand
            }
        });

        match *MECHANISM {
            Mechanism::Sysrand => super::sysrand::fill(dest),
            Mechanism::DevURandom => super::urandom::fill(dest),
        }
    }
}

#[cfg(any(
    all(
        any(target_os = "android", target_os = "linux"),
        feature = "dev_urandom_fallback"
    ),
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris",
    target_os = "illumos"
))]
mod urandom {
    use crate::error;

    #[cfg_attr(any(target_os = "android", target_os = "linux"), cold, inline(never))]
    pub fn fill(dest: &mut [u8]) -> Result<(), error::Unspecified> {
        extern crate std;

        use once_cell::sync::Lazy;

        static FILE: Lazy<Result<std::fs::File, std::io::Error>> =
            Lazy::new(|| std::fs::File::open("/dev/urandom"));

        match *FILE {
            Ok(ref file) => {
                use std::io::Read;
                (&*file).read_exact(dest).map_err(|_| error::Unspecified)
            }
            Err(_) => Err(error::Unspecified),
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod darwin {
    use crate::{c, error};

    pub fn fill(dest: &mut [u8]) -> Result<(), error::Unspecified> {
        let r = unsafe { SecRandomCopyBytes(kSecRandomDefault, dest.len(), dest.as_mut_ptr()) };
        match r {
            0 => Ok(()),
            _ => Err(error::Unspecified),
        }
    }

    // XXX: This is emulating an opaque type with a non-opaque type. TODO: Fix
    // this when
    // https://github.com/rust-lang/rfcs/pull/1861#issuecomment-274613536 is
    // resolved.
    #[repr(C)]
    struct SecRandomRef([u8; 0]);

    #[link(name = "Security", kind = "framework")]
    extern "C" {
        static kSecRandomDefault: &'static SecRandomRef;

        // For now `rnd` must be `kSecRandomDefault`.
        #[must_use]
        fn SecRandomCopyBytes(
            rnd: &'static SecRandomRef,
            count: c::size_t,
            bytes: *mut u8,
        ) -> c::int;
    }
}

#[cfg(any(target_os = "fuchsia"))]
mod fuchsia {
    use crate::error;

    pub fn fill(dest: &mut [u8]) -> Result<(), error::Unspecified> {
        unsafe {
            zx_cprng_draw(dest.as_mut_ptr(), dest.len());
        }
        Ok(())
    }

    #[link(name = "zircon")]
    extern "C" {
        fn zx_cprng_draw(buffer: *mut u8, length: usize);
    }
}
