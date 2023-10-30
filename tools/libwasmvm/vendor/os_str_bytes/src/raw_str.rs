use std::borrow::Borrow;
use std::borrow::Cow;
use std::borrow::ToOwned;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::mem;
use std::ops::Deref;
use std::ops::Index;
use std::ops::Range;
use std::ops::RangeFrom;
use std::ops::RangeFull;
use std::ops::RangeInclusive;
use std::ops::RangeTo;
use std::ops::RangeToInclusive;
use std::result;
use std::str;

#[cfg(feature = "memchr")]
use memchr::memmem::find;
#[cfg(feature = "memchr")]
use memchr::memmem::rfind;

use super::imp;
use super::imp::raw;
use super::iter::Split;
use super::pattern::Encoded as EncodedPattern;
use super::private;
use super::Pattern;

if_checked_conversions! {
    use super::EncodingError;
    use super::Result;
}

#[cfg(not(feature = "memchr"))]
fn find(string: &[u8], pat: &[u8]) -> Option<usize> {
    (0..=string.len().checked_sub(pat.len())?)
        .find(|&x| string[x..].starts_with(pat))
}

#[cfg(not(feature = "memchr"))]
fn rfind(string: &[u8], pat: &[u8]) -> Option<usize> {
    (pat.len()..=string.len())
        .rfind(|&x| string[..x].ends_with(pat))
        .map(|x| x - pat.len())
}

#[allow(clippy::missing_safety_doc)]
unsafe trait TransmuteBox {
    fn transmute_box<R>(self: Box<Self>) -> Box<R>
    where
        R: ?Sized + TransmuteBox,
    {
        let value = Box::into_raw(self);
        // SAFETY: This trait is only implemented for types that can be
        // transmuted.
        unsafe { Box::from_raw(mem::transmute_copy(&value)) }
    }
}

// SAFETY: This struct has a layout that makes this operation safe.
unsafe impl TransmuteBox for RawOsStr {}
unsafe impl TransmuteBox for [u8] {}

/// A container for borrowed byte strings converted by this crate.
///
/// This wrapper is intended to prevent violating the invariants of the
/// [unspecified encoding] used by this crate and minimize encoding
/// conversions.
///
/// # Indices
///
/// Methods of this struct that accept indices require that the index lie on a
/// UTF-8 boundary. Although it is possible to manipulate platform strings
/// based on other indices, this crate currently does not support them for
/// slicing methods. They would add significant complication to the
/// implementation and are generally not necessary. However, all indices
/// returned by this struct can be used for slicing.
///
/// On Unix, all indices are permitted, to avoid false positives. However,
/// relying on this implementation detail is discouraged. Platform-specific
/// indices are error-prone.
///
/// # Complexity
///
/// All searching methods have worst-case multiplicative time complexity (i.e.,
/// `O(self.raw_len() * pat.len())`). Enabling the "memchr" feature allows
/// these methods to instead run in linear time in the worst case (documented
/// for [`memchr::memmem::find`][memchr complexity]).
///
/// # Safety
///
/// Although this type is annotated with `#[repr(transparent)]`, the inner
/// representation is not stable. Transmuting between this type and any other
/// causes immediate undefined behavior.
///
/// [memchr complexity]: memchr::memmem::find#complexity
/// [unspecified encoding]: super#encoding
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
#[repr(transparent)]
pub struct RawOsStr([u8]);

impl RawOsStr {
    const fn from_inner(string: &[u8]) -> &Self {
        // SAFETY: This struct has a layout that makes this operation safe.
        unsafe { mem::transmute(string) }
    }

