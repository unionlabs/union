use core::fmt::{Debug, Display};

use roaring::RoaringBitmap;
use serde::{ser::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use serde_with::{serde_as, DeserializeAs, SerializeAs};
use unionlabs_primitives::{encoding::Base64, Bytes, FixedBytes};

use crate::checkpoint_summary::EpochId;

pub const BLS_G1_SIZE: usize = 48;
pub const BLS_G2_SIZE: usize = 96;
pub const BLS_DST: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

pub type AuthorityPublicKeyBytes = CryptoBytes<BLS_G2_SIZE>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CryptoBytes<const N: usize>(pub FixedBytes<N, Base64>);

impl<const N: usize> Display for CryptoBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<const N: usize> Serialize for CryptoBytes<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Bytes::<Base64>::new(self.0.get().to_vec()).serialize(serializer)
    }
}

impl<'de, const N: usize> Deserialize<'de> for CryptoBytes<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let b = Bytes::<Base64>::deserialize(deserializer)?;

        Ok(Self(
            b.as_ref().try_into().map_err(serde::de::Error::custom)?,
        ))
    }
}

pub type AuthorityStrongQuorumSignInfo = AuthorityQuorumSignInfo<true>;
pub type AggregateAuthoritySignature = CryptoBytes<BLS_G1_SIZE>;

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthorityQuorumSignInfo<const STRONG_THRESHOLD: bool> {
    pub epoch: EpochId,
    pub signature: AggregateAuthoritySignature,
    #[serde_as(as = "SuiBitmap")]
    pub signers_map: RoaringBitmap,
}

pub(crate) struct SuiBitmap;

#[inline]
pub(crate) fn to_custom_deser_error<'de, D, E>(e: E) -> D::Error
where
    E: Debug,
    D: Deserializer<'de>,
{
    serde::de::Error::custom(format!("byte deserialization failed, cause by: {:?}", e))
}

#[inline]
pub(crate) fn to_custom_ser_error<S, E>(e: E) -> S::Error
where
    E: Debug,
    S: Serializer,
{
    S::Error::custom(format!("byte serialization failed, cause by: {:?}", e))
}

impl SerializeAs<roaring::RoaringBitmap> for SuiBitmap {
    fn serialize_as<S>(source: &roaring::RoaringBitmap, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = vec![];

        source
            .serialize_into(&mut bytes)
            .map_err(to_custom_ser_error::<S, _>)?;
        Vec::<u8>::serialize(&bytes, serializer)
    }
}

impl<'de> DeserializeAs<'de, roaring::RoaringBitmap> for SuiBitmap {
    fn deserialize_as<D>(deserializer: D) -> Result<roaring::RoaringBitmap, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::<u8>::deserialize(deserializer)?;
        deserialize_sui_bitmap(&bytes).map_err(to_custom_deser_error::<'de, D, _>)
    }
}

// RoaringBitmap::deserialize_from() or iter() do not check for duplicates.
// So this function is needed to sanitize the bitmap to ensure unique entries.
fn deserialize_sui_bitmap(bytes: &[u8]) -> std::io::Result<roaring::RoaringBitmap> {
    let orig_bitmap = roaring::RoaringBitmap::deserialize_from(bytes)?;
    // Ensure there is no duplicated entries in the bitmap.
    let mut seen = std::collections::BTreeSet::new();
    let mut new_bitmap = roaring::RoaringBitmap::new();
    for v in orig_bitmap.iter() {
        if seen.insert(v) {
            new_bitmap.insert(v);
        }
    }
    Ok(new_bitmap)
}
