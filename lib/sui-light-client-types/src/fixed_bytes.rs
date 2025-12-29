use core::fmt::Display;

use unionlabs_primitives::{Bytes, FixedBytes, encoding::Encoding};

pub struct SuiFixedBytes<const N: usize, E: Encoding>(pub FixedBytes<N, E>);

impl<const N: usize, E: Encoding> SuiFixedBytes<N, E> {
    pub fn new(bytes: FixedBytes<N, E>) -> Self {
        Self(bytes)
    }
}

impl<const N: usize, E: Encoding> std::fmt::Debug for SuiFixedBytes<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SuiFixedBytes").field(&self.0).finish()
    }
}

impl<const N: usize, E: Encoding> Eq for SuiFixedBytes<N, E> {}

impl<const N: usize, E: Encoding> PartialEq for SuiFixedBytes<N, E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<const N: usize, E: Encoding> Clone for SuiFixedBytes<N, E> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

#[cfg(feature = "serde")]
impl<const N: usize, E: Encoding> serde::Serialize for SuiFixedBytes<N, E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes = Bytes::<E>::new(self.0.into_bytes());

        bytes.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, const N: usize, E: Encoding> serde::Deserialize<'de> for SuiFixedBytes<N, E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Bytes::<E>::deserialize(deserializer)?;

        Ok(Self(FixedBytes::new(bytes.as_ref().try_into().unwrap())))
    }
}

impl<const N: usize, E: Encoding> Display for SuiFixedBytes<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "bincode")]
impl<const N: usize, Enc: Encoding> bincode::Encode for SuiFixedBytes<N, Enc> {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.encode(encoder)
    }
}

#[cfg(feature = "bincode")]
impl<const N: usize, Enc: Encoding, Context> bincode::Decode<Context> for SuiFixedBytes<N, Enc> {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Ok(Self(
            <FixedBytes<N, Enc> as bincode::Decode<Context>>::decode(decoder)?,
        ))
    }
}

#[cfg(feature = "bincode")]
impl<'de, const N: usize, Enc: Encoding, Context> bincode::BorrowDecode<'de, Context>
    for SuiFixedBytes<N, Enc>
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        bincode::Decode::decode(decoder)
    }
}
