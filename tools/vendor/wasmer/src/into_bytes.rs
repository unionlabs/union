use bytes::Bytes;
use std::borrow::Cow;

/// Convert binary data into [`bytes::Bytes`].
pub trait IntoBytes {
    /// Convert binary data into [`bytes::Bytes`].
    fn into_bytes(self) -> Bytes;
}

impl IntoBytes for Bytes {
    fn into_bytes(self) -> Bytes {
        self
    }
}

impl IntoBytes for Vec<u8> {
    fn into_bytes(self) -> Bytes {
        Bytes::from(self)
    }
}

impl IntoBytes for &Vec<u8> {
    fn into_bytes(self) -> Bytes {
        Bytes::from(self.clone())
    }
}

impl IntoBytes for &[u8] {
    fn into_bytes(self) -> Bytes {
        Bytes::from(self.to_vec())
    }
}

impl<const N: usize> IntoBytes for &[u8; N] {
    fn into_bytes(self) -> Bytes {
        Bytes::from(self.to_vec())
    }
}

impl IntoBytes for &str {
    fn into_bytes(self) -> Bytes {
        Bytes::from(self.as_bytes().to_vec())
    }
}

impl IntoBytes for Cow<'_, [u8]> {
    fn into_bytes(self) -> Bytes {
        Bytes::from(self.to_vec())
    }
}
