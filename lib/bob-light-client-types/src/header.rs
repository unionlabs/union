use ethereum_light_client_types::{AccountProof, StorageProof};
use rlp::Encodable;
use unionlabs::{
    errors::InvalidLength,
    ethereum::keccak256,
    primitives::{Bytes, H64, H160, H256, H2048, U256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub l1_height: u64,
    pub dispute_game_factory_account_proof: AccountProof,
    pub game_index: U256,
    pub game_proof: StorageProof,
    pub game_account_proof: AccountProof,
    pub game_account_code: Bytes,
    pub l2_ibc_account_proof: AccountProof,
    pub l2_header: L2Header,
    pub output_root_proof: OutputRootProof,
}

// https://github.com/ethereum-optimism/optimism/blob/99a53381019d3571359d989671ccf70f8d69dfd9/packages/contracts-bedrock/src/libraries/Types.sol#L25
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct OutputRootProof {
    /// The version of the output, always zero in bob?
    pub version: H256,
    /// The state root of the L2 block.
    pub state_root: H256,
    /// The storage root of the [L2ToL1MessagePasser contract](https://docs.gobob.xyz/learn/reference/contracts/#bob-mainnet-l2).
    pub message_passer_storage_root: H256,
    /// The block hash pointed by the output.
    pub latest_block_hash: H256,
}

// Bedrock v1.7.2 rely on go-ethereum v1.13.8
// https://github.com/ethereum/go-ethereum/blob/b20b4a71598481443d60b261d3e5dcb37f8a0d82/core/types/block.go#L65
#[derive(Debug, Clone, PartialEq, rlp::RlpEncodable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct L2Header {
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
        serde(with = "unionlabs::primitives::uint::u256_big_endian_hex")
    )]
    pub difficulty: U256,
    #[cfg_attr(
        feature = "serde",
        serde(with = "unionlabs::primitives::uint::u256_big_endian_hex")
    )]
    pub number: U256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub gas_limit: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub gas_used: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub timestamp: u64,
    pub extra_data: BytesMax32,
    pub mix_hash: H256,
    pub nonce: H64,
    #[cfg_attr(
        feature = "serde",
        serde(with = "unionlabs::primitives::uint::u256_big_endian_hex")
    )]
    pub base_fee_per_gas: U256,
    pub withdrawals_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub blob_gas_used: u64,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::u64_hex"))]
    pub excess_blob_gas: u64,
    pub parent_beacon_block_root: H256,
    pub requests_hash: H256,
}

impl L2Header {
    #[must_use]
    pub fn hash(&self) -> H256 {
        keccak256(self.rlp_bytes())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "Bytes", into = "Bytes")
)]
pub struct BytesMax32(Bytes);

impl rlp::Encodable for BytesMax32 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.encoder().encode_value(&self.0);
    }
}

#[cfg(feature = "bincode")]
impl bincode::Encode for BytesMax32 {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.0, encoder)
    }
}

#[cfg(feature = "bincode")]
impl<Context> bincode::Decode<Context> for BytesMax32 {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        <Bytes as bincode::Decode<Context>>::decode(decoder)?
            .try_into()
            .map_err(|_| bincode::error::DecodeError::Other("invalid BytesMax32 length"))
    }
}
#[cfg(feature = "bincode")]
bincode::impl_borrow_decode!(BytesMax32);

impl TryFrom<Bytes> for BytesMax32 {
    type Error = InvalidLength;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        if value.len() > 32 {
            Err(InvalidLength {
                expected: unionlabs::errors::ExpectedLength::LessThan(33),
                found: value.len(),
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl From<BytesMax32> for Bytes {
    fn from(value: BytesMax32) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn hash() {
        let header = L2Header {
            parent_hash: hex!("53636825d8a9e633b4fcd73fbd3980317519695427e3514689b4d783c15b265e").into(),
            sha3_uncles: hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").into(),
            miner: hex!("4200000000000000000000000000000000000011").into(),
            state_root: hex!("8f079c04ffe7b0782ea9e3ec105fec4417a6876b2d1c73233fadcce2d4d3c833").into(),
            transactions_root: hex!(
                "fcf5f195056c9e94d2e017b1e850b2c996481dcef04316753262b3b4b56e8e6b"
            ).into(),
            receipts_root: hex!(
                "f2343e43f271a1ad1382f680bd2d263103ca3ecb56b32ce816d6d4a46a3ccf80"
            ).into(),
            logs_bloom: Box::new(hex!("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").into()),
            difficulty: 0_u64.into(),
            number: 0x1ec241d_u64.into(),
            gas_limit: 0x1c9c380,
            gas_used: 0xb4ea,
            timestamp: 0x69f07b5d,
            extra_data: <Bytes>::from(hex!("01000000fa000000060000000000000000")).try_into().unwrap(),
            mix_hash: hex!("5153f2e659ea5f7841427ec4fe15415eb68bf14fea44f7fb53cabb25c9597fb5").into(),
            nonce: hex!("0000000000000000").into(),
            base_fee_per_gas: 0xfc_u64.into(),
            withdrawals_root: hex!(
                "cca4e8849ead727dc9f5e8223eeb80e3e5ad03cccddf1f82b5ee9664905ec215"
            ).into(),
            blob_gas_used: 0,
            excess_blob_gas: 0,
            parent_beacon_block_root: hex!(
                "f88b3eb7a821ca63323b5964588d24e973c8399bc7125de7fd963dc7e1726ebb"
            ).into(),
            requests_hash: hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").into(),
        };

        let hash: H256 =
            hex!("4b4db973e20f92507f07cab622c2e80821e72a7a185c42ea5658be0db5a310c3").into();

        assert_eq!(hash, header.hash());
    }
}
