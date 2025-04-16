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
}

impl L2Header {
    #[must_use]
    pub fn hash(&self) -> H256 {
        keccak256(self.rlp_bytes())
    }
}
