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
use std::str;

#[cfg(feature = "memchr")]
use memchr::memmem::find;
#[cfg(feature = "memchr")]
use memchr::memmem::rfind;

use super::imp::raw;
use super::iter::Split;
use super::pattern::Encoded as EncodedPattern;
use super::OsStrBytes;
use super::OsStringBytes;
use super::Pattern;

#[cfg(not(feature = "memchr"))]
fn find(string: &[u8], pat: &[u8]) -> Option<usize> {
    for i in 0..=string.len().checked_sub(pat.len())? {
        if string[i..].starts_with(pat) {
            return Some(i);
        }
    }
    None
}

#[cfg(not(feature = "memchr"))]
fn rfind(string: &[u8], pat: &[u8]) -> Option<usize> {
    for i in (pat.len()..=string.len()).rev() {
        if string[..i].ends_with(pat) {
            return Some(i - pat.len());
        }
    }
    None
}

macro_rules! impl_trim_matches {
    ( $self:ident , $pat:expr , $strip_method:ident ) => {{
        let pat = $pat.__encode();
        let pat = pat.__get();
        if pat.is_empty() {
            return $self;
        }

        let mut string = &$self.0;
        while let Some(substring) = string.$strip_method(pat) {
            string = substring;
        }
        Self::from_raw_bytes_unchecked(string)
    }};
}

macro_rules! impl_split_once_raw {
    ( $self:ident , $pat:expr , $find_fn:expr ) => {{
        let pat = $pat.__get();

        let index = $find_fn(&$self.0, pat)?;
        let prefix = &$self.0[..index];
        let suffix = &$self.0[index + pat.len()..];
        Some((
            Self::from_raw_bytes_unchecked(prefix),
            Self::from_raw_bytes_unchecked(suffix),
        ))
    }};
}

/// A container for the byte strings converted by [`OsStrBytes`].
///
/// This wrapper is intended to prevent violating the invariants of the
/// [unspecified encoding] used by this crate and minimize encoding
/// conversions.
///
/// Although this type is annotated with `#[repr(transparent)]`, the inner
/// representation is not stable. Transmuting between this type and any other
/// causes immediate undefined behavior.
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
/// [memchr complexity]: memchr::memmem::find#complexity
/// [unspecified encoding]: super#encoding
#[derive(Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
#[repr(transparent)]
pub struct RawOsStr([u8]);

impl RawOsStr {
    fn from_raw_bytes_unchecked(string: &[u8]) -> &Self {
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
        match string.to_raw_bytes() {
            Cow::Borrowed(string) => {
                Cow::Borrowed(Self::from_raw_bytes_unchecked(string))
            }
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
        Self::from_raw_bytes_unchecked(string.as_bytes())
    }

    /// Returns the byte string stored by this container.
    ///
    /// The result will match what would be returned by
    /// [`OsStrBytes::to_raw_bytes`] for the same string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStrBytes;
    /// use os_str_bytes::RawOsStr;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsStr::new(&os_string);
    /// assert_eq!(os_string.to_raw_bytes(), raw.as_raw_bytes());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
    #[inline]
    #[must_use]
    pub fn as_raw_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Equivalent to [`str::contains`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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

    pub(super) fn rsplit_once_raw<P>(&self, pat: &P) -> Option<(&Self, &Self)>
    where
        P: EncodedPattern,
    {
        impl_split_once_raw!(self, pat, rfind)
    }

    /// Equivalent to [`str::rsplit_once`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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

        let start = self.0[..index]
            .iter()
            .rposition(|&x| !raw::is_continuation(x))
            .expect("invalid raw bytes");
        let mut end = index + 1;
        end += self.0[end..]
            .iter()
            .position(|&x| !raw::is_continuation(x))
            .unwrap_or_else(|| self.raw_len() - end);
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
    /// Panics if the pattern is a byte outside of the ASCII range or empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use os_str_bytes::RawOsStr;
    ///
    /// let raw = RawOsStr::from_str("foobar");
    /// assert_eq!(["f", "", "bar"], *raw.split("o").collect::<Vec<_>>());
    /// ```
    #[inline]
    #[must_use]
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
    pub fn split_at(&self, mid: usize) -> (&Self, &Self) {
        self.check_bound(mid);

        let (prefix, suffix) = self.0.split_at(mid);
        (
            Self::from_raw_bytes_unchecked(prefix),
            Self::from_raw_bytes_unchecked(suffix),
        )
    }

    pub(super) fn split_once_raw<P>(&self, pat: &P) -> Option<(&Self, &Self)>
    where
        P: EncodedPattern,
    {
        impl_split_once_raw!(self, pat, find)
    }

    /// Equivalent to [`str::split_once`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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

        self.0.strip_prefix(pat).map(Self::from_raw_bytes_unchecked)
    }

    /// Equivalent to [`str::strip_suffix`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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

        self.0.strip_suffix(pat).map(Self::from_raw_bytes_unchecked)
    }

