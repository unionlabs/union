module ibc::timeout_packet {

    use ibc::engine;
    use ibc::dispatcher;
    use ibc::helpers;
    use ibc::ibc;
    use std::string::{String};

    public entry fun timeout_packet<T: key + store + drop>(
        client_type: String,
        port_id: address,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64,
        _next_sequence_recv: u64
    ) {
        let packet =
            ibc::timeout_packet<T>(
                client_type,
                port_id,
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp,
                proof,
                proof_height,
                _next_sequence_recv
            );

        engine::dispatch<T>(helpers::pack_timeout_packet_params(packet));

        dispatcher::delete_storage<T>();

    }
}
