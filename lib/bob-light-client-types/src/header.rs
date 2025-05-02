use ethereum_light_client_types::{AccountProof, StorageProof};
use rlp::Encodable;
use unionlabs::{
    ethereum::keccak256,
    primitives::{H160, H2048, H256, H64, H72, U256},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub l1_height: u64,
    pub l2_oracle_account_proof: AccountProof,
    pub l2_oracle_l2_outputs_slot_proof: StorageProof,
    pub l2_ibc_account_proof: AccountProof,
    pub l2_header: L2Header,
    pub output_index: u32,
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
    pub extra_data: H72,
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
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Nullable::is_none")
    )]
    pub requests_hash: Nullable<H256>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Default, Debug, Clone, PartialEq)]
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

impl<T: Encodable> Encodable for Nullable<T> {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        if let Some(ref value) = self.0 {
            s.append(value);
        }
    }
}

impl L2Header {
    #[must_use]
    pub fn hash(&self) -> H256 {
        keccak256(self.rlp_bytes())
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn hash() {
        let header = L2Header {
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
