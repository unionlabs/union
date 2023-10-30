//! This crate allows interacting with the data stored by [`OsStr`] and
//! [`OsString`], without resorting to panics or corruption for invalid UTF-8.
//! Thus, methods can be used that are already defined on [`[u8]`][slice] and
//! [`Vec<u8>`].
//!
//! Typically, the only way to losslessly construct [`OsStr`] or [`OsString`]
//! from a byte sequence is to use `OsStr::new(str::from_utf8(bytes)?)`, which
//! requires the bytes to be valid in UTF-8. However, since this crate makes
//! conversions directly between the platform encoding and raw bytes, even some
//! strings invalid in UTF-8 can be converted.
//!
//! In most cases, [`RawOsStr`] and [`RawOsString`] should be used.
//! [`OsStrBytes`] and [`OsStringBytes`] provide lower-level APIs that are
//! easier to misuse.
//!
//! # Encoding
//!
//! The encoding of bytes returned or accepted by methods of this crate is
//! intentionally left unspecified. It may vary for different platforms, so
//! defining it would run contrary to the goal of generic string handling.
//! However, the following invariants will always be upheld:
//!
//! - The encoding will be compatible with UTF-8. In particular, splitting an
//!   encoded byte sequence by a UTF-8–encoded character always produces other
//!   valid byte sequences. They can be re-encoded without error using
//!   [`OsStrBytes::from_raw_bytes`] and similar methods.
//!
//! - All characters valid in platform strings are representable. [`OsStr`] and
//!   [`OsString`] can always be losslessly reconstructed from extracted bytes.
//!
//! Note that the chosen encoding may not match how Rust stores these strings
//! internally, which is undocumented. For instance, the result of calling
//! [`OsStr::len`] will not necessarily match the number of bytes this crate
//! uses to represent the same string.
//!
//! Additionally, concatenation may yield unexpected results without a UTF-8
//! separator. If two platform strings need to be concatenated, the only safe
//! way to do so is using [`OsString::push`]. This limitation also makes it
//! undesirable to use the bytes in interchange.
//!
//! Since this encoding can change between versions and platforms, it should
//! not be used for storage. The standard library provides implementations of
//! [`OsStrExt`] and [`OsStringExt`] for various platforms, which should be
//! preferred for that use case.
//!
//! # User Input
//!
//! Traits in this crate should ideally not be used to convert byte sequences
//! that did not originate from [`OsStr`] or a related struct. The encoding
//! used by this crate is an implementation detail, so it does not make sense
//! to expose it to users.
//!
//! Crate [bstr] offers some useful alternative methods, such as
//! [`ByteSlice::to_os_str`] and [`ByteVec::into_os_string`], that are meant
//! for user input. But, they reject some byte sequences used to represent
//! valid platform strings, which would be undesirable for reliable path
//! handling. They are best used only when accepting unknown input.
//!
//! This crate is meant to help when you already have an instance of [`OsStr`]
//! and need to modify the data in a lossless way.
//!
//! # Features
//!
//! These features are optional and can be enabled or disabled in a
//! "Cargo.toml" file.
//!
//! ### Default Features
//!
//! - **memchr** -
//!   Changes the implementation to use crate [memchr] for better performance.
//!   This feature is useless when "raw\_os\_str" is disabled.
//!
//!   For more information, see [`RawOsStr`][memchr complexity].
//!
//! - **raw\_os\_str** -
//!   Enables use of [`RawOsStr`] and [`RawOsString`].
//!
//! ### Optional Features
//!
//! - **print\_bytes** -
//!   Provides implementations of [`print_bytes::ToBytes`] for [`RawOsStr`] and
//!   [`RawOsString`].
//!
//! - **uniquote** -
//!   Provides implementations of [`uniquote::Quote`] for [`RawOsStr`] and
//!   [`RawOsString`].
//!
//! # Implementation
//!
//! Some methods return [`Cow`] to account for platform differences. However,
//! no guarantee is made that the same variant of that enum will always be
//! returned for the same platform. Whichever can be constructed most
//! efficiently will be returned.
//!
//! All traits are [sealed], meaning that they can only be implemented by this
//! crate. Otherwise, backward compatibility would be more difficult to
//! maintain for new features.
//!
//! # Complexity
//!
//! The time complexities of trait methods will vary based on what
//! functionality is available for the platform. At worst, they will all be
//! linear, but some can take constant time. For example,
//! [`OsStringBytes::from_raw_vec`] might be able to reuse the allocation for
//! its argument.
//!
//! # Examples
//!
//! ```
//! # #[cfg(any())]
//! use std::env;
//! use std::fs;
//! # use std::io;
//!
//! use os_str_bytes::OsStrBytes;
//!
//! # mod env {
//! #   use std::env;
//! #   use std::ffi::OsString;
//! #
//! #   pub fn args_os() -> impl Iterator<Item = OsString> {
//! #       let mut file = env::temp_dir();
//! #       file.push("os_str_bytes\u{E9}.txt");
//! #       return vec![OsString::new(), file.into_os_string()].into_iter();
//! #   }
//! # }
//! #
//! for file in env::args_os().skip(1) {
//!     if file.to_raw_bytes().first() != Some(&b'-') {
//!         let string = "Hello, world!";
//!         fs::write(&file, string)?;
//!         assert_eq!(string, fs::read_to_string(file)?);
//!     }
//! }
//! #
//! # Ok::<_, io::Error>(())
//! ```
//!
//! [bstr]: https://crates.io/crates/bstr
//! [`ByteSlice::to_os_str`]: https://docs.rs/bstr/0.2.12/bstr/trait.ByteSlice.html#method.to_os_str
//! [`ByteVec::into_os_string`]: https://docs.rs/bstr/0.2.12/bstr/trait.ByteVec.html#method.into_os_string
//! [memchr complexity]: RawOsStr#complexity
//! [memchr]: https://crates.io/crates/memchr
//! [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
//! [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
//! [sealed]: https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed
//! [print\_bytes]: https://crates.io/crates/print_bytes

