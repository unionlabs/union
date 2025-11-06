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

#[allow(implicit_const_copy)]
module ibc::ibc_packet {
    use sui::table::Table;
    use sui::clock::{Self, Clock};

    use ibc::commitment;
    use ibc::channel::Channel;
    use ibc::connection::Connection;
    use ibc::events;
    use ibc::light_client::LightClientManager;
    use ibc::packet::{Self, Packet};
    use ibc::state;

    const COMMITMENT_MAGIC: vector<u8>     = x"0100000000000000000000000000000000000000000000000000000000000000";
    const COMMITMENT_MAGIC_ACK: vector<u8> = x"0200000000000000000000000000000000000000000000000000000000000000";

    const CHAN_STATE_OPEN: u8 = 3;
    const CONN_STATE_OPEN: u8 = 3;

    const EBase: u64 = 10400;
    const EAckLenMismatch: u64 = EBase + 1;
    const EMakerMsgLenMismatch: u64 = EBase + 2;
    const EPacketCommitmentNotFound: u64 = EBase + 3;
    const EPacketAlreadyAcknowledged: u64 = EBase + 4;
    const EAckAlreadyExist: u64 = EBase + 5;
    const EPacketNotReceived: u64 = EBase + 6;
    const EPacketAlreadyReceived: u64 = EBase + 7;
    const EAlreadyReceived: u64 = EBase + 8;
    const ETimestampTimeout: u64 = EBase + 9;
    const ETimeoutHeightNotSupported: u64 = EBase + 10;
    const EBatchSameChannelOnly: u64 = EBase + 11;
    const EClientNotFound: u64 = EBase + 12;
    const EInvalidConnectionState: u64 = EBase + 13;
    const EInvalidChannelState: u64 = EBase + 14;
    const EChannelNotFound: u64 = EBase + 15;
    const ENotEnoughPackets: u64 = EBase + 16;
    const EPacketAlreadySent: u64 = EBase + 17;
    const EAcknowledgementIsEmpty: u64 = EBase + 18;
    const ETimestampTimeoutNotReached: u64 = EBase + 19;
    const ETimeoutMustBeSet: u64 = EBase + 20;
    const ELatestTimestampNotFound: u64 = EBase + 21;
    const EConnectionNotFound: u64 = EBase + 22;
    const EPacketHaveNotTimedOut: u64 = EBase + 23;

    public(package) fun send_packet(
        ibc_uid: &mut UID,
        channels: &Table<u32, Channel>,
        clock: &Clock,
        source_channel: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        data: vector<u8>,
        ctx: &TxContext
    ): packet::Packet {
        // Check if the channel exists in the store
        if(!channels.contains(source_channel)) {
            abort EChannelNotFound
        };

        // Validate timeout values
        assert!(
            timeout_height == 0, ETimeoutHeightNotSupported
        );

        assert!(
            timeout_timestamp > (clock.timestamp_ms() * 1_000_000),
            ETimestampTimeout
        );

        let channel = *channels.borrow(source_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, EInvalidChannelState);
        
        // Prepare packet for commitment
        let packet =
            packet::new(
                source_channel,
                channel.counterparty_channel_id(),
                data,
                timeout_height,
                timeout_timestamp
            );
        let packet_hash = commitment::commit_packet(&packet);
        let commitment_key =
            commitment::batch_packets_commitment_key(
                packet_hash
            );

        assert!(!state::has_commitment(ibc_uid, commitment_key), EPacketAlreadySent);

        state::commit(ibc_uid, commitment_key, COMMITMENT_MAGIC);

        // This is very important for the relayers to be able to get the exact transaction from the `packet_hash`.
        // They will later use this to get the full packet.
        state::add_commitment_to_digest(ibc_uid, packet_hash, *ctx.digest());

        events::emit_packet_send(
            source_channel,
            packet_hash,
            packet
        );

        packet
    }