    /// Converts a platform-native string into a representation that can be
    /// more easily manipulated.
    ///
    /// This method performs the necessary conversion immediately, so it can be
    /// expensive to call. It is recommended to continue using the returned
    /// instance as long as possible (instead of the original [`OsStr`]), to
    /// avoid repeated conversions.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// println!("{:?}", RawOsStr::new(&os_string));
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn new(string: &OsStr) -> Cow<'_, Self> {
        match imp::os_str_to_bytes(string) {
            Cow::Borrowed(string) => Cow::Borrowed(Self::from_inner(string)),
            Cow::Owned(string) => Cow::Owned(RawOsString(string)),
        }
    }

    /// Wraps a string, without copying or encoding conversion.
    ///
    /// This method is much more efficient than [`RawOsStr::new`], since the
    /// [encoding] used by this crate is compatible with UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let string = "foobar";
    /// let raw = RawOsStr::from_str(string);
    /// assert_eq!(string, raw);
    /// ```
    ///
    /// [encoding]: super#encoding
    #[allow(clippy::should_implement_trait)]
    #[inline]
    #[must_use]
    pub fn from_str(string: &str) -> &Self {
        Self::from_inner(string.as_bytes())
    }

    /// Wraps a byte string, without copying or encoding conversion.
    ///
    /// # Panics
    ///
    /// Panics if the string is not valid for the [unspecified encoding] used
    /// by this crate.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// let raw_bytes = raw.as_raw_bytes();
    /// assert_eq!(&*raw, RawOsStr::assert_from_raw_bytes(raw_bytes));
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [unspecified encoding]: super#encoding
    #[inline]
    #[must_use = "method should not be used for validation"]
    #[track_caller]
    pub fn assert_from_raw_bytes(string: &[u8]) -> &Self {
        expect_encoded!(raw::validate_bytes(string));

        Self::from_inner(string)
    }

    if_checked_conversions! {
        /// Wraps a byte string, without copying or encoding conversion.
        ///
        /// [`assert_from_raw_bytes`] should almost always be used instead. For
        /// more information, see [`EncodingError`].
        ///
        /// # Errors
        ///
        /// See documentation for [`EncodingError`].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// # use std::io;
        ///
        /// use os_str_bytes::RawOsStr;
        ///
        /// let os_string = env::current_exe()?.into_os_string();
        /// let raw = RawOsStr::new(&os_string);
        /// assert_eq!(Ok(&*raw), RawOsStr::from_raw_bytes(raw.as_raw_bytes()));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [`assert_from_raw_bytes`]: Self::assert_from_raw_bytes
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(feature = "checked_conversions"))
        )]
        #[inline]
        pub fn from_raw_bytes(string: &[u8]) -> Result<&Self> {
            raw::validate_bytes(string)
                .map(|()| Self::from_inner(string))
                .map_err(EncodingError)
        }
    }

    /// Wraps a byte string, without copying or encoding conversion.
    ///
    /// # Safety
    ///
    /// The string must be valid for the [unspecified encoding] used by this
    /// crate.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// let raw_bytes = raw.as_raw_bytes();
    /// assert_eq!(&*raw, unsafe {
    ///     RawOsStr::from_raw_bytes_unchecked(raw_bytes)
    /// });
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [unspecified encoding]: super#encoding
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn from_raw_bytes_unchecked(string: &[u8]) -> &Self {
        if cfg!(debug_assertions) {
            expect_encoded!(raw::validate_bytes(string));
        }

        Self::from_inner(string)
    }

    /// Returns the byte string stored by this container.
    ///
    /// The returned string will use an [unspecified encoding].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let string = "foobar";
    /// let raw = RawOsStr::from_str(string);
    /// assert_eq!(string.as_bytes(), raw.as_raw_bytes());
    /// ```
    ///
    /// [unspecified encoding]: super#encoding
    #[inline]
    #[must_use]
    pub fn as_raw_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Equivalent to [`str::contains`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.contains("oo"));
    /// assert!(!raw.contains("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn contains<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        self.find(pat).is_some()
    }

    /// Equivalent to [`str::ends_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.ends_with("bar"));
    /// assert!(!raw.ends_with("foo"));
    /// ```
    #[inline]
    #[must_use]
    pub fn ends_with<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__get();

        self.0.ends_with(pat)
    }

    /// Equivalent to [`str::ends_with`] but accepts this type for the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.ends_with_os(RawOsStr::from_str("bar")));
    /// assert!(!raw.ends_with_os(RawOsStr::from_str("foo")));
    /// ```
    #[inline]
    #[must_use]
    pub fn ends_with_os(&self, pat: &Self) -> bool {
        raw::ends_with(&self.0, &pat.0)
    }

    /// Equivalent to [`str::find`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(Some(1), raw.find("o"));
    /// assert_eq!(None, raw.find("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn find<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__get();

        find(&self.0, pat)
    }

    /// Equivalent to [`str::is_empty`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// assert!(RawOsStr::from_str("").is_empty());
    /// assert!(!RawOsStr::from_str("foobar").is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the length of the byte string stored by this container.
    ///
    /// Only the following assumptions can be made about the result:
    /// - The length of any Unicode character is the length of its UTF-8
    ///   representation (i.e., [`char::len_utf8`]).
    /// - Splitting a string at a UTF-8 boundary will return two strings with
    ///   lengths that sum to the length of the original string.
    ///
    /// This method may return a different result than would [`OsStr::len`]
    /// when called on same string, since [`OsStr`] uses an unspecified
    /// encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// assert_eq!(6, RawOsStr::from_str("foobar").raw_len());
    /// assert_eq!(0, RawOsStr::from_str("").raw_len());
    /// ```
    #[inline]
    #[must_use]
    pub fn raw_len(&self) -> usize {
        self.0.len()
    }

    /// Equivalent to [`str::rfind`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(Some(2), raw.rfind("o"));
    /// assert_eq!(None, raw.rfind("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn rfind<P>(&self, pat: P) -> Option<usize>
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__get();

        rfind(&self.0, pat)
    }

    fn split_once_raw_with<P, F>(
        &self,
        pat: &P,
        find_fn: F,
    ) -> Option<(&Self, &Self)>
    where
        F: FnOnce(&[u8], &[u8]) -> Option<usize>,
        P: EncodedPattern,
    {
        let pat = pat.__get();

        let index = find_fn(&self.0, pat)?;
        let prefix = &self.0[..index];
        let suffix = &self.0[index + pat.len()..];
        Some((Self::from_inner(prefix), Self::from_inner(suffix)))
    }

    pub(super) fn rsplit_once_raw<P>(&self, pat: &P) -> Option<(&Self, &Self)>
    where
        P: EncodedPattern,
    {
        self.split_once_raw_with(pat, rfind)
    }

    /// Equivalent to [`str::rsplit_once`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(
    ///     Some((RawOsStr::from_str("fo"), RawOsStr::from_str("bar"))),
    ///     raw.rsplit_once("o"),
    /// );
    /// assert_eq!(None, raw.rsplit_once("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn rsplit_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern,
    {
        self.rsplit_once_raw(&pat.__encode())
    }

    // https://github.com/rust-lang/rust/blob/49c68bd53f90e375bfb3cbba8c1c67a9e0adb9c0/src/libcore/str/mod.rs#L2184-L2221
    #[cold]
    #[inline(never)]
    #[track_caller]
    fn index_boundary_error(&self, index: usize) -> ! {
        debug_assert!(raw::is_continuation(self.0[index]));

        let start = expect_encoded!(self.0[..index]
            .iter()
            .rposition(|&x| !raw::is_continuation(x)));
        let mut end = index + 1;
        end += self.0[end..]
            .iter()
            .take_while(|&&x| raw::is_continuation(x))
            .count();
        let code_point = raw::decode_code_point(&self.0[start..end]);
        panic!(
            "byte index {} is not a valid boundary; it is inside U+{:04X} \
             (bytes {}..{})",
            index, code_point, start, end,
        );
    }

    #[track_caller]
    fn check_bound(&self, index: usize) {
        if let Some(&byte) = self.0.get(index) {
            if raw::is_continuation(byte) {
                self.index_boundary_error(index);
            }
        }
    }

    /// Equivalent to [`str::split`], but empty patterns are not accepted.
    ///
    /// # Panics
    ///
    /// Panics if the pattern is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.split("o").eq(["f", "", "bar"]));
    /// ```
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn split<P>(&self, pat: P) -> Split<'_, P>
    where
        P: Pattern,
    {
        Split::new(self, pat)
    }

    /// Equivalent to [`str::split_at`].
    ///
    /// # Panics
    ///
    /// Panics if the index is not a [valid boundary].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(
    ///     ((RawOsStr::from_str("fo"), RawOsStr::from_str("obar"))),
    ///     raw.split_at(2),
    /// );
    /// ```
    ///
    /// [valid boundary]: #indices
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn split_at(&self, mid: usize) -> (&Self, &Self) {
        self.check_bound(mid);

        let (prefix, suffix) = self.0.split_at(mid);
        (Self::from_inner(prefix), Self::from_inner(suffix))
    }

    pub(super) fn split_once_raw<P>(&self, pat: &P) -> Option<(&Self, &Self)>
    where
        P: EncodedPattern,
    {
        self.split_once_raw_with(pat, find)
    }

    /// Equivalent to [`str::split_once`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(
    ///     Some((RawOsStr::from_str("f"), RawOsStr::from_str("obar"))),
    ///     raw.split_once("o"),
    /// );
    /// assert_eq!(None, raw.split_once("of"));
    /// ```
    #[inline]
    #[must_use]
    pub fn split_once<P>(&self, pat: P) -> Option<(&Self, &Self)>
    where
        P: Pattern,
    {
        self.split_once_raw(&pat.__encode())
    }

    /// Equivalent to [`str::starts_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.starts_with("foo"));
    /// assert!(!raw.starts_with("bar"));
    /// ```
    #[inline]
    #[must_use]
    pub fn starts_with<P>(&self, pat: P) -> bool
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__get();

        self.0.starts_with(pat)
    }

    /// Equivalent to [`str::starts_with`] but accepts this type for the
    /// pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert!(raw.starts_with_os(RawOsStr::from_str("foo")));
    /// assert!(!raw.starts_with_os(RawOsStr::from_str("bar")));
    /// ```
    #[inline]
    #[must_use]
    pub fn starts_with_os(&self, pat: &Self) -> bool {
        raw::starts_with(&self.0, &pat.0)
    }

    /// Equivalent to [`str::strip_prefix`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!(
    ///     Some(RawOsStr::from_str("11foo1bar111")),
    ///     raw.strip_prefix("1"),
    /// );
    /// assert_eq!(None, raw.strip_prefix("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn strip_prefix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__get();

        self.0.strip_prefix(pat).map(Self::from_inner)
    }

    /// Equivalent to [`str::strip_suffix`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!(
    ///     Some(RawOsStr::from_str("111foo1bar11")),
    ///     raw.strip_suffix("1"),
    /// );
    /// assert_eq!(None, raw.strip_suffix("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn strip_suffix<P>(&self, pat: P) -> Option<&Self>
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        let pat = pat.__get();

        self.0.strip_suffix(pat).map(Self::from_inner)
    }

    /// Converts this representation back to a platform-native string.
    ///
    /// When possible, use [`RawOsStrCow::into_os_str`] for a more efficient
    /// conversion on some platforms.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// assert_eq!(os_string, raw.to_os_str());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn to_os_str(&self) -> Cow<'_, OsStr> {
        expect_encoded!(imp::os_str_from_bytes(&self.0))
    }

    /// Equivalent to [`OsStr::to_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let string = "foobar";
    /// let raw = RawOsStr::from_str(string);
    /// assert_eq!(Some(string), raw.to_str());
    /// ```
    #[inline]
    #[must_use]
    pub fn to_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }

    /// Converts this string to the best UTF-8 representation possible.
    ///
    /// Invalid sequences will be replaced with
    /// [`char::REPLACEMENT_CHARACTER`].
    ///
    /// This method may return a different result than would
    /// [`OsStr::to_string_lossy`] when called on same string, since [`OsStr`]
    /// uses an unspecified encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// println!("{}", raw.to_str_lossy());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn to_str_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.0)
    }

    fn trim_matches_raw_with<P, F>(&self, pat: &P, strip_fn: F) -> &Self
    where
        F: for<'a> Fn(&'a [u8], &[u8]) -> Option<&'a [u8]>,
        P: EncodedPattern,
    {
        let pat = pat.__get();
        if pat.is_empty() {
            return self;
        }

        let mut string = &self.0;
        while let Some(substring) = strip_fn(string, pat) {
            string = substring;
        }
        Self::from_inner(string)
    }

    fn trim_end_matches_raw<P>(&self, pat: &P) -> &Self
    where
        P: EncodedPattern,
    {
        self.trim_matches_raw_with(pat, <[_]>::strip_suffix)
    }

    /// Equivalent to [`str::trim_end_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!("111foo1bar", raw.trim_end_matches("1"));
    /// assert_eq!("111foo1bar111", raw.trim_end_matches("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn trim_end_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        self.trim_end_matches_raw(&pat.__encode())
    }

    /// Equivalent to [`str::trim_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!("foo1bar", raw.trim_matches("1"));
    /// assert_eq!("111foo1bar111", raw.trim_matches("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn trim_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        let pat = pat.__encode();
        self.trim_start_matches_raw(&pat).trim_end_matches_raw(&pat)
    }

    fn trim_start_matches_raw<P>(&self, pat: &P) -> &Self
    where
        P: EncodedPattern,
    {
        self.trim_matches_raw_with(pat, <[_]>::strip_prefix)
    }

    /// Equivalent to [`str::trim_start_matches`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("111foo1bar111");
    /// assert_eq!("foo1bar111", raw.trim_start_matches("1"));
    /// assert_eq!("111foo1bar111", raw.trim_start_matches("o"));
    /// ```
    #[inline]
    #[must_use]
    pub fn trim_start_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        self.trim_start_matches_raw(&pat.__encode())
    }
}

