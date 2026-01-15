// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cairo subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
use core::hash::HashStateTrait;
use core::poseidon::PoseidonTrait;
use ibc::types::{Timestamp, TimestampTrait};

#[derive(Debug, Drop, Serde, starknet::Store)]
pub struct ClientState {
    pub chain_id: felt252,
    pub latest_height: u64,
}

#[derive(Debug, Drop, Serde, starknet::Store)]
pub struct ConsensusState {
    pub contracts_trie_root: felt252,
    pub classes_trie_root: felt252,
    pub timestamp: Timestamp,
}

#[generate_trait]
pub impl ConsensusStateImpl of ConsensusStateTrait {
    fn ethabi_encode(self: @ConsensusState) -> ByteArray {
        let mut bz = Default::default();

        bz.append_felt252(*self.contracts_trie_root);
        bz.append_felt252(*self.classes_trie_root);
        bz.append_u256(self.timestamp.nanos().into());

        bz
    }
}

#[derive(Debug, Drop, Serde)]
pub struct Block {
    pub block_number: u64,
    pub parent_block_hash: felt252,
    pub contracts_trie_root: felt252,
    pub classes_trie_root: felt252,
    pub sequencer_address: felt252,
    // SECONDS
    pub block_timestamp: u64,
    pub transaction_count: u64,
    pub events_count: u64,
    pub state_diff_length: u64,
    pub state_diff_commitment: felt252,
    pub transactions_commitment: felt252,
    pub events_commitment: felt252,
    pub receipts_commitment: felt252,
    pub l1_gas_price: (u128, u128),
    pub l1_data_gas_price: (u128, u128),
    pub l2_gas_price: (u128, u128),
    pub l1_da_mode: L1DaMode,
    pub protocol_version: ByteArray,
}

#[generate_trait]
pub impl BlockImpl of BlockTrait {
    fn hash(self: @Block) -> felt252 {
        PoseidonTrait::new()
            .update('STARKNET_BLOCK_HASH1')
            .update((*self.block_number).into())
            .update(
                PoseidonTrait::new()
                    .update('STARKNET_STATE_V0')
                    .update(*self.contracts_trie_root)
                    .update(*self.classes_trie_root)
                    .finalize(),
            )
            .update(*self.sequencer_address)
            .update((*self.block_timestamp).into())
            .update(
                {
                    let mut inner_hash: ByteArray = Default::default();
                    inner_hash.append_u64(*self.transaction_count);
                    inner_hash.append_u64(*self.events_count);
                    inner_hash.append_u64(*self.state_diff_length);
                    match self.l1_da_mode {
                        L1DaMode::Calldata => inner_hash.append_u64(0),
                        // 64 bit binary with msb = 1
                        L1DaMode::Blob => inner_hash.append_u64(9223372036854775808),
                    }
                    let (_, val) = inner_hash.read_felt252(0);
                    val
                },
            )
            .update(*self.state_diff_commitment)
            .update(*self.transactions_commitment)
            .update(*self.events_commitment)
            .update(*self.receipts_commitment)
            .update(
                {
                    let (l1_gas_price_0, l1_gas_price_1) = *self.l1_gas_price;
                    let (l1_data_gas_price_0, l1_data_gas_price_1) = *self.l1_data_gas_price;
                    let (l2_gas_price_0, l2_gas_price_1) = *self.l2_gas_price;
                    PoseidonTrait::new()
                        .update('STARKNET_GAS_PRICES0')
                        .update(l1_gas_price_0.into())
                        .update(l1_gas_price_1.into())
                        .update(l1_data_gas_price_0.into())
                        .update(l1_data_gas_price_1.into())
                        .update(l2_gas_price_0.into())
                        .update(l2_gas_price_1.into())
                        .finalize()
                },
            )
            .update(
                {
                    let mut bz: ByteArray = Default::default();
                    // bz: 32
                    // protocol_version_len: 7
                    // 25 0's at the beginning
                    //
                    let mut protocol_version_len = 32 - self.protocol_version.len();
                    if protocol_version_len >= 16 {
                        bz.append_u128(0);
                        protocol_version_len -= 16;
                    }

                    if protocol_version_len >= 8 {
                        bz.append_u64(0);
                        protocol_version_len -= 8;
                    }

                    if protocol_version_len >= 4 {
                        bz.append_u32(0);
                        protocol_version_len -= 4;
                    }

                    if protocol_version_len >= 2 {
                        bz.append_u16(0);
                        protocol_version_len -= 2;
                    }

                    if protocol_version_len >= 1 {
                        bz.append_u8(0);
                    }

                    bz.append(self.protocol_version);

                    let (_, val) = bz.read_felt252(0);

                    val
                },
            )
            .update(0)
            .update(*self.parent_block_hash)
            .finalize()
    }
}


