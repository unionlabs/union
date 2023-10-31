use std::fmt::Debug;

use super::private;

pub trait Encoded {
    fn __get(&self) -> &[u8];
}

#[derive(Clone, Debug)]
pub struct EncodedChar {
    buffer: [u8; 4],
    length: usize,
}

impl Encoded for EncodedChar {
    fn __get(&self) -> &[u8] {
        &self.buffer[..self.length]
    }
}

impl Encoded for &str {
    fn __get(&self) -> &[u8] {
        self.as_bytes()
    }
}

/// Allows a type to be used for searching by [`RawOsStr`] and [`RawOsString`].
///
/// This trait is very similar to [`str::pattern::Pattern`], but its methods
/// are private and it is implemented for different types.
///
/// [`RawOsStr`]: super::RawOsStr
/// [`RawOsString`]: super::RawOsString
/// [`str::pattern::Pattern`]: ::std::str::pattern::Pattern
#[cfg_attr(os_str_bytes_docs_rs, doc(cfg(feature = "raw_os_str")))]
pub trait Pattern: private::Sealed {
    #[doc(hidden)]
    type __Encoded: Clone + Debug + Encoded;

    #[doc(hidden)]
    fn __encode(self) -> Self::__Encoded;
}

impl Pattern for char {
    type __Encoded = EncodedChar;

    fn __encode(self) -> Self::__Encoded {
        let mut encoded = EncodedChar {
            buffer: [0; 4],
            length: 0,
        };
        encoded.length = self.encode_utf8(&mut encoded.buffer).len();
        encoded
    }
}

impl Pattern for &str {
    type __Encoded = Self;

    fn __encode(self) -> Self::__Encoded {
        self
    }
}

impl<'a> Pattern for &'a String {
    type __Encoded = <&'a str as Pattern>::__Encoded;

    fn __encode(self) -> Self::__Encoded {
        (**self).__encode()
    }
}