impl AsRef<Self> for RawOsStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<RawOsStr> for str {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        RawOsStr::from_str(self)
    }
}

impl AsRef<RawOsStr> for String {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        (**self).as_ref()
    }
}

impl Default for &RawOsStr {
    #[inline]
    fn default() -> Self {
        RawOsStr::from_str("")
    }
}

impl<'a> From<&'a RawOsStr> for Cow<'a, RawOsStr> {
    #[inline]
    fn from(value: &'a RawOsStr) -> Self {
        Cow::Borrowed(value)
    }
}

impl From<Box<str>> for Box<RawOsStr> {
    #[inline]
    fn from(value: Box<str>) -> Self {
        value.into_boxed_bytes().transmute_box()
    }
}

impl ToOwned for RawOsStr {
    type Owned = RawOsString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        RawOsString(self.0.to_owned())
    }
}

/// Extensions to [`Cow<RawOsStr>`] for additional conversions.
///
/// [`Cow<RawOsStr>`]: Cow
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub trait RawOsStrCow<'a>: private::Sealed {
    /// Converts this representation back to a platform-native string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsStr;
    /// use os_str_bytes::RawOsStrCow;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// assert_eq!(os_string, raw.into_os_str());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[must_use]
    fn into_os_str(self) -> Cow<'a, OsStr>;

    /// Returns the byte string stored by this container.
    ///
    /// The returned string will use an [unspecified encoding].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::borrow::Cow;
    ///
    /// use os_str_bytes::RawOsStr;
    /// use os_str_bytes::RawOsStrCow;
    ///
    /// let string = "foobar";
    /// let raw = Cow::Borrowed(RawOsStr::from_str(string));
    /// assert_eq!(string.as_bytes(), &*raw.into_raw_bytes());
    /// ```
    ///
    /// [unspecified encoding]: super#encoding
    #[must_use]
    fn into_raw_bytes(self) -> Cow<'a, [u8]>;
}