    public(package) fun recv_packet(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
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

    public(package) fun recv_intent_packet(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
        channels: &Table<u32, Channel>,
        clock: &clock::Clock,
        packets: vector<Packet>,
        maker: address,
        maker_msgs: vector<vector<u8>>,
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
            0,
            vector::empty(),
            true,
            acknowledgements
        );
    }

    public(package) fun write_acknowledgement(
        ibc_uid: &mut UID,
        channels: &Table<u32, Channel>,
        packet: packet::Packet,
        acknowledgement: vector<u8>,
    ) {
        assert!(!acknowledgement.is_empty(), EAcknowledgementIsEmpty);

        if(!channels.contains(packet.destination_channel_id())) {
            abort EChannelNotFound
        };

        let channel = *channels.borrow(packet.destination_channel_id());
        assert!(channel.state() == CHAN_STATE_OPEN, EInvalidChannelState);

        let packet_hash = commitment::commit_packet(&packet);

        let commitment_key =
            commitment::batch_receipts_commitment_key(
                packet_hash
            );

        inner_write_acknowledgement(ibc_uid, commitment_key, acknowledgement);

        events::emit_write_ack(
            packet.destination_channel_id(),
            packet_hash,
            acknowledgement,
        )
    }

    public(package) fun acknowledge_packet(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
        channels: &Table<u32, Channel>,
        packets: vector<packet::Packet>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64,
        relayer: address,
    )  {
        let l = packets.length();
        assert!(l > 0, ENotEnoughPackets);

        let source_channel = packets[0].source_channel_id();

        if(!channels.contains(source_channel)) {
            abort EChannelNotFound
        };
        let channel = channels.borrow(source_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, EInvalidChannelState);

        let connection = connections.borrow(channel.connection_id());
        assert!(connection.state() == CONN_STATE_OPEN, EInvalidConnectionState);

        let client_id = connection.client_id();

        if (!client_mgr.exists(client_id)) {
            abort EClientNotFound
        };

        let commitment_key = commitment::batch_receipts_commitment_key(
            commitment::commit_packets(&packets)
        );

        let err =
            client_mgr.verify_membership(
                client_id,
                proof_height,
                proof,
                commitment_key,
                commitment::commit_acks(acknowledgements)
            );

        assert!(err == 0, err);

        let mut i = 0;
        while (i < l) {
            let packet = packets[i];

            assert!(packet.source_channel_id() == source_channel, EBatchSameChannelOnly);

            let packet_hash = commitment::commit_packet(&packet);
            let commitment_key =
                commitment::batch_packets_commitment_key(
                    packet_hash
                );
            set_packet_acknowledged(ibc_uid, commitment_key);
            
            events::emit_packet_ack(
                source_channel,
                packet_hash,
                acknowledgements[i],
                relayer
            );

            i = i + 1;
        }
    }

    public(package) fun timeout_packet(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
        channels: &Table<u32, Channel>,
        packet: Packet,
        proof: vector<u8>,
        proof_height: u64,
    ) {
        let source_channel = packet.source_channel_id();

        if(!channels.contains(source_channel)) {
            abort EChannelNotFound
        };
        let channel = channels.borrow(source_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, EInvalidChannelState);

        let connection_id = channel.connection_id();

        if(!connections.contains(connection_id)) {
            abort EConnectionNotFound
        };
        let connection = connections.borrow(connection_id);
        assert!(
            connection.state() == CONN_STATE_OPEN,
            EInvalidConnectionState
        );
        let client_id = connection.client_id();

        if(!client_mgr.exists(client_id)) {
            abort EClientNotFound
        };
        let proof_timestamp =
            client_mgr.get_timestamp_at_height(client_id, proof_height);
        assert!(proof_timestamp != 0, ELatestTimestampNotFound);

        let packet_hash = commitment::commit_packet(&packet);
        let commitment_key = commitment::batch_receipts_commitment_key(packet_hash);

        let err = client_mgr.verify_non_membership(
            client_id, proof_height, proof, commitment_key);

        assert!(err == 0, err);

        if (packet.timeout_timestamp() == 0) {
            abort ETimeoutMustBeSet
        } else {
            assert!(
                packet.timeout_timestamp() < proof_timestamp,
                ETimestampTimeoutNotReached
            );
        };

        set_packet_acknowledged(
            ibc_uid,
            commitment::batch_packets_commitment_key(
                packet_hash
            )
        );

        events::emit_timeout_packet(packet_hash);
    }