// Only require a nightly compiler when building documentation for docs.rs.
// This is a private option that should not be used.
// https://github.com/rust-lang/docs.rs/issues/147#issuecomment-389544407
// https://github.com/dylni/os_str_bytes/issues/2
#![cfg_attr(os_str_bytes_docs_rs, feature(doc_cfg))]
// Nightly is also currently required for the SGX platform.
#![cfg_attr(
    all(target_vendor = "fortanix", target_env = "sgx"),
    feature(sgx_platform)
)]
#![warn(unsafe_op_in_unsafe_fn)]
#![warn(unused_results)]

use std::borrow::Cow;
use std::error::Error;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::Path;
use std::path::PathBuf;
use std::result;

macro_rules! if_raw_str {
    ( $($item:item)+ ) => {
        $(
            #[cfg(feature = "raw_os_str")]
            $item
        )+
    };
}

#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown"),
    path = "wasm32/mod.rs"
)]
#[cfg_attr(windows, path = "windows/mod.rs")]
#[cfg_attr(
    not(any(all(target_arch = "wasm32", target_os = "unknown"), windows)),
    path = "common/mod.rs"
)]
mod imp;

mod util;

if_raw_str! {
    pub mod iter;

    mod pattern;
    pub use pattern::Pattern;

    mod raw_str;
    pub use raw_str::RawOsStr;
    pub use raw_str::RawOsString;
}

/// The error that occurs when a byte sequence is not representable in the
/// platform encoding.
///
/// [`Result::unwrap`] should almost always be called on results containing
/// this error. It should be known whether or not byte sequences are properly
/// encoded for the platform, since [the module-level documentation][encoding]
/// discourages using encoded bytes in interchange. Results are returned
/// primarily to make panicking behavior explicit.
///
/// On Unix, this error is never returned, but [`OsStrExt`] or [`OsStringExt`]
/// should be used instead if that needs to be guaranteed.
///
/// [encoding]: self#encoding
/// [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
/// [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
/// [`Result::unwrap`]: ::std::result::Result::unwrap
#[derive(Debug, Eq, PartialEq)]
pub struct EncodingError(imp::EncodingError);

impl Display for EncodingError {
    #[inline]
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Error for EncodingError {}

type Result<T> = result::Result<T, EncodingError>;

/// A platform agnostic variant of [`OsStrExt`].
///
/// For more information, see [the module-level documentation][module].
///
/// [module]: self
/// [`OsStrExt`]: ::std::os::unix::ffi::OsStrExt
pub trait OsStrBytes: private::Sealed + ToOwned {
    /// Converts a byte slice into an equivalent platform-native string.
    ///
    /// Provided byte strings should always be valid for the [unspecified
    /// encoding] used by this crate.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::ffi::OsStr;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStrBytes;
    ///
    /// let os_string = env::current_exe()?;
    /// let os_bytes = os_string.to_raw_bytes();
    /// assert_eq!(os_string, OsStr::from_raw_bytes(os_bytes).unwrap());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [unspecified encoding]: self#encoding
    fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
    where
        S: Into<Cow<'a, [u8]>>;

