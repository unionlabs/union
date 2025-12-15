pub use ethereum_light_client_types::AccountProof;
use starknet_crypto::{pedersen_hash, poseidon_hash_many};
use starknet_types::{Felt, MerkleNode};
use unionlabs_primitives::Bytes;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub l1_height: u64,
    pub l1_contract_account_proof: AccountProof,
    pub l1_block_hash_proof: Vec<Bytes>,
    pub l2_block: L2Block,
    pub l2_ibc_contract_proof: ContractProof,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct L2Block {
    pub block_number: u64,
    pub parent_block_hash: Felt,
    pub contracts_trie_root: Felt,
    pub classes_trie_root: Felt,
    pub sequencer_address: Felt,
    // SECONDS
    pub block_timestamp: u64,
    pub transaction_count: u32,
    pub events_count: u32,
    pub state_diff_length: u32,
    pub state_diff_commitment: Felt,
    pub transactions_commitment: Felt,
    pub events_commitment: Felt,
    pub receipts_commitment: Felt,
    pub l1_gas_price: (u128, u128),
    pub l1_data_gas_price: (u128, u128),
    pub l2_gas_price: (u128, u128),
    pub l1_da_mode: L1DaMode,
    pub protocol_version: String,
}

impl L2Block {
    /// <https://docs.starknet.io/learn/protocol/blocks#block-hash>
    /// <https://github.com/starkware-libs/sequencer/blob/079ed26ce95b3b10de40c9916ffa332aaecd9f06/crates/starknet_api/src/block_hash/block_hash_calculator.rs#L134>
    // TODO: Handle different versions
    pub fn hash(&self) -> Felt {
        poseidon_hash_many(
            &[
                // hex(b"STARKNET_BLOCK_HASH1")
                Felt::from_hex("0x535441524b4e45545f424c4f434b5f4841534831").unwrap(),
                self.block_number.into(),
                // global_root
                poseidon_hash_many(
                    &[
                        // hex(b"STARKNET_STATE_V0")
                        Felt::from_hex("0x535441524b4e45545f53544154455f5630").unwrap(),
                        self.contracts_trie_root,
                        self.classes_trie_root,
                    ]
                    .map(Into::into),
                )
                .into(),
                self.sequencer_address,
                self.block_timestamp.into(),
                // https://github.com/starkware-libs/sequencer/blob/079ed26ce95b3b10de40c9916ffa332aaecd9f06/crates/starknet_api/src/block_hash/block_hash_calculator.rs#L230
                Felt::from_be_bytes({
                    let mut bz = [0; 32];

                    bz[0..8].copy_from_slice(&(self.transaction_count as u64).to_be_bytes());
                    bz[8..16].copy_from_slice(&(self.events_count as u64).to_be_bytes());
                    bz[16..24].copy_from_slice(&(self.state_diff_length as u64).to_be_bytes());
                    bz[24] = match self.l1_da_mode {
                        L1DaMode::Calldata => 0b0000_0000,
                        L1DaMode::Blob => 0b1000_0000,
                    };

                    bz
                }),
                self.state_diff_commitment,
                self.transactions_commitment,
                self.events_commitment,
                self.receipts_commitment,
                poseidon_hash_many(
                    &[
                        // hex(b"STARKNET_GAS_PRICES0")
                        Felt::from_hex("0x535441524b4e45545f4741535f50524943455330").unwrap(),
                        self.l1_gas_price.0.into(),
                        self.l1_gas_price.1.into(),
                        self.l1_data_gas_price.0.into(),
                        self.l1_data_gas_price.1.into(),
                        self.l2_gas_price.0.into(),
                        self.l2_gas_price.1.into(),
                    ]
                    .map(Into::into),
                )
                .into(),
                Felt::from_be_bytes({
                    let mut bz = [0; 32];

                    bz[32 - self.protocol_version.len()..]
                        .copy_from_slice(self.protocol_version.as_bytes());

                    bz
                }),
                Felt::ZERO,
                self.parent_block_hash,
            ]
            .map(Into::into),
        )
        .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum L1DaMode {
    Blob,
    Calldata,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ContractProof {
    /// The nodes in the union of the paths from the contracts tree root to the requested leaves
    pub nodes: Vec<MerkleNode>,
    pub contract_leaf_data: ContractLeafData,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ContractLeafData {
    pub nonce: Felt,
    pub class_hash: Felt,
    pub storage_root: Felt,
}

impl ContractLeafData {
    pub fn hash(&self) -> Felt {
        pedersen_hash(
            &pedersen_hash(
                &pedersen_hash(&self.class_hash.into(), &self.storage_root.into()),
                &self.nonce.into(),
            ),
            &Felt::ZERO.into(),
        )
        .into()
    }
}

#[test]
fn l2_block_hash_3996475() {
    // https://feeder.alpha-mainnet.starknet.io/feeder_gateway/get_block?blockNumber=3996475
    let block = L2Block {
        block_number: 3996475,
        parent_block_hash: Felt::from_hex(
            "07488afa914e19281d6a859f1673d91f84b124576677bc90790954934bcf6a90",
        )
        .unwrap(),
        classes_trie_root: Felt::from_hex(
            "052dedb4984ca5bde1fa31f46bdedd2462779d7a6db3039be87eb0c532d79470",
        )
        .unwrap(),
        contracts_trie_root: Felt::from_hex(
            "02c6e3ddcdcf9bcd4b9e01c4b94408b6cf8b82ca9a1b40d808612483278b5afb",
        )
        .unwrap(),
        sequencer_address: Felt::from_hex(
            "01176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8",
        )
        .unwrap(),
        block_timestamp: 1764693045,
        transaction_count: 8,
        events_count: 14 + 7 + 104 + 5 + 3 + 7 + 5 + 5,
        state_diff_length: 108,
        state_diff_commitment: Felt::from_hex(
            "000d69e24d96773a920991dcd7f86fea0526acb3dae9bb3955caf840c71b54f6",
        )
        .unwrap(),
        transactions_commitment: Felt::from_hex(
            "01df3ce5acd86d8c2d7f1155997a70a004ee0a0c36c67c9baafe87ace22f30d9",
        )
        .unwrap(),
        events_commitment: Felt::from_hex(
            "030a53d5d62958b18f1094b66c4ad4c3bcee8dd2a36666fc5fc8b46ddaa5b37c",
        )
        .unwrap(),
        receipts_commitment: Felt::from_hex(
            "0494e30696606f6208ac02b701f2350460c35b0be17cdf23e4017c79a6a69f2f",
        )
        .unwrap(),
        l1_gas_price: (0x6df5cf40, 0x27d11e1709d4),
        l1_data_gas_price: (0x1, 0x5cb2),
        l2_gas_price: (0x1edd2, 0xb2d05e00),
        l1_da_mode: L1DaMode::Blob,
        protocol_version: "0.14.0".to_owned(),
    };

    dbg!(&block);

    assert_eq!(
        block.hash(),
        Felt::from_hex("0x366cae7718ded291ef9c5f4c2aba8c3c27baa0e563fd64ba72fe51c2abc4675")
            .unwrap()
    );
}
