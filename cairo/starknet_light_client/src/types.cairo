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

use core::hash::HashStateTrait;
use core::poseidon::{Poseidon, PoseidonTrait};
use ibc::types::Timestamp;

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

#[derive(Debug, Drop, Serde)]
pub struct Block {
    pub block_number: felt252,
    pub parent_block_hash: felt252,
    pub contracts_trie_root: felt252,
    pub classes_trie_root: felt252,
    pub sequencer_address: felt252,
    // SECONDS
    pub block_timestamp: felt252,
    pub transaction_count: felt252,
    pub events_count: felt252,
    pub state_diff_length: felt252,
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
    fn hash(self: @Block) -> Result<(), ()> {
        let _ = PoseidonTrait::new()
            .update(0x535441524b4e45545f424c4f434b5f4841534831)
            .update(*self.block_number)
            .update(
                PoseidonTrait::new()
                    .update(0x535441524b4e45545f53544154455f5630)
                    .update(*self.contracts_trie_root)
                    .update(*self.classes_trie_root)
                    .finalize(),
            )
            .update(*self.sequencer_address)
            .update(*self.block_timestamp)
            .finalize();
        Ok(())
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