    /// Converts this representation back to a platform-native string.
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
        OsStr::from_raw_bytes(&self.0).expect("invalid raw bytes")
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

    /// Equivalent to [`str::trim_end_matches`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    #[must_use]
    pub fn trim_end_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        impl_trim_matches!(self, pat, strip_suffix)
    }

    /// Equivalent to [`str::trim_start_matches`].
    ///
    /// # Panics
    ///
    /// Panics if the pattern is a byte outside of the ASCII range.
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
    #[must_use]
    pub fn trim_start_matches<P>(&self, pat: P) -> &Self
    where
        P: Pattern,
    {
        impl_trim_matches!(self, pat, strip_prefix)
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
    fn from(other: &'a RawOsStr) -> Self {
        Cow::Borrowed(other)
    }
}

macro_rules! r#impl {
    (
        $index_type:ty
        $(, $index_var:ident , $first_bound:expr $(, $second_bound:expr)?)?
    ) => {
        impl Index<$index_type> for RawOsStr {
            type Output = Self;

            #[inline]
            fn index(&self, idx: $index_type) -> &Self::Output {
                $(
                    let $index_var = &idx;
                    self.check_bound($first_bound);
                    $(self.check_bound($second_bound);)?
                )?

                Self::from_raw_bytes_unchecked(&self.0[idx])
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

impl ToOwned for RawOsStr {
    type Owned = RawOsString;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        RawOsString(self.0.to_owned())
    }
}

/// A container for the byte strings converted by [`OsStringBytes`].
///
/// For more information, see [`RawOsStr`].
///
/// [unspecified encoding]: super#encoding
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
        Self(string.into_raw_vec())
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
        OsString::from_raw_vec(self.0).expect("invalid raw bytes")
    }

    /// Returns the byte string stored by this container.
    ///
    /// The result will match what would be returned by
    /// [`OsStringBytes::into_raw_vec`] for the same string.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// # use std::io;
    ///
    /// use os_str_bytes::OsStringBytes;
    /// use os_str_bytes::RawOsString;
    ///
    /// let os_string = env::current_exe()?.into_os_string();
    /// let raw = RawOsString::new(os_string.clone());
    /// assert_eq!(os_string.into_raw_vec(), raw.into_raw_vec());
    /// #
    /// # Ok::<_, io::Error>(())
    /// ```
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
    pub fn into_string(self) -> Result<String, Self> {
        String::from_utf8(self.0).map_err(|x| Self(x.into_bytes()))
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
        RawOsStr::from_raw_bytes_unchecked(&self.0)
    }
}

impl From<String> for RawOsString {
    #[inline]
    fn from(other: String) -> Self {
        Self::from_string(other)
    }
}

impl From<RawOsString> for Cow<'_, RawOsStr> {
    #[inline]
    fn from(other: RawOsString) -> Self {
        Cow::Owned(other)
    }
}

macro_rules! r#impl {
    ( $index_type:ty ) => {
        impl Index<$index_type> for RawOsString {
            type Output = RawOsStr;

            #[inline]
            fn index(&self, idx: $index_type) -> &Self::Output {
                &(**self)[idx]
            }
        }
    };
}
r#impl!(Range<usize>);
r#impl!(RangeFrom<usize>);
r#impl!(RangeFull);
r#impl!(RangeInclusive<usize>);
r#impl!(RangeTo<usize>);
r#impl!(RangeToInclusive<usize>);

struct Buffer<'a>(&'a [u8]);

impl Debug for Buffer<'_> {
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
                    .field(&Buffer(&self.0))
                    .finish()
            }
        }
    };
}
r#impl!(RawOsStr);
r#impl!(RawOsString);

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