impl<'a> RawOsStrCow<'a> for Cow<'a, RawOsStr> {
    #[inline]
    fn into_os_str(self) -> Cow<'a, OsStr> {
        match self {
            Cow::Borrowed(string) => string.to_os_str(),
            Cow::Owned(string) => Cow::Owned(string.into_os_string()),
        }
    }

    #[inline]
    fn into_raw_bytes(self) -> Cow<'a, [u8]> {
        match self {
            Cow::Borrowed(string) => Cow::Borrowed(&string.0),
            Cow::Owned(string) => Cow::Owned(string.0),
        }
    }
}

/// A container for owned byte strings converted by this crate.
///
/// For more information, see [`RawOsStr`].
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub struct RawOsString(Vec<u8>);

impl RawOsString {
    /// Converts a platform-native string into a representation that can be
    /// more easily manipulated.
    ///
    /// For more information, see [`RawOsStr::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// println!("{:?}", RawOsString::new(os_string));
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn new(string: OsString) -> Self {
        Self(imp::os_string_into_vec(string))
    }

    /// Wraps a string, without copying or encoding conversion.
    ///
    /// This method is much more efficient than [`RawOsString::new`], since the
    /// [encoding] used by this crate is compatible with UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let raw = RawOsString::from_string(string.clone());
    /// assert_eq!(string, raw);
    /// ```
    ///
    /// [encoding]: super#encoding
    #[inline]
    #[must_use]
    pub fn from_string(string: String) -> Self {
        Self(string.into_bytes())
    }

