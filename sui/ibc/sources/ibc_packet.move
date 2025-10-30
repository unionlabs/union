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

module ibc::ibc_packet {
    use std::string::String;

    use sui::table::Table;
    use sui::clock::{Self, Clock};

    use ibc::commitment;
    use ibc::channel::{Self, Channel};
    use ibc::connection_end::ConnectionEnd;
    use ibc::events;
    use ibc::light_client::LightClientManager;
    use ibc::packet::Packet;
    use ibc::state;

    const COMMITMENT_MAGIC: vector<u8>     = x"0100000000000000000000000000000000000000000000000000000000000000";
    const COMMITMENT_MAGIC_ACK: vector<u8> = x"0200000000000000000000000000000000000000000000000000000000000000";

    const CHAN_STATE_OPEN: u8 = 3;
    const CONN_STATE_OPEN: u8 = 3;

    const E_ACK_LEN_MISMATCH: u64 = 2;
    const E_MAKER_MSG_LEN_MISMATCH: u64 = 3;
    const E_PACKET_COMMITMENT_NOT_FOUND: u64 = 4;
    const E_PACKET_ALREADY_ACKNOWLEDGED: u64 = 5;
    const E_ACK_ALREADY_EXIST: u64 = 6;
    const E_PACKET_NOT_RECEIVED: u64 = 7;
    const E_PACKET_ALREADY_RECEIVED: u64 = 8;
    const E_ALREADY_RECEIVED: u64 = 9;
    const E_TIMESTAMP_TIMEOUT: u64 = 10;
    const E_TIMEOUT_HEIGHT_NOT_SUPPORTED: u64 = 11;
    const E_BATCH_SAME_CHANNEL_ONLY: u64 = 12;
    const E_CLIENT_NOT_FOUND: u64 = 13;
    const E_INVALID_CONNECTION_STATE: u64 = 14;
    const E_INVALID_CHANNEL_STATE: u64 = 15;
    const E_CHANNEL_NOT_FOUND: u64 = 16;
    const E_NOT_ENOUGH_PACKETS: u64 = 17;

    public fun recv_packet(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, ConnectionEnd>,
        channels: &Table<u32, Channel>,
        clock: &clock::Clock,
        packets: vector<Packet>,
        maker: address,
        maker_msgs: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64,
        acknowledgements: vector<vector<u8>>,
    ) {
        process_receive(
            ibc_uid,
            client_mgr,
            connections,
            channels,
            clock,
            packets,
            maker,
            maker_msgs,
            proof_height,
            proof,
            false,
            acknowledgements
        );
    }


    fun process_receive(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, ConnectionEnd>,
        channels: &Table<u32, Channel>,
        clock: &Clock,
        packets: vector<Packet>,
        maker: address,
        maker_msgs: vector<vector<u8>>,
        proof_height: u64,
        proof: vector<u8>,
        intent: bool,
        acknowledgements: vector<vector<u8>>
    ) {
        let l = packets.length();
        assert!(l != 0, E_NOT_ENOUGH_PACKETS);

        assert!(l == acknowledgements.length(), E_ACK_LEN_MISMATCH);
        assert!(l == maker_msgs.length(), E_MAKER_MSG_LEN_MISMATCH);

        let destination_channel = packets[0].destination_channel_id();

        if(!channels.contains(destination_channel)) {
            abort E_CHANNEL_NOT_FOUND
        };
        let channel = channels.borrow(destination_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, E_INVALID_CHANNEL_STATE);

        let connection = connections.borrow(channel.connection_id()); 
        assert!(connection.state() == CONN_STATE_OPEN, E_INVALID_CONNECTION_STATE);

        let client_id = connection.client_id();

        if(!client_mgr.exists(client_id)) {
            abort E_CLIENT_NOT_FOUND
        };

        if (!intent) {
            let commitment_key = commitment::batch_packets_commitment_key(
                commitment::commit_packets(&packets)
            );

            let err =
                client_mgr.verify_membership(
                    client_id,
                    proof_height,
                    proof,
                    commitment_key,
                    COMMITMENT_MAGIC
                );

            if (err != 0) {
                abort err
            };
        };
        let mut i = 0;
        while (i < l) {
            let packet = packets[i];

            assert!(packet.destination_channel_id() == destination_channel, E_BATCH_SAME_CHANNEL_ONLY);

            if (packet.timeout_height() != 0) {
                abort E_TIMEOUT_HEIGHT_NOT_SUPPORTED
            };


            let current_timestamp = clock::timestamp_ms(clock) * 1_000_000; 
            assert!(
                current_timestamp < packet.timeout_timestamp(),
                E_TIMESTAMP_TIMEOUT
            );

            let packet_hash = commitment::commit_packet(&packet);
            let commitment_key =
                commitment::batch_receipts_commitment_key(
                    packet_hash
                );


            if(set_packet_receive(ibc_uid, commitment_key)) {
                // Normally this is not an error and results in noop in the traditional impls where
                // the app is called by the ibc. But since it is the other way here, we have to abort
                // to prevent double processing in the app side. 
                abort E_ALREADY_RECEIVED;
            } else {
                let maker_msg = maker_msgs[i];
                if (intent) {
                    events::emit_intent_packet_recv(
                        destination_channel,
                        packet_hash,
                        maker,
                        maker_msg
                    );
                } else {
                    events::emit_packet_recv(
                        destination_channel,
                        packet_hash,
                        maker,
                        maker_msg
                    );
                };

                let acknowledgement = acknowledgements[i];
                if (!acknowledgement.is_empty()) {
                    inner_write_acknowledgement(ibc_uid, commitment_key, acknowledgement);
                    events::emit_write_ack(
                        destination_channel,
                        packet_hash,
                        acknowledgement
                    );
                };
            };
            i = i + 1;
        }
    }

    fun inner_write_acknowledgement(
        ibc_uid: &mut UID,
        commitment_key: vector<u8>, acknowledgement: vector<u8>
    ) {
        if (!state::has_commitment(ibc_uid, commitment_key)) {
            abort E_PACKET_NOT_RECEIVED
        };
        let commitment = state::borrow_commitment_mut(ibc_uid, commitment_key);
        assert!(
            *commitment == COMMITMENT_MAGIC,
            E_ACK_ALREADY_EXIST
        );

        *commitment = commitment::commit_ack(acknowledgement);
    }

    fun set_packet_receive(ibc_uid: &mut UID, commitment_key: vector<u8>): bool {
        if (state::has_commitment(ibc_uid, commitment_key)) {
            true
        } else {
            state::commit(ibc_uid, commitment_key, COMMITMENT_MAGIC);
            false
        }
    }

    fun set_packet_acknowledged(ibc_uid: &mut UID, commitment_key: vector<u8>) {
        assert!(state::has_commitment(ibc_uid, commitment_key), E_PACKET_COMMITMENT_NOT_FOUND);

        let commitment = state::borrow_commitment_mut(ibc_uid, commitment_key);
        assert!(commitment != COMMITMENT_MAGIC_ACK, E_PACKET_ALREADY_ACKNOWLEDGED);
        assert!(commitment == COMMITMENT_MAGIC, E_PACKET_COMMITMENT_NOT_FOUND);

        *commitment = COMMITMENT_MAGIC_ACK;
    }
}
