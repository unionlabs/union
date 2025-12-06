use ethereum_light_client_types::StorageProof;
use starknet_types_core::{
    felt::Felt,
    hash::{Poseidon, StarkHash as _},
};
use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    pub l1_height: u64,
    pub l1_block_hash_proof: StorageProof,
    pub l2_block: L2Block,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct L2Block {
    block_number: u64,
    parent_block_hash: H256,
    global_state_root: H256,
    sequencer_address: H256,
    // SECONDS
    block_timestamp: u64,
    transaction_count: u32,
    events_count: u32,
    state_diff_length: u32,
    state_diff_commitment: H256,
    transactions_commitment: H256,
    events_commitment: H256,
    receipts_commitment: H256,
    l1_gas_price: (u128, u128),
    l1_data_gas_price: (u128, u128),
    l2_gas_price: (u128, u128),
    l1_da_mode: L1DaMode,
    protocol_version: String,
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

impl L2Block {
    /// <https://docs.starknet.io/learn/protocol/blocks#block-hash>
    /// <https://github.com/starkware-libs/sequencer/blob/079ed26ce95b3b10de40c9916ffa332aaecd9f06/crates/starknet_api/src/block_hash/block_hash_calculator.rs#L134>
    pub fn hash(&self) -> H256 {
        Poseidon::hash_array(&[
            // hex(b"STARKNET_BLOCK_HASH1")
            const { Felt::from_hex_unwrap("0x535441524b4e45545f424c4f434b5f4841534831") },
            self.block_number.into(),
            Felt::from_bytes_be(self.global_state_root.get()),
            Felt::from_bytes_be(self.sequencer_address.get()),
            self.block_timestamp.into(),
            // https://github.com/starkware-libs/sequencer/blob/079ed26ce95b3b10de40c9916ffa332aaecd9f06/crates/starknet_api/src/block_hash/block_hash_calculator.rs#L230
            Felt::from_bytes_be_slice(
                [
                    (self.transaction_count as u64),
                    (self.events_count as u64),
                    (self.state_diff_length as u64),
                    match self.l1_da_mode {
                        // 0b0000_0000 ++ 7 bytes 0 padding
                        L1DaMode::Calldata => 0_u64,
                        // 0b1000_0000 ++ 7 bytes 0 padding
                        L1DaMode::Blob => 1 << 63,
                    },
                ]
                .map(u64::to_be_bytes)
                .as_flattened(),
            ),
            Felt::from_bytes_be(self.state_diff_commitment.get()),
            Felt::from_bytes_be(self.transactions_commitment.get()),
            Felt::from_bytes_be(self.events_commitment.get()),
            Felt::from_bytes_be(self.receipts_commitment.get()),
            Poseidon::hash_array(&[
                // hex(b"STARKNET_GAS_PRICES0")
                const { Felt::from_hex_unwrap("0x535441524b4e45545f4741535f50524943455330") },
                self.l1_gas_price.0.into(),
                self.l1_gas_price.1.into(),
                self.l1_data_gas_price.0.into(),
                self.l1_data_gas_price.1.into(),
                self.l2_gas_price.0.into(),
                self.l2_gas_price.1.into(),
            ]),
            Felt::from_bytes_be_slice(self.protocol_version.as_bytes()),
            Felt::ZERO,
            Felt::from_bytes_be(self.parent_block_hash.get()),
        ])
        .to_bytes_be()
        .into()
    }
}

#[test]
fn l2_block_hash() {
    use hex_literal::hex;

    // https://feeder.alpha-mainnet.starknet.io/feeder_gateway/get_block?blockNumber=3996475
    let block = L2Block {
        block_number: 3996475,
        parent_block_hash: hex!("07488afa914e19281d6a859f1673d91f84b124576677bc90790954934bcf6a90")
            .into(),
        global_state_root: hex!("000b977d63eeb59fda732ff60c6b956a91bd1c30784b2a25829f3a5fd882b0f8")
            .into(),
        sequencer_address: hex!("01176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8")
            .into(),
        block_timestamp: 1764693045,
        transaction_count: 8,
        events_count: 14 + 7 + 104 + 5 + 3 + 7 + 5 + 5,
        state_diff_length: 108,
        state_diff_commitment: hex!(
            "000d69e24d96773a920991dcd7f86fea0526acb3dae9bb3955caf840c71b54f6"
        )
        .into(),
        transactions_commitment: hex!(
            "01df3ce5acd86d8c2d7f1155997a70a004ee0a0c36c67c9baafe87ace22f30d9"
        )
        .into(),
        events_commitment: hex!("030a53d5d62958b18f1094b66c4ad4c3bcee8dd2a36666fc5fc8b46ddaa5b37c")
            .into(),
        receipts_commitment: hex!(
            "0494e30696606f6208ac02b701f2350460c35b0be17cdf23e4017c79a6a69f2f"
        )
        .into(),
        l1_gas_price: (0x6df5cf40, 0x27d11e1709d4),
        l1_data_gas_price: (0x1, 0x5cb2),
        l2_gas_price: (0x1edd2, 0xb2d05e00),
        l1_da_mode: L1DaMode::Blob,
        protocol_version: "0.14.0".to_owned(),
    };

    dbg!(&block);

    assert_eq!(
        block.hash(),
        <H256>::new(hex!(
            "0366cae7718ded291ef9c5f4c2aba8c3c27baa0e563fd64ba72fe51c2abc4675"
        ))
    );
}