    /// Wraps a byte string, without copying or encoding conversion.
    ///
    /// # Panics
    ///
    /// Panics if the string is not valid for the [unspecified encoding] used
    /// by this crate.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsString::new(os_string);
    /// let raw_bytes = raw.clone().into_raw_vec();
    /// assert_eq!(raw, RawOsString::assert_from_raw_vec(raw_bytes));
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [unspecified encoding]: super#encoding
    #[inline]
    #[must_use = "method should not be used for validation"]
    #[track_caller]
    pub fn assert_from_raw_vec(string: Vec<u8>) -> Self {
        expect_encoded!(raw::validate_bytes(&string));

        Self(string)
    }

    if_checked_conversions! {
        /// Wraps a byte string, without copying or encoding conversion.
        ///
        /// [`assert_from_raw_vec`] should almost always be used instead. For
        /// more information, see [`EncodingError`].
        ///
        /// # Errors
        ///
        /// See documentation for [`EncodingError`].
        ///
        /// # Examples
        ///
        /// ```
        /// use std::env;
        /// # use std::io;
        ///
        /// use os_str_bytes::RawOsString;
        ///
        /// let os_string = env::current_exe()?.into_os_string();
        /// let raw = RawOsString::new(os_string);
        /// let raw_clone = raw.clone();
        /// assert_eq!(Ok(raw), RawOsString::from_raw_vec(raw_clone.into_raw_vec()));
        /// #
        /// # Ok::<_, io::Error>(())
        /// ```
        ///
        /// [`assert_from_raw_vec`]: Self::assert_from_raw_vec
        #[cfg_attr(
            os_str_bytes_docs_rs,
            doc(cfg(feature = "checked_conversions"))
        )]
        #[inline]
        pub fn from_raw_vec(string: Vec<u8>) -> Result<Self> {
            raw::validate_bytes(&string)
                .map(|()| Self(string))
                .map_err(EncodingError)
        }
    }

    /// Wraps a byte string, without copying or encoding conversion.
    ///
    /// # Safety
    ///
    /// The string must be valid for the [unspecified encoding] used by this
    /// crate.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsString::new(os_string);
    /// let raw_bytes = raw.clone().into_raw_vec();
    /// assert_eq!(raw, unsafe {
    ///     RawOsString::from_raw_vec_unchecked(raw_bytes)
    /// });
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    ///
    /// [unspecified encoding]: super#encoding
    #[inline]
    #[must_use]
    #[track_caller]
    pub unsafe fn from_raw_vec_unchecked(string: Vec<u8>) -> Self {
        if cfg!(debug_assertions) {
            expect_encoded!(raw::validate_bytes(&string));
        }

        Self(string)
    }

    /// Equivalent to [`String::clear`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let mut raw = RawOsString::new(os_string);
    /// raw.clear();
    /// assert!(raw.is_empty());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Equivalent to [`String::into_boxed_str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let raw = RawOsString::from_string(string.clone());
    /// assert_eq!(string, *raw.into_box());
    /// ```
    #[inline]
    #[must_use]
    pub fn into_box(self) -> Box<RawOsStr> {
        self.0.into_boxed_slice().transmute_box()
    }

    /// Converts this representation back to a platform-native string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsString::new(os_string.clone());
    /// assert_eq!(os_string, raw.into_os_string());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn into_os_string(self) -> OsString {
        expect_encoded!(imp::os_string_from_vec(self.0))
    }

    /// Returns the byte string stored by this container.
    ///
    /// The returned string will use an [unspecified encoding].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let raw = RawOsString::from_string(string.clone());
    /// assert_eq!(string.into_bytes(), raw.into_raw_vec());
    /// ```
    ///
    /// [unspecified encoding]: super#encoding
    #[inline]
    #[must_use]
    pub fn into_raw_vec(self) -> Vec<u8> {
        self.0
    }

    /// Equivalent to [`OsString::into_string`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let raw = RawOsString::from_string(string.clone());
    /// assert_eq!(Ok(string), raw.into_string());
    /// ```
    #[inline]
    pub fn into_string(self) -> result::Result<String, Self> {
        String::from_utf8(self.0).map_err(|x| Self(x.into_bytes()))
    }

    /// Equivalent to [`String::shrink_to_fit`].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let string = "foobar".to_owned();
    /// let mut raw = RawOsString::from_string(string.clone());
    /// raw.shrink_to_fit();
    /// assert_eq!(string, raw);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// Equivalent to [`String::split_off`].
    ///
    /// # Panics
    ///
    /// Panics if the index is not a [valid boundary].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let mut raw = RawOsString::from_string("foobar".to_owned());
    /// assert_eq!("bar", raw.split_off(3));
    /// assert_eq!("foo", raw);
    /// ```
    ///
    /// [valid boundary]: RawOsStr#indices
    #[inline]
    #[must_use]
    #[track_caller]
    pub fn split_off(&mut self, at: usize) -> Self {
        self.check_bound(at);

        Self(self.0.split_off(at))
    }

    /// Equivalent to [`String::truncate`].
    ///
    /// # Panics
    ///
    /// Panics if the index is not a [valid boundary].
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsString;
    ///
    /// let mut raw = RawOsString::from_string("foobar".to_owned());
    /// raw.truncate(3);
    /// assert_eq!("foo", raw);
    /// ```
    ///
    /// [valid boundary]: RawOsStr#indices
    #[inline]
    #[track_caller]
    pub fn truncate(&mut self, new_len: usize) {
        self.check_bound(new_len);

        self.0.truncate(new_len);
    }
}

