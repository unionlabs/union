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

use ibc::types::Timestamp;

#[derive(Debug, Drop, Serde, starknet::Store)]
pub struct ClientState {
    // TODO(aeryz): does this needs to be bytes31 or felt252 works too? (it probably works)
    pub chain_id: felt252,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
    /// This field only ever has one of two values:
    ///
    /// - 0: client is not frozen
    /// - 1: client is frozen
    ///
    /// Both the field name and type match the ICS07 Tendermint implementation.
    ///
    /// Note that the above bounds are not enforced at the type level, which also matches the
    /// Tendermint specification.
    pub frozen_height: u64,
    pub latest_height: u64,
    /// For clients that connect to the cosmwasm implementation of ibc-union, the contract address
    /// of the IBC host is required in order to verify storage proofs. For clients connecting to IBC
    /// classic, this field is not required and can be ignored during client creation and migration.
    pub contract_address: u256,
}

#[derive(Debug, Drop, Serde, starknet::Store)]
pub struct ConsensusState {
    /// Block timestamp (in nanoseconds)
    pub timestamp: Timestamp,
    /// App hash from the block header
    pub app_hash: u256,
    /// Next validators from the block header
    pub next_validators_hash: u256,
}

#[derive(Debug, Drop, Serde)]
pub struct Header {
    pub signed_header: SignedHeader,
    pub trusted_height: u64,
    pub zkp: ZeroKnowledgeProof,
}


#[derive(Debug, Drop, Serde)]
pub struct SignedHeader {
    pub height: u64,
    pub secs: u64,
    pub nanos: u64,
    pub validators_hash: u256,
    pub next_validators_hash: u256,
    pub app_hash: u256,
}

#[derive(Debug, Drop, Serde)]
pub struct ZeroKnowledgeProof {}
