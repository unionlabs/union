module ibc::recv_packet {
    use ibc::packet::{Self, Packet};
    use std::string::{String};
    use ibc::channel;
    use ibc::engine;
    use ibc::commitment;
    use ibc::helpers;
    use ibc::ibc;
    use ibc::dispatcher;

    use std::vector;
    use std::timestamp;
    use std::block;

    const COMMITMENT_MAGIC: vector<u8> = x"0100000000000000000000000000000000000000000000000000000000000000";

    /// Receives and processes an IBC packet
    ///
    /// Note that any sanity check failures will result in this function to be aborted in order for caller's
    /// storage to be reverted. This will result in acks won't be able to written.
    public entry fun recv_packet<T: key + store + drop>(
        client_type: String,
        port_id: address,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_data: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        proof: vector<u8>,
        proof_height: u64
    ) {
        let port = ibc::get_port_id<T>();
        assert!(port == port_id, 100);

        let packets: vector<Packet> = vector::empty();
        let i = 0;
        while (i < vector::length(&packet_source_channels)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_data, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };

        process_receive<T>(
            client_type,
            packets,
            proof_height,
            proof,
            false
        );
    }

    public fun process_receive<T: key + store + drop>(
        client_type: String,
        packets: vector<Packet>,
        proof_height: u64,
        proof: vector<u8>,
        intent: bool
    ) {
        let l = vector::length(&packets);
        // assert!(l > 0, E_NOT_ENOUGH_PACKETS);

        assert!(l > 0, 2);

        let first_packet = *vector::borrow(&packets, 0);
        let source_channel = packet::source_channel(&first_packet);
        let destination_channel = packet::destination_channel(&first_packet);

        let channel = ibc::ensure_channel_state(destination_channel);
        let client_id = ibc::ensure_connection_state(channel::connection_id(&channel));

        if (!intent) {
            let commitment_key;
            if (l == 1) {
                commitment_key = commitment::batch_packets_commitment_key(
                    source_channel,
                    commitment::commit_packet(&first_packet)
                )
            } else {
                commitment_key = commitment::batch_packets_commitment_key(
                    source_channel,
                    commitment::commit_packets(&packets)
                )
            };

            let err =
                ibc::verify_commitment(
                    client_type,
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

        let i = 0;
        while (i < l) {
            let packet = *vector::borrow(&packets, i);

            if (packet::timeout_height(&packet) != 0) {
                assert!(
                    block::get_current_block_height() < packet::timeout_height(&packet),
                    // E_HEIGHT_TIMEOUT
                    1
                );
            };

            let current_timestamp = timestamp::now_seconds() * 1_000_000_000; // 1e9
            if (packet::timeout_timestamp(&packet) != 0) {
                assert!(
                    current_timestamp < packet::timeout_timestamp(&packet),
                    // E_TIMESTAMP_TIMEOUT
                    1
                );
            };

            let commitment_key =
                commitment::batch_receipts_commitment_key(
                    destination_channel,
                    commitment::commit_packet(&packet)
                );

            if (!set_packet_receive(commitment_key)) {
                let acknowledgement =
                    if (intent) {
                        let param = helpers::pack_recv_intent_packet_params(packet);
                        engine::dispatch<T>(param);

                        let ack = dispatcher::get_return_value<T>();

                        dispatcher::delete_storage<T>();
                        ibc::emit_recv_intent_packet(packet);
                        ack
                    } else {
                        let param = helpers::pack_recv_packet_params(packet);
                        engine::dispatch<T>(param);

                        let ack = dispatcher::get_return_value<T>();

                        dispatcher::delete_storage<T>();
                        ibc::emit_recv_packet(packet);
                        ack
                    };
                if (vector::length(&acknowledgement) > 0) {
                    ibc::inner_write_acknowledgement(
                        commitment_key, packet, acknowledgement
                    );
                };
            };
            i = i + 1;
        }
    }

    fun set_packet_receive(commitment_key: vector<u8>): bool {
        if (ibc::get_commitment(commitment_key) != vector::empty()) { true }
        else {
            ibc::set_commitment(commitment_key, COMMITMENT_MAGIC);
            false
        }
    }
}