impl AsRef<RawOsStr> for RawOsString {
    #[inline]
    fn as_ref(&self) -> &RawOsStr {
        self
    }
}

impl Borrow<RawOsStr> for RawOsString {
    #[inline]
    fn borrow(&self) -> &RawOsStr {
        self
    }
}

impl Deref for RawOsString {
    type Target = RawOsStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        RawOsStr::from_inner(&self.0)
    }
}

impl From<RawOsString> for Box<RawOsStr> {
    #[inline]
    fn from(value: RawOsString) -> Self {
        value.into_box()
    }
}

impl From<Box<RawOsStr>> for RawOsString {
    #[inline]
    fn from(value: Box<RawOsStr>) -> Self {
        Self(value.transmute_box::<[_]>().into_vec())
    }
}

impl From<RawOsString> for Cow<'_, RawOsStr> {
    #[inline]
    fn from(value: RawOsString) -> Self {
        Cow::Owned(value)
    }
}

impl From<String> for RawOsString {
    #[inline]
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

struct DebugBuffer<'a>(&'a [u8]);

impl Debug for DebugBuffer<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;

        let mut string = self.0;
        let mut invalid_length = 0;
        while !string.is_empty() {
            let (invalid, substring) = string.split_at(invalid_length);

            let valid = match str::from_utf8(substring) {
                Ok(valid) => {
                    string = &[];
                    valid
                }
                Err(error) => {
                    let (valid, substring) =
                        substring.split_at(error.valid_up_to());

                    let invalid_char_length =
                        error.error_len().unwrap_or_else(|| substring.len());
                    if valid.is_empty() {
                        invalid_length += invalid_char_length;
                        continue;
                    }
                    string = substring;
                    invalid_length = invalid_char_length;

                    // SAFETY: This slice was validated to be UTF-8.
                    unsafe { str::from_utf8_unchecked(valid) }
                }
            };

            raw::debug(invalid, f)?;
            Display::fmt(&valid.escape_debug(), f)?;
        }

        f.write_str("\"")
    }
}

