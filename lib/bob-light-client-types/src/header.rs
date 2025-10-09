use ethereum_light_client_types::{AccountProof, StorageProof};
use rlp::Encodable;
use unionlabs::{
    ethereum::keccak256,
    primitives::{Bytes, H64, H72, H160, H256, H2048, U256},
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
    pub requests_hash: H256,
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
            parent_hash: hex!("327686d326438b9f95b8300c1ceed12050a3d685fcfbe895f23f8a812e57ee15").into(),
            sha3_uncles: hex!("1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").into(),
            miner: hex!("4200000000000000000000000000000000000011").into(),
            state_root: hex!("096c1251fad148c6e8d39934dfb1b1e677232ab7b7a7ab294195a99826ec2e2b").into(),
            transactions_root: hex!(
                "333c9ba379b9c46686ca661ebb6a033f9d5a0cdc11f00b175d2e1706c34f9614"
            ).into(),
            receipts_root: hex!(
                "4d8e11add88fec0151369ca910de47451e314d30d7c7fd81d1906a34ca10de0b"
            ).into(),
            logs_bloom: Box::new(hex!("57bbb78267e7bf5d0cfe97ffb7ee6bd5fefed75fb6efbf3dafd77bc8febbbf7cedeed71e5cdd7df43763f8fdcf3eb77bf47f8b749eedfb8cffffaf0ff7a7fbe0b777dd9bb27ffc2d6d7ef2eebd5768fee7efacafcbffeff7ec7ef7dcbbfa3674ffbecfb61f7fd7ef7b3feaabef1f9e2b595f7f6f4efd979cde6bb1df157f97eff7cf2e2bffe57fcecf6bdf3af80efcd3b773ffefff7bd799fdeeb87fffd6d5fb1fef195ff7f8f3a9d7fff7425ef5a5fffdf179ba65be1dddb3efbb5b8d9efbb63a548ffed9fbf577cbc92fdd276b7fdf750fc7afffda3bf2bfbe07877b9fffb6c6debedbedb5f77b7ffdbfefbadf77a5eda2bfe62ffe5effee58dbc3575f2bdf").into()),
            difficulty: 0_u64.into(),
            number: 32376940_u64.into(),
            gas_limit: 0x8f0d180,
            gas_used: 0x254cdb6,
            timestamp: 0x68666dbb,
            extra_data: hex!("000000003200000003").into(),
            mix_hash: hex!("40bfcd42c3cb3b7966a467ce8cdc2638cbd6b03558448a82a47b42ad5504ef72").into(),
            nonce: hex!("0000000000000000").into(),
            base_fee_per_gas: 0x326e8f_u64.into(),
            withdrawals_root: hex!(
                "6ee1a180fdbccc2e70984ac91116d64c58feb817a417a81500b9b0cbd69a9373"
            ).into(),
            blob_gas_used: 0,
            excess_blob_gas: 0,
            parent_beacon_block_root: hex!(
                "4fccae25b4204cb426da9b8b2961be949be574f964e000c6b64627e7c98be4c3"
            ).into(),
            requests_hash: hex!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").into(),
        };

        let hash: H256 =
            hex!("f5b06eb8b0bacf8b030dbd596964b6bf346f7602f906f57db958e12c726367f1").into();

        assert_eq!(hash, header.hash());
    }
}
