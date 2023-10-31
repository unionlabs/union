//! Structured keys.

use std::borrow::Borrow;
use std::cmp;
use std::fmt;
use std::hash;

/// A type that can be converted into a [`Key`](struct.Key.html).
pub trait ToKey {
    /// Perform the conversion.
    fn to_key(&self) -> Key;
}

impl<'a, T> ToKey for &'a T
where
    T: ToKey + ?Sized,
{
    fn to_key(&self) -> Key {
        (**self).to_key()
    }
}

impl<'k> ToKey for Key<'k> {
    fn to_key(&self) -> Key {
        Key { key: self.key }
    }
}

impl ToKey for str {
    fn to_key(&self) -> Key {
        Key::from_str(self)
    }
}

/// A key in a structured key-value pair.
#[derive(Clone)]
pub struct Key<'k> {
    key: &'k str,
}

impl<'k> Key<'k> {
    /// Get a key from a borrowed string.
    pub fn from_str(key: &'k str) -> Self {
        Key { key: key }
    }

    /// Get a borrowed string from this key.
    pub fn as_str(&self) -> &str {
        self.key
    }
}

impl<'k> fmt::Debug for Key<'k> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.key.fmt(f)
    }
}

impl<'k> fmt::Display for Key<'k> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.key.fmt(f)
    }
}

impl<'k> hash::Hash for Key<'k> {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_str().hash(state)
    }
}

impl<'k, 'ko> PartialEq<Key<'ko>> for Key<'k> {
    fn eq(&self, other: &Key<'ko>) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl<'k> Eq for Key<'k> {}

impl<'k, 'ko> PartialOrd<Key<'ko>> for Key<'k> {
    fn partial_cmp(&self, other: &Key<'ko>) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'k> Ord for Key<'k> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<'k> AsRef<str> for Key<'k> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'k> Borrow<str> for Key<'k> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'k> From<&'k str> for Key<'k> {
    fn from(s: &'k str) -> Self {
        Key::from_str(s)
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::borrow::Cow;

    impl ToKey for String {
        fn to_key(&self) -> Key {
            Key::from_str(self)
        }
    }

    impl<'a> ToKey for Cow<'a, str> {
        fn to_key(&self) -> Key {
            Key::from_str(self)
        }
    }
}

#[cfg(feature = "kv_unstable_sval")]
mod sval_support {
    use super::*;

    extern crate sval;

    use self::sval::value::{self, Value};

    impl<'a> Value for Key<'a> {
        fn stream(&self, stream: &mut value::Stream) -> value::Result {
            self.key.stream(stream)
        }
    }
}

#[cfg(feature = "kv_unstable_serde")]
mod serde_support {
    use super::*;

    extern crate serde;

    use self::serde::{Serialize, Serializer};

    impl<'a> Serialize for Key<'a> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.key.serialize(serializer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_from_string() {
        assert_eq!("a key", Key::from_str("a key").as_str());
    }
}