macro_rules! r#impl {
    ( $type:ty ) => {
        impl Debug for $type {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($type))
                    .field(&DebugBuffer(&self.0))
                    .finish()
            }
        }
    };
}
r#impl!(RawOsStr);
r#impl!(RawOsString);

macro_rules! r#impl {
    ( $index_type:ty $(, $index_var:ident , $($bound:expr),+)? ) => {
        impl Index<$index_type> for RawOsStr {
            type Output = Self;

            #[inline]
            fn index(&self, idx: $index_type) -> &Self::Output {
                $(
                    let $index_var = &idx;
                    $(self.check_bound($bound);)+
                )?

                Self::from_inner(&self.0[idx])
            }
        }

        impl Index<$index_type> for RawOsString {
            type Output = RawOsStr;

            #[allow(clippy::indexing_slicing)]
            #[inline]
            fn index(&self, idx: $index_type) -> &Self::Output {
                &(**self)[idx]
            }
        }
    };
}
r#impl!(Range<usize>, x, x.start, x.end);
r#impl!(RangeFrom<usize>, x, x.start);
r#impl!(RangeFull);
// [usize::MAX] will always be a valid inclusive end index.
#[rustfmt::skip]
r#impl!(RangeInclusive<usize>, x, *x.start(), x.end().wrapping_add(1));
r#impl!(RangeTo<usize>, x, x.end);
r#impl!(RangeToInclusive<usize>, x, x.end.wrapping_add(1));

