// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

module ibc::ibc_channel {
    use std::string::String;

    use sui::table::Table;
    use sui::hash::keccak256;

    use ibc::commitment;
    use ibc::channel::{Self, Channel};
    use ibc::channel_state;
    use ibc::connection::Connection;
    use ibc::connection_state;
    use ibc::events;
    use ibc::light_client::LightClientManager;
    use ibc::state;

    const EBase: u64 = 10300;
    const EInvalidConnectionState: u64 = EBase + 1;
    const EInvalidChannelState: u64 = EBase + 2;

    public(package) fun channel_open_init(
        ibc_uid: &mut UID,
        connections: &Table<u32, Connection>,
        channels: &mut Table<u32, Channel>,
        port_id: address,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    ) {
        // Ensure the connection exists and is in the OPEN state
        let connection = connections.borrow(connection_id);
        assert!(
            connection.state() == connection_state::new_open(),
            EInvalidConnectionState
        );

        // Generate a new channel ID
        let channel_id = (channels.length() as u32) + 1;

        // Create a new channel and set its properties
        let channel =
            channel::new(
                channel_state::new_init(),
                connection_id,
                0,
                counterparty_port_id,
                version
            );

        state::add_channel_to_port(ibc_uid, channel_id, port_id);

        commit_channel(ibc_uid, channel_id, channel);

        channels.add(channel_id, channel);

        events::emit_channel_open_init(
            port_id,
            channel_id,
            counterparty_port_id,
            connection_id,
            version
        );
    }

    public fun channel_open_try(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
        channels: &mut Table<u32, Channel>,
        port_id: address,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        // Ensure the connection exists and is in the OPEN state
        let connection = connections.borrow(connection_id);
        assert!(
            connection.state() == connection_state::new_open(),
            EInvalidConnectionState
        );

        // Construct the expected channel state to verify against the proof
        let expected_channel =
            channel::new(
                channel_state::new_init(),
                connection.counterparty_connection_id(),
                0,
                port_id.to_bytes(),
                counterparty_version
            );

        // let light_client = ibc_store.clients.borrow(connection.client_id());
        // Verify the channel state using the provided proof and expected state
        let res =
            verify_channel_state(
                client_mgr,
                connection.client_id(),
                proof_height,
                proof_init,
                counterparty_channel_id,
                expected_channel
            );
        assert!(res == 0, res);

        // Generate a new channel ID
        let channel_id = (channels.length() as u32) + 1;

        // Create a new channel and set its properties
        let channel =
            channel::new(
                channel_state::new_try_open(),
                connection_id,
                counterparty_channel_id,
                counterparty_port_id,
                version
            );

        // Commit the created channel to the storage
        state::add_channel_to_port(ibc_uid, channel_id, port_id);

        commit_channel(ibc_uid, channel_id, channel);

        channels.add(channel_id, channel);

        events::emit_channel_open_try(
            port_id,
            channel_id,
            counterparty_port_id,
            counterparty_channel_id,
            connection_id,
            counterparty_version
        );
    }

    public fun channel_open_ack(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
        channels: &mut Table<u32, Channel>,
        port_id: address,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        // Ensure the channel exists and is in the INIT state
        let channel = channels.borrow_mut(channel_id);
        assert!(
            channel.state() == channel_state::new_init(),
            EInvalidChannelState
        );

        let connection_id = channel.connection_id();
        let connection = connections.borrow(connection_id);

        // Construct the expected channel state to verify against the proof
        let expected_channel =
            channel::new(
                channel_state::new_try_open(),
                connection.counterparty_connection_id(),
                channel_id,
                port_id.to_bytes(),
                counterparty_version
            );

        // Verify the channel state using the provided proof and expected state
        let res =
            verify_channel_state(
                client_mgr,
                connection.client_id(),
                proof_height,
                proof_try,
                counterparty_channel_id,
                expected_channel
            );
        assert!(res == 0, res);

        // Update the channel state to OPEN and set the counterparty channel ID
        channel.set_state(channel_state::new_open());
        channel.set_counterparty_channel_id(counterparty_channel_id);
        channel.set_version(counterparty_version);

        // Emit an event for the channel open acknowledgment
        events::emit_channel_open_ack(
            port_id,
            channel_id,
            *channel.counterparty_port_id(),
            counterparty_channel_id,
            connection_id
        );

        // Commit the updated channel to storage
        commit_channel(ibc_uid, channel_id, *channel);
    }

    public fun channel_open_confirm(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
        channels: &mut Table<u32, Channel>,
        port_id: address,
        channel_id: u32,
        proof_ack: vector<u8>,
        proof_height: u64
    ) {
        // Ensure the channel exists and is in the TRYOPEN state
        let channel = channels.borrow_mut(channel_id);
        assert!(
            channel.state() == channel_state::new_try_open(),
            EInvalidChannelState
        );

        let connection_id = channel.connection_id();
        let connection = connections.borrow(connection_id);

        // Construct the expected channel state in the OPEN state to verify against the proof
        let expected_channel =
            channel::new(
                channel_state::new_open(),
                connection.counterparty_connection_id(),
                channel_id,
                port_id.to_bytes(),
                *channel.version()
            );

        // Verify the channel state using the provided proof and expected state
        let res =
            verify_channel_state(
                client_mgr,
                connection.client_id(),
                proof_height,
                proof_ack,
                channel.counterparty_channel_id(),
                expected_channel
            );
        assert!(res == 0, res);

        channel.set_state(channel_state::new_open());

        events::emit_channel_open_confirm(
            port_id,
            channel_id,
            *channel.counterparty_port_id(),
            channel.counterparty_channel_id(),
            channel.connection_id()
        );

        // Commit the final state of the channel to storage
        commit_channel(ibc_uid, channel_id, *channel);
    }

    fun commit_channel(
        ibc_uid: &mut UID, channel_id: u32, channel: Channel
    ) {
        state::add_or_update_commitment(
            ibc_uid,
            commitment::channel_commitment_key(channel_id),
            keccak256(&channel.encode())
        );
    }

    fun verify_channel_state(
        client_mgr: &LightClientManager,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        channel_id: u32,
        channel: Channel
    ): u64 {
        client_mgr.verify_membership(
            client_id,
            height,
            proof,
            commitment::channel_commitment_key(channel_id),
            keccak256(&channel.encode())
        )
    }
}
