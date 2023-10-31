use core::ops;

/// A specialized copy-on-write byte string.
///
/// The purpose of this type is to permit usage of a "borrowed or owned
/// byte string" in a way that keeps std/no-std compatibility. That is, in
/// no-std mode, this type devolves into a simple &[u8] with no owned variant
/// available. We can't just use a plain Cow because Cow is not in core.
#[derive(Clone, Debug)]
pub struct CowBytes<'a>(Imp<'a>);

// N.B. We don't use std::borrow::Cow here since we can get away with a
// Box<[u8]> for our use case, which is 1/3 smaller than the Vec<u8> that
// a Cow<[u8]> would use.
#[cfg(feature = "std")]
#[derive(Clone, Debug)]
enum Imp<'a> {
    Borrowed(&'a [u8]),
    Owned(Box<[u8]>),
}

#[cfg(not(feature = "std"))]
#[derive(Clone, Debug)]
struct Imp<'a>(&'a [u8]);

impl<'a> ops::Deref for CowBytes<'a> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a> CowBytes<'a> {
    /// Create a new borrowed CowBytes.
    #[inline(always)]
    pub fn new<B: ?Sized + AsRef<[u8]>>(bytes: &'a B) -> CowBytes<'a> {
        CowBytes(Imp::new(bytes.as_ref()))
    }

    /// Create a new owned CowBytes.
    #[cfg(feature = "std")]
    #[inline(always)]
    pub fn new_owned(bytes: Box<[u8]>) -> CowBytes<'static> {
        CowBytes(Imp::Owned(bytes))
    }

    /// Return a borrowed byte string, regardless of whether this is an owned
    /// or borrowed byte string internally.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Return an owned version of this copy-on-write byte string.
    ///
    /// If this is already an owned byte string internally, then this is a
    /// no-op. Otherwise, the internal byte string is copied.
    #[cfg(feature = "std")]
    #[inline(always)]
    pub fn into_owned(self) -> CowBytes<'static> {
        match self.0 {
            Imp::Borrowed(b) => CowBytes::new_owned(Box::from(b)),
            Imp::Owned(b) => CowBytes::new_owned(b),
        }
    }
}

impl<'a> Imp<'a> {
    #[cfg(feature = "std")]
    #[inline(always)]
    pub fn new(bytes: &'a [u8]) -> Imp<'a> {
        Imp::Borrowed(bytes)
    }

    #[cfg(not(feature = "std"))]
    #[inline(always)]
    pub fn new(bytes: &'a [u8]) -> Imp<'a> {
        Imp(bytes)
    }

    #[cfg(feature = "std")]
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Imp::Owned(ref x) => x,
            Imp::Borrowed(x) => x,
        }
    }

    #[cfg(not(feature = "std"))]
    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        self.0
    }
}