macro_rules! r#impl {
    ( $type:ty , $other_type:ty ) => {
        impl PartialEq<$other_type> for $type {
            #[inline]
            fn eq(&self, other: &$other_type) -> bool {
                let raw: &RawOsStr = self;
                let other: &RawOsStr = other.as_ref();
                raw == other
            }
        }

        impl PartialEq<$type> for $other_type {
            #[inline]
            fn eq(&self, other: &$type) -> bool {
                other == self
            }
        }
    };
}
r#impl!(RawOsStr, RawOsString);
r#impl!(&RawOsStr, RawOsString);
r#impl!(RawOsStr, str);
r#impl!(RawOsStr, String);
r#impl!(&RawOsStr, String);
r#impl!(RawOsString, str);
r#impl!(RawOsString, &str);
r#impl!(RawOsString, String);

#[cfg(feature = "print_bytes")]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "print_bytes")))]
mod print_bytes {
    use print_bytes::ByteStr;
    use print_bytes::ToBytes;
    #[cfg(windows)]
    use print_bytes::WideStr;

    #[cfg(windows)]
    use crate::imp::raw;

    use super::RawOsStr;
    use super::RawOsString;

    impl ToBytes for RawOsStr {
        #[inline]
        fn to_bytes(&self) -> ByteStr<'_> {
            self.0.to_bytes()
        }

        #[cfg(windows)]
        #[inline]
        fn to_wide(&self) -> Option<WideStr> {
            Some(WideStr::new(raw::encode_wide_unchecked(&self.0).collect()))
        }
    }

    impl ToBytes for RawOsString {
        #[inline]
        fn to_bytes(&self) -> ByteStr<'_> {
            (**self).to_bytes()
        }

        #[cfg(windows)]
        #[inline]
        fn to_wide(&self) -> Option<WideStr> {
            (**self).to_wide()
        }
    }
}

#[cfg(feature = "uniquote")]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "uniquote")))]
mod uniquote {
    use uniquote::Formatter;
    use uniquote::Quote;
    use uniquote::Result;

    use crate::imp::raw;

    use super::RawOsStr;
    use super::RawOsString;

    impl Quote for RawOsStr {
        #[inline]
        fn escape(&self, f: &mut Formatter<'_>) -> Result {
            raw::uniquote::escape(&self.0, f)
        }
    }

    impl Quote for RawOsString {
        #[inline]
        fn escape(&self, f: &mut Formatter<'_>) -> Result {
            (**self).escape(f)
        }
    }
}
