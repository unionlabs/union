module ibc::channel_handshake {
    use std::string::{String};
    use ibc::ibc;
    use ibc::dispatcher;
    use ibc::engine;
    use ibc::helpers;

    public entry fun channel_open_init<T: key + store + drop>(
        port_id: address,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    ) {
        let (channel_id, connection_id) =
            ibc::channel_open_init<T>(
                port_id,
                counterparty_port_id,
                connection_id,
                version
            );

        engine::dispatch<T>(
            helpers::pack_channel_open_init_params(connection_id, channel_id, version)
        );

        dispatcher::delete_storage<T>();
    }

    public entry fun channel_open_try<T: key + store + drop>(
        port_id: address,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        let channel_id =
            ibc::channel_open_try<T>(
                port_id,
                connection_id,
                counterparty_channel_id,
                counterparty_port_id,
                version,
                counterparty_version,
                proof_init,
                proof_height
            );

        engine::dispatch<T>(
            helpers::pack_channel_open_try_params(
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            )
        );

        dispatcher::delete_storage<T>();
    }

    public entry fun channel_open_ack<T: key + store + drop>(
        port_id: address,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        ibc::channel_open_ack<T>(
            port_id,
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height
        );

        engine::dispatch<T>(
            helpers::pack_channel_open_ack_params(
                channel_id,
                counterparty_channel_id,
                counterparty_version
            )
        );

        dispatcher::delete_storage<T>();
    }

    public entry fun channel_open_confirm<T: key + store + drop>(
        port_id: address,
        channel_id: u32,
        proof_ack: vector<u8>,
        proof_height: u64
    ) {
        ibc::channel_open_confirm<T>(port_id, channel_id, proof_ack, proof_height);

        engine::dispatch<T>(helpers::pack_channel_open_confirm_params(channel_id));

        dispatcher::delete_storage<T>();
    }
}
