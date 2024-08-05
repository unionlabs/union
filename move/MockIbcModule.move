module IBCModuleAddr::IBCModule {
    use std::vector;
    use IBC::height;
    use aptos_std::string::{String};
    // use IBC::Core;

    // Order Enums
    const ORDER_UNORDERED: u8 = 1;
    const ORDER_ORDERED: u8 = 2;
    
    struct IbcCoreChannelV1Packet has copy, store, drop, key { // TODO: if i try to use this from core, it creates cyclic dependency
        sequence: u64,
        source_port: String,
        source_channel: String,
        destination_port: String,
        destination_channel: String,
        data: vector<u8>,
        timeout_height: height::Height,
        timeout_timestamp: u64,
    }


    public fun on_chan_open_init(
        _order: u8, // ORDER_ORDERED or ORDER_UNORDERED
        _connection_hops: vector<String>,
        _port_id: String,
        _channel_id: String,
        _counterparty_port_id: String, 
        _counterparty_channel_id: String, 
        _version: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_open_try(
        _order: u8, // ORDER_ORDERED or ORDER_UNORDERED
        _connection_hops: vector<String>,
        _port_id: String,
        _channel_id: String,
        _counterparty_port_id: String, 
        _counterparty_channel_id: String, 
        _version: String,
        _counterparty_version: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_open_ack(
        _port_id: String,
        _channel_id: String,
        _counterparty_channel_id: String,
        _counterparty_version: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_open_confirm(
        _port_id: String,
        _channel_id: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_close_init(
        _port_id: String,
        _channel_id: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_close_confirm(
        _port_id: String,
        _channel_id: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_recv_packet(
        _packet: IbcCoreChannelV1Packet,
        _relayer: address
    ): vector<u8> {
        // Implement your logic here
        vector::empty<u8>() // Return an empty vector for success
    }

    public fun on_acknowledgement_packet(
        _packet: IbcCoreChannelV1Packet,
        _acknowledgement: vector<u8>,
        _relayer: address
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_timeout_packet(
        _packet: IbcCoreChannelV1Packet,
        _relayer: address
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }
}