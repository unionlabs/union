use core::fmt;

use consensus_primitives::Timestamp;
use rlp::{RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper};
use sha3::Digest as _;
use unionlabs_primitives::{ByteArrayExt, Bytes, H64, H160, H256, H384, H768, H2048, U256};

#[derive(Debug, Clone, PartialEq, RlpDecodable, RlpEncodable)]
pub struct VoteAttestation {
    // The bitset marks the voted validators.
    pub vote_address_set: ValidatorsBitSet,
    // The aggregated BLS signature of the voted validators' signatures.
    pub agg_signature: H768,
    // The vote data for fast finality.
    pub data: VoteData,
    // Reserved for future usage.
    pub extra: Bytes,
}

#[derive(Clone, PartialEq, RlpDecodableWrapper, RlpEncodableWrapper)]
pub struct ValidatorsBitSet(u64);

impl ValidatorsBitSet {
    pub const fn new(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn is_set(&self, idx: usize) -> bool {
        self.0 & (1 << idx) != 0
    }

    pub const fn count(&self) -> u32 {
        self.0.count_ones()
    }
}

impl fmt::Debug for ValidatorsBitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ValidatorsBitSet")
            .field(&format_args!("{:#064b}", self.0))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, RlpDecodable, RlpEncodable)]
pub struct VoteData {
    // The source block number should be the latest justified block number.
    pub source_number: u64,
    // The block hash of the source block.
    pub source_hash: H256,
    // The target block number which validator wants to vote for.
    pub target_number: u64,
    // The block hash of the target block.
    pub target_hash: H256,
}

impl VoteData {
    pub fn hash(&self) -> H256 {
        sha3::Keccak256::new()
            .chain_update(rlp::encode(self))
            .finalize()
            .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Valset(Vec<(H160, H384)>);

impl Valset {
    pub fn new(inner: Vec<(H160, H384)>) -> Self {
        Self(inner)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (H160, H384)> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, rlp::RlpEncodable)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ParliaHeader {
    pub parent_hash: H256,
    pub sha3_uncles: H256,
    pub miner: H160,
    pub state_root: H256,
    pub transactions_root: H256,
    pub receipts_root: H256,
    // Box since 256 bytes is quite large
    pub logs_bloom: Box<H2048>,
    #[cfg_attr(
        feature = "serde",
        serde(with = "unionlabs_primitives::uint::u256_big_endian_hex")
    )]
    pub difficulty: U256,
    #[cfg_attr(
        feature = "serde",
        serde(with = "unionlabs_primitives::uint::u256_big_endian_hex")
    )]
    pub number: U256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub gas_limit: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub gas_used: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub timestamp: u64,
    pub extra_data: Bytes,
    pub mix_hash: H256,
    pub nonce: H64,
    #[cfg_attr(
        feature = "serde",
        serde(with = "unionlabs_primitives::uint::u256_big_endian_hex")
    )]
    pub base_fee_per_gas: U256,
    pub withdrawals_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub blob_gas_used: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub excess_blob_gas: u64,
    pub parent_beacon_block_root: H256,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Nullable::is_none")
    )]
    pub requests_hash: Nullable<H256>,
}

impl ParliaHeader {
    pub fn full_timestamp(&self) -> Timestamp {
        let millis = u16::from_be_bytes(self.mix_hash.get().array_slice::<30, 2>());
        Timestamp::from_millis((self.timestamp * 1_000) + (millis as u64))
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Nullable<T>(Option<T>);

impl<T> Nullable<T> {
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl<T> From<Option<T>> for Nullable<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

impl<T: rlp::Encodable> rlp::Encodable for Nullable<T> {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        if let Some(ref value) = self.0 {
            s.append(value);
        }
    }
}

impl ParliaHeader {
    pub fn hash(&self) -> H256 {
        sha3::Keccak256::new()
            .chain_update(rlp::encode(self))
            .finalize()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn hash() {
        let header = ParliaHeader {
            parent_hash: hex!("2e0fa901f264102bd20c86822222f0e3be0e72517ddf8ecae5bb09717c542f33").into(),
            sha3_uncles: hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").into(),
            miner: hex!("4200000000000000000000000000000000000011").into(),
            state_root: hex!("1965c4276f2ee3e5e7f4190c1302835c630afeb7b1078f595ad80206d0901fc2").into(),
            transactions_root: hex!(
                "75418567809ce6cb380eebb1895034e31751b2eb691c2f4b170de50d25e8c061"
            ).into(),
            receipts_root: hex!(
                "0d98745240a828bea4726880dca08008c87f95f6a55f7524861bb23917b0b6e7"
            ).into(),
            logs_bloom: Box::new(hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").into()),
            difficulty: 0_u64.into(),
            number: 16649144_u64.into(),
            gas_limit: 30000000,
            gas_used: 64887,
            timestamp: 1746160275,
            extra_data: hex!("00000000fa00000006").into(),
            mix_hash: hex!("a20ab72103cd88aefd65cb31a0c470a6a88e53b4d4da00b27503deb4580e128c").into(),
            nonce: hex!("0000000000000000").into(),
            base_fee_per_gas: 252_u64.into(),
            withdrawals_root: hex!(
                "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"
            ).into(),
            blob_gas_used: 0,
            excess_blob_gas: 0,
            parent_beacon_block_root: hex!(
                "9467bee415d6385906410d33fc263b32ef2d4880f4a6c9559db669acfd325213"
            ).into(),
            requests_hash: Nullable(None),
        };

        let hash: H256 =
            hex!("8975fdc9d79a1ad20b7cca844ef06f25b4a61e76c8d0cef32121aff90dbdef53").into();

        assert_eq!(hash, header.hash());
    }
}