    public(package) fun commit_packet_timeout(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
        channels: &Table<u32, Channel>,
        clock: &Clock,
        packet: Packet,
        proof: vector<u8>,
        proof_height: u64,
        ctx: &TxContext,
    ) {
        let current_timestamp = clock::timestamp_ms(clock) * 1_000_000; 
        assert!(
            current_timestamp >= packet.timeout_timestamp(),
            EPacketHaveNotTimedOut
        );

        let packet_hash = commitment::commit_packet(&packet);

        assert!(
            !state::has_commitment(ibc_uid, commitment::batch_receipts_commitment_key(packet_hash)),
            EPacketAlreadyReceived
        );

        let channel = channels.borrow(packet.destination_channel_id());
        assert!(channel.state() == CHAN_STATE_OPEN, EInvalidChannelState);

        let connection_id = channel.connection_id();

        let connection = connections.borrow(connection_id);
        assert!(
            connection.state() == CONN_STATE_OPEN,
            EInvalidConnectionState
        );
        let client_id = connection.client_id();

        // make sure that the packet is sent
        let err =
            client_mgr.verify_membership(
                client_id,
                proof_height,
                proof,
                commitment::batch_packets_commitment_key(
                    packet_hash
                ),
                COMMITMENT_MAGIC
            );

        if (err != 0) {
            abort err
        };

        let commitment_key = commitment::packet_timeout_commitment_key(packet_hash);

        state::add_commitment_to_digest(
            ibc_uid,
            commitment_key,
            *ctx.digest()
        );

        state::commit(
            ibc_uid,
            commitment_key,
            COMMITMENT_MAGIC
        );
    }


    fun process_receive(
        ibc_uid: &mut UID,
        client_mgr: &LightClientManager,
        connections: &Table<u32, Connection>,
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
        assert!(l != 0, ENotEnoughPackets);

        assert!(l == acknowledgements.length(), EAckLenMismatch);
        assert!(l == maker_msgs.length(), EMakerMsgLenMismatch);

        let destination_channel = packets[0].destination_channel_id();

        if(!channels.contains(destination_channel)) {
            abort EChannelNotFound
        };
        let channel = channels.borrow(destination_channel);
        assert!(channel.state() == CHAN_STATE_OPEN, EInvalidChannelState);

        let connection = connections.borrow(channel.connection_id()); 
        assert!(connection.state() == CONN_STATE_OPEN, EInvalidConnectionState);

        let client_id = connection.client_id();

        if(!client_mgr.exists(client_id)) {
            abort EClientNotFound
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

            assert!(packet.destination_channel_id() == destination_channel, EBatchSameChannelOnly);

            if (packet.timeout_height() != 0) {
                abort ETimeoutHeightNotSupported
            };


            let current_timestamp = clock::timestamp_ms(clock) * 1_000_000; 
            assert!(
                current_timestamp < packet.timeout_timestamp(),
                ETimestampTimeout
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
                abort EAlreadyReceived;
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
            abort EPacketNotReceived
        };
        let commitment = state::borrow_commitment_mut(ibc_uid, commitment_key);
        assert!(
            *commitment == COMMITMENT_MAGIC,
            EAckAlreadyExist
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
        assert!(state::has_commitment(ibc_uid, commitment_key), EPacketCommitmentNotFound);

        let commitment = state::borrow_commitment_mut(ibc_uid, commitment_key);
        assert!(commitment != COMMITMENT_MAGIC_ACK, EPacketAlreadyAcknowledged);
        assert!(commitment == COMMITMENT_MAGIC, EPacketCommitmentNotFound);

        *commitment = COMMITMENT_MAGIC_ACK;
    }
}
