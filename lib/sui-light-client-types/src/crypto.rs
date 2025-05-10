use core::fmt::{Debug, Display};
use std::io;

use roaring::RoaringBitmap;
use unionlabs_primitives::{encoding::Base64, FixedBytes};

use crate::checkpoint_summary::EpochId;

pub const BLS_G1_SIZE: usize = 48;
pub const BLS_G2_SIZE: usize = 96;
pub const BLS_DST: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

pub type AuthorityPublicKeyBytes = CryptoBytes<BLS_G2_SIZE>;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct CryptoBytes<const N: usize>(pub FixedBytes<N, Base64>);

impl<const N: usize> Display for CryptoBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type AuthorityStrongQuorumSignInfo = AuthorityQuorumSignInfo<true>;
pub type AggregateAuthoritySignature = CryptoBytes<BLS_G1_SIZE>;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct AuthorityQuorumSignInfo<const STRONG_THRESHOLD: bool> {
    pub epoch: EpochId,
    pub signature: AggregateAuthoritySignature,
    pub signers_map: SuiBitmap,
}

#[derive(Clone, Debug)]
pub struct SuiBitmap(pub RoaringBitmap);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for SuiBitmap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
        Ok(Self::deserialize_from_bytes(&bytes).map_err(|e| {
            serde::de::Error::custom(format!("byte deserialization failed, cause by: {:?}", e))
        })?)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for SuiBitmap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error as _;
        let bytes = self.serialize_to_bytes().map_err(|e| {
            S::Error::custom(format!("byte serialization failed, cause by: {:?}", e))
        })?;

        Vec::<u8>::serialize(&bytes, serializer)
    }
}

#[cfg(feature = "bincode")]
impl bincode::Encode for SuiBitmap {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let bytes = self
            .serialize_to_bytes()
            .map_err(|e| bincode::error::EncodeError::OtherString(e.to_string()))?;

        bytes.encode(encoder)
    }
}

#[cfg(feature = "bincode")]
impl<Context> bincode::Decode<Context> for SuiBitmap {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let bytes = Vec::<u8>::decode(decoder)?;

        Ok(Self::deserialize_from_bytes(&bytes)
            .map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))?)
    }
}

#[cfg(feature = "bincode")]
impl<'de, Context> bincode::BorrowDecode<'de, Context> for SuiBitmap {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        bincode::Decode::decode(decoder)
    }
}

impl SuiBitmap {
    pub fn serialize_to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![];

        self.0.serialize_into(&mut bytes)?;

        Ok(bytes)
    }

    // RoaringBitmap::deserialize_from() or iter() do not check for duplicates.
    // So this function is needed to sanitize the bitmap to ensure unique entries.
    pub fn deserialize_from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let orig_bitmap = roaring::RoaringBitmap::deserialize_from(bytes)?;
        // Ensure there is no duplicated entries in the bitmap.
        let mut seen = std::collections::BTreeSet::new();
        let mut new_bitmap = roaring::RoaringBitmap::new();
        for v in orig_bitmap.iter() {
            if seen.insert(v) {
                new_bitmap.insert(v);
            }
        }
        Ok(Self(new_bitmap))
    }
}