    /// Converts a platform-native string into an equivalent byte slice.
    ///
    /// The returned bytes string will use an [unspecified encoding].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStrBytes;
    ///
    /// let os_string = env::current_exe()?;
    /// println!("{:?}", os_string.to_raw_bytes());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [unspecified encoding]: self#encoding
    #[must_use]
    fn to_raw_bytes(&self) -> Cow<'_, [u8]>;
}

impl OsStrBytes for OsStr {
    #[inline]
    fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
    where
        S: Into<Cow<'a, [u8]>>,
    {
        match string.into() {
            Cow::Borrowed(string) => {
                imp::os_str_from_bytes(string).map_err(EncodingError)
            }
            Cow::Owned(string) => {
                OsStringBytes::from_raw_vec(string).map(Cow::Owned)
            }
        }
    }

    #[inline]
    fn to_raw_bytes(&self) -> Cow<'_, [u8]> {
        imp::os_str_to_bytes(self)
    }
}

impl OsStrBytes for Path {
    #[inline]
    fn from_raw_bytes<'a, S>(string: S) -> Result<Cow<'a, Self>>
    where
        S: Into<Cow<'a, [u8]>>,
    {
        OsStr::from_raw_bytes(string).map(|os_string| match os_string {
            Cow::Borrowed(os_string) => Cow::Borrowed(Self::new(os_string)),
            Cow::Owned(os_string) => Cow::Owned(os_string.into()),
        })
    }

    #[inline]
    fn to_raw_bytes(&self) -> Cow<'_, [u8]> {
        self.as_os_str().to_raw_bytes()
    }
}

/// A platform agnostic variant of [`OsStringExt`].
///
/// For more information, see [the module-level documentation][module].
///
/// [module]: self
/// [`OsStringExt`]: ::std::os::unix::ffi::OsStringExt
pub trait OsStringBytes: private::Sealed + Sized {
    /// Converts a byte vector into an equivalent platform-native string.
    ///
    /// Provided byte strings should always be valid for the [unspecified
    /// encoding] used by this crate.
    ///
    /// # Errors
    ///
    /// See documentation for [`EncodingError`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use std::ffi::OsString;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStringBytes;
    ///
    /// let os_string = env::current_exe()?;
    /// let os_bytes = os_string.clone().into_raw_vec();
    /// assert_eq!(os_string, OsString::from_raw_vec(os_bytes).unwrap());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [unspecified encoding]: self#encoding
    fn from_raw_vec(string: Vec<u8>) -> Result<Self>;

    /// Converts a platform-native string into an equivalent byte vector.
    ///
    /// The returned byte string will use an [unspecified encoding].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStringBytes;
    ///
    /// let os_string = env::current_exe()?;
    /// println!("{:?}", os_string.into_raw_vec());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [unspecified encoding]: self#encoding
    #[must_use]
    fn into_raw_vec(self) -> Vec<u8>;
}

impl OsStringBytes for OsString {
    #[inline]
    fn from_raw_vec(string: Vec<u8>) -> Result<Self> {
        imp::os_string_from_vec(string).map_err(EncodingError)
    }

    #[inline]
    fn into_raw_vec(self) -> Vec<u8> {
        imp::os_string_into_vec(self)
    }
}

impl OsStringBytes for PathBuf {
    #[inline]
    fn from_raw_vec(string: Vec<u8>) -> Result<Self> {
        OsString::from_raw_vec(string).map(Into::into)
    }

    #[inline]
    fn into_raw_vec(self) -> Vec<u8> {
        self.into_os_string().into_raw_vec()
    }
}

mod private {
    use std::ffi::OsStr;
    use std::ffi::OsString;
    use std::path::Path;
    use std::path::PathBuf;

    pub trait Sealed {}
    impl Sealed for char {}
    impl Sealed for OsStr {}
    impl Sealed for OsString {}
    impl Sealed for Path {}
    impl Sealed for PathBuf {}
    impl Sealed for &str {}
    impl Sealed for &String {}
}
