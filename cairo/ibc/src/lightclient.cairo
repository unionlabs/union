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

use starknet::ContractAddress;
use crate::types::{ClientId, Timestamp};

#[derive(Drop, Serde)]
pub struct ConsensusStateUpdate {
    pub consensus_state_commitment: u256,
    pub height: u64,
}

#[starknet::interface]
pub trait ILightClient<TContractState> {
    /// Create a light client with the `client_id`.
    ///
    /// #### Params
    ///
    /// - `caller`: TODO(aeryz): why not make this get_caller_address()?
    /// - `client_id`: The unique identifier of the client that is going to be created.
    /// - `client_state_bytes`: The client-defined state of the client. Note that there's only a
    /// single client state which usually contains the configuration of the client, info and the
    /// latest known state of the counterparty chain.
    /// - `consensus_state_bytes`: The client-defined state of the counterparty chain per height.
    /// This usually contains the state root at a certain height.
    /// - `relayer`: A user-defined address. CANNOT BE USED for authentication as it's set by the
    /// caller. It does not have to be the signer of this transaction. Use `get_caller_address` for
    /// that.
    ///
    /// #### Return
    ///
    /// Return the client and consensus state commitments. The `consensus_state_commitments` will be
    /// written under the `height`. The `height` is generally parsed from the `client_state_bytes`
    /// form. Check the `cometbls_light_client` implementation to see an example.
    ///
    /// The second return argument (`ByteArray`) is the counterparty chain ID which is again
    /// generally be parsed from the `client_state_bytes`.
    fn create_client(
        ref self: TContractState,
        caller: ContractAddress,
        client_id: ClientId,
        client_state_bytes: Array<felt252>,
        consensus_state_bytes: Array<felt252>,
        relayer: ContractAddress,
    ) -> (ConsensusStateUpdate, ByteArray);

    /// Update the client `client_id` to a new state.
    ///
    /// Update the client and save whatever information is needed for the verification to be
    /// successful. This can be a state root, a header, it depends on the counterparty chain
    /// implementation. Clients are free to specify their update data however they want and it will
    /// be passed as is in the `client_message`.
    //
    /// `relayer` CANNOT BE USED for authentication as it's set by the caller. It does not have to
    /// be the signer of this transaction. Use `get_caller_address` for that.
    ///
    /// Return the client and consensus state commitments. The `consensus_state_commitments` will be
    /// written under the `height`. The meaning of the `height` changes based on the counterparty
    /// chain implementation. Check the `cometbls_light_client` implementation to see an example.
    //
    fn update_client(
        ref self: TContractState,
        caller: ContractAddress,
        client_id: ClientId,
        client_message: Array<felt252>,
        relayer: ContractAddress,
    ) -> ConsensusStateUpdate;

    /// Verify the existence of `key/value` pair on the counterparty chain using `client_id`.
    ///
    /// Verify that the client-defined `proof` - generated at `height` - confirms that the
    /// `key/value` pair exists on the counterparty chain. The existence proof is totally up-to the
    /// counterparty chain. Hence the `proof` is defined by the clients and be passed in as is by
    /// the protocol.
    ///
    /// The clients need the state at the `height` to be able to verify the `proof`. Hence, the flow
    /// is usually an `update_client` call updates the client to the state at the `height` and then
    /// arbitrary number of verifications can be done by using that state at the `height`.
    ///
    /// Return `true` if the proof verification passes.
    fn verify_membership(
        self: @TContractState,
        client_id: ClientId,
        height: u64,
        proof: Array<felt252>,
        key: ByteArray,
        value: ByteArray,
    ) -> bool;

    fn verify_non_membership(
        self: @TContractState, client_id: ClientId, height: u64, proof: ByteArray, key: ByteArray,
    ) -> bool;

    fn get_timestamp_at_height(self: @TContractState, height: u64) -> Timestamp;

    fn get_latest_height(self: @TContractState, client_id: ClientId) -> u64;
}