#[derive(Debug, Drop, Serde)]
pub enum L1DaMode {
    Calldata,
    Blob,
}

#[derive(Debug, Drop, Serde)]
pub struct Header {
    pub block: Block,
}

#[test]
fn l2_block_hash_3996475() {
    let b = Block {
        block_number: 3996475,
        parent_block_hash: 0x07488afa914e19281d6a859f1673d91f84b124576677bc90790954934bcf6a90,
        classes_trie_root: 0x052dedb4984ca5bde1fa31f46bdedd2462779d7a6db3039be87eb0c532d79470,
        contracts_trie_root: 0x02c6e3ddcdcf9bcd4b9e01c4b94408b6cf8b82ca9a1b40d808612483278b5afb,
        sequencer_address: 0x01176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8,
        block_timestamp: 1764693045,
        transaction_count: 8,
        events_count: 14 + 7 + 104 + 5 + 3 + 7 + 5 + 5,
        state_diff_length: 108,
        state_diff_commitment: 0x000d69e24d96773a920991dcd7f86fea0526acb3dae9bb3955caf840c71b54f6,
        transactions_commitment: 0x01df3ce5acd86d8c2d7f1155997a70a004ee0a0c36c67c9baafe87ace22f30d9,
        events_commitment: 0x030a53d5d62958b18f1094b66c4ad4c3bcee8dd2a36666fc5fc8b46ddaa5b37c,
        receipts_commitment: 0x0494e30696606f6208ac02b701f2350460c35b0be17cdf23e4017c79a6a69f2f,
        l1_gas_price: (0x6df5cf40, 0x27d11e1709d4),
        l1_data_gas_price: (0x1, 0x5cb2),
        l2_gas_price: (0x1edd2, 0xb2d05e00),
        l1_da_mode: L1DaMode::Blob,
        protocol_version: "0.14.0",
    };

    assert_eq!(b.hash(), 0x366cae7718ded291ef9c5f4c2aba8c3c27baa0e563fd64ba72fe51c2abc4675)
}

#[test]
fn consensus_state_encoding() {
    let mut encoded: ByteArray = Default::default();
    encoded.append_u256(0x02c6e3ddcdcf9bcd4b9e01c4b94408b6cf8b82ca9a1b40d808612483278b5afb);
    encoded.append_u256(0x052dedb4984ca5bde1fa31f46bdedd2462779d7a6db3039be87eb0c532d79470);
    encoded.append_u256(TimestampTrait::from_secs(1764693045).nanos().into());

    assert_eq!(
        encoded,
        ConsensusState {
            contracts_trie_root: 0x02c6e3ddcdcf9bcd4b9e01c4b94408b6cf8b82ca9a1b40d808612483278b5afb,
            classes_trie_root: 0x052dedb4984ca5bde1fa31f46bdedd2462779d7a6db3039be87eb0c532d79470,
            timestamp: TimestampTrait::from_secs(1764693045),
        }
            .ethabi_encode(),
    )
}
