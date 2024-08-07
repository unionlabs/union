module IBCModuleAddr::IBCModule {
    use std::vector;
    use IBC::height;
    use aptos_std::string::{Self, String};
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
        order: u8, // ORDER_ORDERED or ORDER_UNORDERED
        connection_hops: vector<String>,
        port_id: String,
        channel_id: String,
        counterparty_port_id: String, 
        counterparty_channel_id: String, 
        version: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_open_try(
        order: u8, // ORDER_ORDERED or ORDER_UNORDERED
        connection_hops: vector<String>,
        port_id: String,
        channel_id: String,
        counterparty_port_id: String, 
        counterparty_channel_id: String, 
        version: String,
        counterparty_version: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_open_ack(
        port_id: String,
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_open_confirm(
        port_id: String,
        channel_id: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_close_init(
        port_id: String,
        channel_id: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_chan_close_confirm(
        port_id: String,
        channel_id: String,
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_recv_packet(
        packet: IbcCoreChannelV1Packet,
        relayer: address
    ): vector<u8> {
        // Implement your logic here
        vector::empty<u8>() // Return an empty vector for success
    }

    public fun on_acknowledgement_packet(
        packet: IbcCoreChannelV1Packet,
        acknowledgement: vector<u8>,
        relayer: address
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }

    public fun on_timeout_packet(
        packet: IbcCoreChannelV1Packet,
        relayer: address
    ): u8 {
        // Implement your logic here
        0 // Return 0 for success
    }
}
