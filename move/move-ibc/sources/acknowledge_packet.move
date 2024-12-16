module ibc::acknowledge_packet {

    use ibc::packet::{Self, Packet};
    use ibc::channel;
    use ibc::engine;
    use ibc::commitment;
    use ibc::helpers;
    use ibc::ibc;
    use ibc::dispatcher;

    use std::vector;

    public entry fun acknowledge_packet<T: key + store + drop>(
        port_id: address,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64
    ) {
        let port = ibc::get_port_id<T>();
        // assert!(port == port_id, E_UNAUTHORIZED);
        assert!(port == port_id, 2);

        let packets: vector<Packet> = vector::empty();
        let i = 0;
        while (i < vector::length(&packet_source_channels)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_datas, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };
        let l = vector::length(&packets);
        // assert!(l > 0, E_NOT_ENOUGH_PACKETS);
        assert!(l > 0, 1);

        let first_packet = *vector::borrow(&packets, 0);
        let source_channel = packet::source_channel(&first_packet);
        let destination_channel = packet::destination_channel(&first_packet);

        let channel = ibc::ensure_channel_state(source_channel);
        let client_id = ibc::ensure_connection_state(channel::connection_id(&channel));

        let commitment_key;
        if (l == 1) {
            commitment_key = commitment::batch_receipts_commitment_key(
                destination_channel,
                commitment::commit_packet(&first_packet)
            )
        } else {
            commitment_key = commitment::batch_receipts_commitment_key(
                destination_channel,
                commitment::commit_packets(&packets)
            )
        };

        let err =
            ibc::verify_commitment(
                client_id,
                proof_height,
                proof,
                commitment_key,
                commitment::commit_acks(acknowledgements)
            );

        if (err != 0) {
            abort err
        };

        let i = 0;
        while (i < l) {
            let packet = *vector::borrow(&packets, i);
            let commitment_key =
                commitment::batch_packets_commitment_key(
                    source_channel, commitment::commit_packet(&packet)
                );
            ibc::remove_commitment(commitment_key);

            let acknowledgement = *vector::borrow(&acknowledgements, i);
            // onAcknowledgementPacket(...)

            let param = helpers::pack_acknowledge_packet_params(packet, acknowledgement);
            engine::dispatch<T>(param);

            dispatcher::delete_storage<T>();

            ibc::emit_acknowledge_packet(packet, acknowledgement);

            i = i + 1;
        }
    }
}
