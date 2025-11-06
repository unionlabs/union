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

module ibc::ibc_connection {
    use sui::hash::keccak256;
    use sui::table::Table;

    use ibc::commitment;
    use ibc::connection::{Self, Connection};
    use ibc::events;
    use ibc::light_client::LightClientManager;
    use ibc::state;

    const EInvalidConnectionState: u64 = 3;

    const CONN_STATE_INIT: u8 = 1;
    const CONN_STATE_TRYOPEN: u8 = 2;
    const CONN_STATE_OPEN: u8 = 3;

    public(package) fun connection_open_init(
        ibc_uid: &mut UID,
        connections: &mut Table<u32, Connection>,
        client_id: u32,
        counterparty_client_id: u32
    ) {
        let connection_id = (connections.length() as u32) + 1;

        let connection =
            connection::new(
                CONN_STATE_INIT,
                client_id,
                counterparty_client_id,
                0
            );

        commit_connection(ibc_uid, connection_id, connection);

        connections.add(connection_id, connection);

        ibc::events::emit_connection_open_init(
            connection_id,
            client_id,
            counterparty_client_id
        );
    }

    public(package) fun connection_open_try(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &mut Table<u32, Connection>,
        counterparty_client_id: u32,
        counterparty_connection_id: u32,
        client_id: u32,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        let connection_id = (connections.length() as u32) + 1;

        let connection =
            connection::new(
                CONN_STATE_TRYOPEN,
                client_id,
                counterparty_client_id,
                counterparty_connection_id
            );

        // Construct the expected connection state to verify against the proof
        let expected_connection =
            connection::new(
                CONN_STATE_INIT,
                counterparty_client_id,
                client_id,
                0 // counterparty_connection_id
            );

        // Verify the connection state using the provided proof and expected state
        let res =
            verify_connection_state(
                client_mgr,
                connection.client_id(),
                proof_height,
                proof_init,
                counterparty_connection_id,
                expected_connection
            );

        assert!(res == 0, res);

        commit_connection(ibc_uid, connection_id, connection);

        connections.add(connection_id, connection);

        events::emit_connection_open_try(
            connection_id,
            client_id,
            counterparty_client_id,
            counterparty_connection_id
        );
    }

    public(package) fun connection_open_ack(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &mut Table<u32, Connection>,
        connection_id: u32,
        counterparty_connection_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        let connection = connections.borrow_mut(connection_id);

        // assert that this connection is at the `INIT` phase
        assert!(
            connection.state() == CONN_STATE_INIT,
            EInvalidConnectionState
        );

        // Create the expected connection state to verify against the proof
        let expected_connection =
            connection::new(
                CONN_STATE_TRYOPEN,
                connection.counterparty_client_id(),
                connection.client_id(),
                connection_id
            );

        // Verify the connection state using the provided proof and expected state
        let res =
            verify_connection_state(
                client_mgr,
                connection.client_id(),
                proof_height,
                proof_try,
                counterparty_connection_id,
                expected_connection
            );
        assert!(res == 0, res);

        // Update the connection state to TRYOPEN and set the counterparty connection ID
        connection.set_state(CONN_STATE_OPEN);
        connection.set_counterparty_connection_id(counterparty_connection_id);

        events::emit_connection_open_ack(
            connection_id,
            connection.client_id(),
            connection.counterparty_client_id(),
            connection.counterparty_connection_id()
        );

        // Commit the updated connection to storage
        commit_connection(ibc_uid, connection_id, *connection);
    }

    public(package) fun connection_open_confirm(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &mut Table<u32, Connection>,
        connection_id: u32,
        proof_ack: vector<u8>,
        proof_height: u64
    ) {
        let connection = connections.borrow_mut(connection_id);
        assert!(
            connection.state() == CONN_STATE_TRYOPEN,
            EInvalidConnectionState
        );

        // Create the expected connection state in the `OPEN` state to verify against the proof
        let expected_connection =
            connection::new(
                CONN_STATE_OPEN,
                connection.counterparty_client_id(),
                connection.client_id(),
                connection_id
            );

        let counterparty_connection_id = connection.counterparty_connection_id();

        // Verify the connection state using the provided proof and expected state
        let res =
            verify_connection_state(
                client_mgr,
                connection.client_id(),
                proof_height,
                proof_ack,
                counterparty_connection_id,
                expected_connection
            );
        assert!(res == 0, res);

        // Update the connection state to OPEN
        connection.set_state(CONN_STATE_OPEN);

        // Emit an event for connection confirmation
        events::emit_connection_open_confirm(
            connection_id,
            connection.client_id(),
            connection.counterparty_client_id(),
            counterparty_connection_id
        );

        // Commit the final state of the connection to storage
        commit_connection(ibc_uid, connection_id, *connection);
    }

    fun commit_connection(
        ibc_uid: &mut UID, connection_id: u32, connection: Connection
    ) {
        state::add_or_update_commitment(
            ibc_uid,
            commitment::connection_commitment_key(connection_id),
            keccak256(&connection.encode())
        );
    }

    fun verify_connection_state(
        client_mgr: &LightClientManager,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        connection_id: u32,
        counterparty_connection: Connection
    ): u64 {
        client_mgr.verify_membership(
            client_id,
            height,
            proof,
            commitment::connection_commitment_key(connection_id),
            keccak256(&counterparty_connection.encode())
        )
    }
}
