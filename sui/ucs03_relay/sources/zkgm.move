module ucs03::zkgm_relay {
    use ibc::ibc;
    use ibc::packet::{Self, Packet};
    use std::string::{Self, String, utf8};
    use sui::table::{Self, Table};
    use ucs03::ethabi;
    use sui::bcs;
    use sui::clock;
    use sui::address::{to_string};
    use sui::event;
    use sui::coin::{Self, Coin, TreasuryCap, CoinMetadata};

        // Constants
    const ACK_SUCCESS: u8 = 1;
    const ACK_FAILURE: u8 = 0;
    const ACK_LENGTH: u64 = 1;

    // Errors
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
    const VERSION: vector<u8> = b"ucs03-zkgm-0";
    const E_INVALID_BYTES_ADDRESS: u64 = 1;
    const E_UNAUTHORIZED: u64 = 2;
    const E_INVALID_ACKNOWLEDGEMENT: u64 = 3;
    const E_INVALID_PROTOCOL_VERSION: u64 = 4;
    const E_INVALID_COUNTERPARTY_PROTOCOL_VERSION: u64 = 6;
    const E_INVALID_AMOUNT: u64 = 7;
    const E_UNSTOPPABLE: u64 = 8;

    public struct ZkgmPacket has copy, drop, store {
        salt: vector<u8>,
        path: u256,
        syscall: vector<u8>
    }

    public struct SyscallPacket has copy, drop, store {
        version: u8,
        index: u8,
        packet: vector<u8>
    }

    public struct ForwardPacket has copy, drop, store {
        channel_id: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        syscall_packet: vector<u8>
    }

    public struct MultiplexPacket has copy, drop, store {
        sender: vector<u8>,
        eureka: bool,
        contract_address: vector<u8>,
        contract_calldata: vector<u8>
    }

    public struct BatchPacket has copy, drop, store {
        syscall_packets: vector<vector<u8>>
    }

    public struct OnZkgmParams has copy, drop, store {
        sender: vector<u8>,
        contract_calldata: vector<u8>
    }

    public struct IIBCModuleOnRecvPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>
    }

    public struct IIBCModuleOnAcknowledgementPacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address
    }

    public struct IIBCModuleOnTimeoutPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address
    }

    public struct FungibleAssetTransferPacket has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        sent_token: vector<u8>,
        sent_token_prefix: u256,
        sent_symbol: string::String,
        sent_name: string::String,
        sent_amount: u64,
        ask_token: vector<u8>,
        ask_amount: u64,
        only_maker: bool
    }

    public struct Acknowledgement has copy, drop, store {
        tag: u256,
        inner_ack: vector<u8>
    }

    public struct BatchAcknowledgement has copy, drop, store {
        acknowledgements: vector<vector<u8>>
    }

    public struct AssetTransferAcknowledgement has copy, drop, store {
        fill_type: u256,
        market_maker: vector<u8>
    }

    public struct ChannelBalancePair has copy, drop, store {
        channel: u32,
        token: address
    }

    public struct RelayStore has key {
        id: UID,
        in_flight_packet: Table<vector<u8>, Packet>,
        channel_balance: Table<ChannelBalancePair, u256>,
        token_origin: Table<address, u256>
    }

    // Events
    #[event]
    public struct DenomCreated has copy, drop, store {
        channel_id: u32,
        denom: String,
        token: address
    }

    #[event]
    public struct Received has copy, drop, store {
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }

    #[event]
    public struct Sent has copy, drop, store {
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }

    #[event]
    public struct Refunded has copy, drop, store {
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }
    
    fun init(ctx: &mut TxContext) {
        let id = object::new(ctx);

        transfer::share_object(RelayStore {
            id: id,
            in_flight_packet: table::new(ctx),
            channel_balance: table::new(ctx),
            token_origin: table::new(ctx)
        });
    }

    public fun decode_ack(buf: vector<u8>): Acknowledgement {
        let mut index = 0x20;
        let tag = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let inner_ack =
            ethabi::decode_vector!<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );

        Acknowledgement { tag: tag, inner_ack: inner_ack }
    }

    public fun encode_ack(packet: &Acknowledgement): vector<u8> {
        let mut buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u256>(&mut buf, packet.tag);

        let version_offset = 0x40;
        ethabi::encode_uint<u32>(&mut buf, version_offset);

        ethabi::encode_vector!<u8>(
            &mut buf,
            &packet.inner_ack,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun encode_asset_transfer_ack(
        ack: &AssetTransferAcknowledgement
    ): vector<u8> {
        let mut buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u256>(&mut buf, ack.fill_type);

        let version_offset = 0x40;
        ethabi::encode_uint<u32>(&mut buf, version_offset);

        ethabi::encode_vector!<u8>(
            &mut buf,
            &ack.market_maker,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun decode_asset_transfer_ack(buf: vector<u8>): AssetTransferAcknowledgement {
        let mut index = 0x20;
        let fill_type = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let market_maker =
            ethabi::decode_vector!<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );

        AssetTransferAcknowledgement { fill_type: fill_type, market_maker: market_maker }
    }

    public fun decode_batch_ack(buf: vector<u8>): BatchAcknowledgement {
        let mut index = 0x40;
        let main_arr_length = ethabi::decode_uint(&buf, &mut index);
        index = index + (0x20 * main_arr_length as u64);

        let mut idx = 0;
        let mut acknowledgements = vector::empty();
        while (idx < main_arr_length) {
            let inner_vec =
                ethabi::decode_vector!<u8>(
                    &buf,
                    &mut index,
                    |buf, index| {
                        (ethabi::decode_uint(buf, index) as u8)
                    }
                );
            vector::push_back(&mut acknowledgements, inner_vec);
            idx = idx + 1;
        };

        BatchAcknowledgement { acknowledgements: acknowledgements }
    }

    public fun encode_batch_ack(ack: &BatchAcknowledgement): vector<u8> {
        let mut buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        let ack_arr_len = vector::length(&ack.acknowledgements);
        ethabi::encode_uint<u64>(&mut buf, ack_arr_len);
        if (ack_arr_len < 2) {
            if (ack_arr_len == 1) {
                ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
                ethabi::encode_vector!<u8>(
                    &mut buf,
                    vector::borrow(&ack.acknowledgements, 0),
                    |some_variable, data| {
                        ethabi::encode_uint<u8>(some_variable, *data);
                    }
                );
                return buf
            };
            return buf
        };

        let initial_stage = 0x20 * (ack_arr_len as u32);
        let mut idx = 1;
        let mut prev_val = initial_stage;
        ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
        while (idx < ack_arr_len) {
            let prev_length = vector::length(
                vector::borrow(&ack.acknowledgements, idx - 1)
            );
            ethabi::encode_uint<u32>(&mut buf, prev_val
                + 0x20 * (prev_length + 1 as u32));
            prev_val = prev_val + 0x20 * (prev_length + 1 as u32);
            idx = idx + 1;
        };
        idx = 0;
        while (idx < ack_arr_len) {
            ethabi::encode_vector!<u8>(
                &mut buf,
                vector::borrow(&ack.acknowledgements, idx),
                |some_variable, data| {
                    ethabi::encode_uint<u8>(some_variable, *data);
                }
            );
            idx = idx + 1;
        };

        buf
    }

    public fun decode_batch_packet(buf: vector<u8>): BatchPacket {
        let mut index = 0x40;
        let main_arr_length = ethabi::decode_uint(&buf, &mut index);
        index = index + (0x20 * main_arr_length as u64);

        let mut idx = 0;
        let mut syscall_packets = vector::empty();
        while (idx < main_arr_length) {
            let inner_vec =
                ethabi::decode_vector!<u8>(
                    &buf,
                    &mut index,
                    |buf, index| {
                        (ethabi::decode_uint(buf, index) as u8)
                    }
                );
            vector::push_back(&mut syscall_packets, inner_vec);
            idx = idx + 1;
        };

        BatchPacket { syscall_packets: syscall_packets }
    }

    public fun encode_batch_packet(ack: &BatchPacket): vector<u8> {
        let mut buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        let ack_arr_len = vector::length(&ack.syscall_packets);
        ethabi::encode_uint<u64>(&mut buf, ack_arr_len);
        if (ack_arr_len < 2) {
            if (ack_arr_len == 1) {
                ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
                ethabi::encode_vector!<u8>(
                    &mut buf,
                    vector::borrow(&ack.syscall_packets, 0),
                    |some_variable, data| {
                        ethabi::encode_uint<u8>(some_variable, *data);
                    }
                );
                return buf
            };
            return buf
        };

        let initial_stage = 0x20 * (ack_arr_len as u32);
        let mut idx = 1;
        let mut prev_val = initial_stage;
        ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
        while (idx < ack_arr_len) {
            let prev_length = vector::length(
                vector::borrow(&ack.syscall_packets, idx - 1)
            );
            ethabi::encode_uint<u32>(&mut buf, prev_val
                + 0x20 * (prev_length + 1 as u32));
            prev_val = prev_val + 0x20 * (prev_length + 1 as u32);
            idx = idx + 1;
        };
        idx = 0;
        while (idx < ack_arr_len) {
            ethabi::encode_vector!<u8>(
                &mut buf,
                vector::borrow(&ack.syscall_packets, idx),
                |some_variable, data| {
                    ethabi::encode_uint<u8>(some_variable, *data);
                }
            );
            idx = idx + 1;
        };

        buf
    }

    public fun decode_syscall(buf: vector<u8>): SyscallPacket {
        let mut index = 0x20;
        let version = ethabi::decode_uint(&buf, &mut index);
        let mut index_syscall = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;

        let packet =
            ethabi::decode_vector!<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );

        SyscallPacket {
            version: (version as u8),
            index: (index_syscall as u8),
            packet: packet
        }
    }

    public fun encode_forward(packet: &ForwardPacket): vector<u8> {
        let mut buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u32>(&mut buf, packet.channel_id);
        ethabi::encode_uint<u64>(&mut buf, packet.timeout_height);
        ethabi::encode_uint<u64>(&mut buf, packet.timeout_timestamp);
        ethabi::encode_uint<u8>(&mut buf, 0x80);
        ethabi::encode_bytes(&mut buf, &packet.syscall_packet);
        buf
    }

    public fun decode_forward(buf: vector<u8>): ForwardPacket {
        let mut index = 0x20;
        let channel_id = ethabi::decode_uint(&buf, &mut index);
        let timeout_height = ethabi::decode_uint(&buf, &mut index);
        let timeout_timestamp = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let syscall_packet = ethabi::decode_bytes(&buf, &mut index);

        ForwardPacket {
            channel_id: (channel_id as u32),
            timeout_height: (timeout_height as u64),
            timeout_timestamp: (timeout_timestamp as u64),
            syscall_packet: syscall_packet
        }
    }

    public fun decode_multiplex(buf: vector<u8>): MultiplexPacket {
        let mut index = 0x40;
        let eureka = ethabi::decode_uint(&buf, &mut index) == 1;
        index = index + 0x20 * 2;
        let sender = ethabi::decode_bytes(&buf, &mut index);
        let contract_address = ethabi::decode_bytes(&buf, &mut index);
        let contract_calldata = ethabi::decode_bytes(&buf, &mut index);

        MultiplexPacket {
            sender: sender,
            eureka: eureka,
            contract_address: contract_address,
            contract_calldata: contract_calldata
        }
    }

    public fun encode_multiplex_sender_and_calldata(
        sender: vector<u8>, contract_calldata: vector<u8>
    ): vector<u8> {
        let mut buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, 0x40);
        let length_of_first = vector::length(&sender);
        ethabi::encode_uint<u64>(&mut buf, ((length_of_first / 32) * 0x20) + 0x80);
        ethabi::encode_bytes(&mut buf, &sender);
        ethabi::encode_bytes(&mut buf, &contract_calldata);
        buf
    }

    public fun decode_fungible_asset_transfer(buf: vector<u8>): FungibleAssetTransferPacket {
        let mut index = 0x80;
        let sent_token_prefix = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x40;
        let sent_amount = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let ask_amount = ethabi::decode_uint(&buf, &mut index);
        let only_maker = (ethabi::decode_uint(&buf, &mut index) == 1);
        let sender = ethabi::decode_bytes(&buf, &mut index);
        let receiver = ethabi::decode_bytes(&buf, &mut index);
        let sent_token = ethabi::decode_bytes(&buf, &mut index);
        let sent_symbol = ethabi::decode_string(&buf, &mut index);
        let sent_name = ethabi::decode_string(&buf, &mut index);
        let ask_token = ethabi::decode_bytes(&buf, &mut index);

        FungibleAssetTransferPacket {
            sender: sender,
            receiver: receiver,
            sent_token: sent_token,
            sent_token_prefix: sent_token_prefix,
            sent_symbol: sent_symbol,
            sent_name: sent_name,
            sent_amount: (sent_amount as u64),
            ask_token: ask_token,
            ask_amount: (ask_amount as u64),
            only_maker: only_maker
        }
    }

    public fun encode_packet(packet: &ZkgmPacket): vector<u8> {
        let mut buf = vector::empty<u8>();
        ethabi::encode_uint<u8>(&mut buf, 0x20);
        ethabi::encode_uint<u8>(&mut buf, 0x60);
        ethabi::encode_uint<u256>(&mut buf, packet.path);

        let version_offset = 0x20 * 4;
        ethabi::encode_uint<u32>(
            &mut buf,
            version_offset + ((vector::length(&packet.salt) * 0x20) as u32)
        );

        ethabi::encode_vector!<u8>(
            &mut buf,
            &packet.salt,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        ethabi::encode_vector!<u8>(
            &mut buf,
            &packet.syscall,
            |some_variable, data| {
                ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun decode_packet(buf: vector<u8>): ZkgmPacket {
        let mut index = 0x40;
        let packet_path = ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let salt =
            ethabi::decode_vector!<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );
        let syscall =
            ethabi::decode_vector!<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_uint(buf, index) as u8)
                }
            );

        ZkgmPacket { salt: salt, path: packet_path, syscall: syscall }
    }
    
    public fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }


    public fun starts_with(s: String, prefix: String): bool {
        let s_len = string::length(&s);
        let prefix_len = string::length(&prefix);

        if (prefix_len > s_len) {
            return false
        };

        // Convert String to vector<u8>
        let s_bytes: vector<u8> = *string::bytes(&s);
        let prefix_bytes: vector<u8> = *string::bytes(&prefix);

        let mut i = 0;
        while (i < prefix_len) {
            if (vector::borrow(&s_bytes, i) != vector::borrow(&prefix_bytes, i)) {
                return false
            };
            i = i + 1;
        };
        true
    }

    public entry fun channel_open_init(
        ibc_store: &mut ibc::IBCStore,
        counterparty_port_id: vector<u8>,
        connection_id: u32, 
        version: String
    ) {
        ibc::channel_open_init(
            ibc_store,
            utf8(b"&get_signer()"),
            counterparty_port_id,
            connection_id,
            version
        );

        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

    }
    public entry fun channel_open_try(
        ibc_store: &mut ibc::IBCStore,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_COUNTERPARTY_PROTOCOL_VERSION
        };

        ibc::channel_open_try(
            ibc_store,
            utf8(b"&get_signer()"),
            connection_id,
            counterparty_channel_id,
            counterparty_port_id,
            version,
            counterparty_version,
            proof_init,
            proof_height
        );
    }

    public entry fun channel_open_ack(
        ibc_store: &mut ibc::IBCStore,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        // Store the channel_id
        ibc::channel_open_ack(
            ibc_store,
            utf8(b"&get_signer()"),
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height
        );
        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_COUNTERPARTY_PROTOCOL_VERSION
        };
    }

    public entry fun channel_open_confirm(
        ibc_store: &mut ibc::IBCStore,
        channel_id: u32, 
        proof_ack: vector<u8>, 
        proof_height: u64
    )  {
        ibc::channel_open_confirm(
            ibc_store,
            utf8(b"&get_signer()"),
            channel_id,
            proof_ack,
            proof_height
        );
    }

    public entry fun channel_close_init(_channel_id: u32) {
        abort E_UNSTOPPABLE
    }

    public entry fun channel_close_confirm(_channel_id: u32) {
        abort E_UNSTOPPABLE
    }

    /// Function to trim a prefix from the string if it starts with that prefix
    public fun trim_prefix(s: String, prefix: String): String {
        // Check if the string starts with the prefix
        if (!starts_with(s, prefix)) {
            return s // If the prefix doesn't match, return the string as-is
        };

        // Get the lengths
        let prefix_len = string::length(&prefix);
        let s_len = string::length(&s);

        // Get the bytes of the string and create a new trimmed vector
        let s_bytes = string::bytes(&s);
        let mut trimmed_bytes = vector::empty<u8>();

        // Manually copy elements starting from prefix_len to s_len
        let mut i = prefix_len;
        while (i < s_len) {
            vector::push_back(&mut trimmed_bytes, s_bytes[i]);
            i = i + 1;
        };


        // Convert the trimmed vector back to a string
        string::utf8(trimmed_bytes)
    }


    public fun recv_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        clock: &clock::Clock,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        proof: vector<u8>,
        proof_height: u64,
        ctx: &mut TxContext
    ) {
        let mut packets: vector<Packet> = vector::empty();
        let mut i = 0;
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
        ibc::recv_packet(
            ibc_store,
            clock,
            packets,
            proof,
            proof_height,
            vector[1]
        );
        while (i < vector::length(&packets)) {
            let packet = *vector::borrow(&packets, i);

        };
    }

        #[test]
    fun test_zkgm_encode_decode() {
        let output =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000032dcd60000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        let zkgm_data = ZkgmPacket { salt: b"helloo", path: 3333334, syscall: b"hellloo" };

        let zkgm_bytes = encode_packet(&zkgm_data);
        assert!(zkgm_bytes == output, 0);

        let zkgm_data_decoded = decode_packet(zkgm_bytes);
        assert!(zkgm_data_decoded.salt == b"helloo", 1);
        assert!(zkgm_data_decoded.path == 3333334, 2);
        assert!(zkgm_data_decoded.syscall == b"hellloo", 3);
    }

    #[test]
    fun test_decode_syscall() {
        let output =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000007100000000000000000000000000000000000000000000000000000000000000f40000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";

        let syscall_data_decoded = decode_syscall(output);
        assert!(syscall_data_decoded.version == 113, 1);
        assert!(syscall_data_decoded.index == 244, 2);
        assert!(syscall_data_decoded.packet == b"hellloo", 3);
    }

    #[test]
    fun test_encode_decode_ack() {
        let output =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000007157f2addb00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        let ack_data = Acknowledgement { tag: 7788909223344, inner_ack: b"hellloo" };

        let ack_bytes = encode_ack(&ack_data);
        assert!(ack_bytes == output, 0);

        let ack_data_decoded = decode_ack(ack_bytes);
        assert!(ack_data_decoded.tag == 7788909223344, 1);
        assert!(ack_data_decoded.inner_ack == b"hellloo", 3);
    }

    #[test]
    fun test_encode_decode_asset_transfer_ack() {
        let output =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000007157f2addb00000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000700000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000006f";
        let ack_data = AssetTransferAcknowledgement {
            fill_type: 7788909223344,
            market_maker: b"hellloo"
        };

        let ack_bytes = encode_asset_transfer_ack(&ack_data);
        assert!(ack_bytes == output, 0);

        let ack_data_decoded = decode_asset_transfer_ack(ack_bytes);
        assert!(ack_data_decoded.fill_type == 7788909223344, 1);
        assert!(ack_data_decoded.market_maker == b"hellloo", 3);
    }

    #[test]
    fun test_encode_decode_batch_ack() {
        // ---------------- TEST 1 ----------------
        let output =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006900000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065";
        let mut outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"hello");
        vector::push_back(&mut outer_arr, b"hi");
        vector::push_back(&mut outer_arr, b"hehe");
        let ack_data = BatchAcknowledgement { acknowledgements: outer_arr };
        let ack_bytes = encode_batch_ack(&ack_data);
        assert!(ack_bytes == output, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 3, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"hi", 3);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 2) == b"hehe", 4);

        // ---------------- TEST 2 ----------------
        let output2 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000069";
        let mut outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"hello");
        vector::push_back(&mut outer_arr, b"hi");
        let ack_data2 = BatchAcknowledgement { acknowledgements: outer_arr };
        let ack_bytes2 = encode_batch_ack(&ack_data2);
        assert!(ack_bytes2 == output2, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes2);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 2, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"hi", 3);

        // ---------------- TEST 3 ----------------
        let output3 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000032000000000000000000000000000000000000000000000000000000000000003400000000000000000000000000000000000000000000000000000000000000360000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000003c000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000007300000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();
        let mut idx = 0;
        vector::push_back(&mut outer_arr, b"xdddd");
        vector::push_back(&mut outer_arr, b"test");
        while (idx < 10) {
            vector::push_back(&mut outer_arr, b"");
            idx = idx + 1;
        };

        let ack_data3 = BatchAcknowledgement { acknowledgements: outer_arr };
        let ack_bytes3 = encode_batch_ack(&ack_data3);
        assert!(ack_bytes3 == output3, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes3);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 12, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"xdddd", 2);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 1) == b"test", 3);

        // ---------------- TEST 4 ----------------
        let output4 =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064";
        let mut outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"xdddd");

        let ack_data4 = BatchAcknowledgement { acknowledgements: outer_arr };
        let ack_bytes4 = encode_batch_ack(&ack_data4);
        assert!(ack_bytes4 == output4, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes4);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 1, 1);
        assert!(*vector::borrow(&ack_data_decoded.acknowledgements, 0) == b"xdddd", 2);

        // ---------------- TEST 5 ----------------
        let output5 =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();

        let ack_data5 = BatchAcknowledgement { acknowledgements: outer_arr };
        let ack_bytes5 = encode_batch_ack(&ack_data5);
        assert!(ack_bytes5 == output5, 0);
        let ack_data_decoded = decode_batch_ack(ack_bytes5);
        assert!(vector::length(&ack_data_decoded.acknowledgements) == 0, 1);

    }

    #[test]
    fun test_encode_decode_batch_packet() {
        // ---------------- TEST 1 ----------------
        let output =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006900000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000068000000000000000000000000000000000000000000000000000000000000006500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065";
        let mut outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"hello");
        vector::push_back(&mut outer_arr, b"hi");
        vector::push_back(&mut outer_arr, b"hehe");
        let ack_data = BatchPacket { syscall_packets: outer_arr };
        let ack_bytes = encode_batch_packet(&ack_data);
        assert!(ack_bytes == output, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 3, 1);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 1) == b"hi", 3);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 2) == b"hehe", 4);

        // ---------------- TEST 2 ----------------
        let output2 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000006f000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000069";
        let mut outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"hello");
        vector::push_back(&mut outer_arr, b"hi");
        let ack_data2 = BatchPacket { syscall_packets: outer_arr };
        let ack_bytes2 = encode_batch_packet(&ack_data2);
        assert!(ack_bytes2 == output2, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes2);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 2, 1);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 0) == b"hello", 2);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 1) == b"hi", 3);

        // ---------------- TEST 3 ----------------
        let output3 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000032000000000000000000000000000000000000000000000000000000000000003400000000000000000000000000000000000000000000000000000000000000360000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000003c000000000000000000000000000000000000000000000000000000000000003e00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000065000000000000000000000000000000000000000000000000000000000000007300000000000000000000000000000000000000000000000000000000000000740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();
        let mut idx = 0;
        vector::push_back(&mut outer_arr, b"xdddd");
        vector::push_back(&mut outer_arr, b"test");
        while (idx < 10) {
            vector::push_back(&mut outer_arr, b"");
            idx = idx + 1;
        };

        let ack_data3 = BatchPacket { syscall_packets: outer_arr };
        let ack_bytes3 = encode_batch_packet(&ack_data3);
        assert!(ack_bytes3 == output3, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes3);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 12, 1);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 0) == b"xdddd", 2);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 1) == b"test", 3);

        // ---------------- TEST 4 ----------------
        let output4 =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000780000000000000000000000000000000000000000000000000000000000000064000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000640000000000000000000000000000000000000000000000000000000000000064";
        let mut outer_arr = vector::empty();
        vector::push_back(&mut outer_arr, b"xdddd");

        let ack_data4 = BatchPacket { syscall_packets: outer_arr };
        let ack_bytes4 = encode_batch_packet(&ack_data4);
        assert!(ack_bytes4 == output4, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes4);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 1, 1);
        assert!(*vector::borrow(&ack_data_decoded.syscall_packets, 0) == b"xdddd", 2);

        // ---------------- TEST 5 ----------------
        let output5 =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();

        let ack_data5 = BatchPacket { syscall_packets: outer_arr };
        let ack_bytes5 = encode_batch_packet(&ack_data5);
        assert!(ack_bytes5 == output5, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes5);
        assert!(vector::length(&ack_data_decoded.syscall_packets) == 0, 1);

    }

    #[test]
    fun test_encode_decode_forward_packet() {
        let output =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000005161500000000000000000000000000000000000000000000000000000000000056ce0000000000000000000000000000000000000000000000000000000000002b670000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000005a4578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c44617461000000000000";
        let forward_data = ForwardPacket {
            channel_id: 333333,
            timeout_height: 22222,
            timeout_timestamp: 11111,
            syscall_packet: b"ExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallData"
        };

        let ack_bytes = encode_forward(&forward_data);
        assert!(ack_bytes == output, 0);

        let forward_data_decoded = decode_forward(ack_bytes);
        assert!(forward_data_decoded.channel_id == forward_data.channel_id, 0);
        assert!(forward_data_decoded.timeout_height == forward_data.timeout_height, 1);
        assert!(
            forward_data_decoded.timeout_timestamp == forward_data.timeout_timestamp, 2
        );
        assert!(forward_data_decoded.syscall_packet == forward_data.syscall_packet, 3);
    }

    #[test]
    fun test_decode_multiplex() {
        let output =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000002a3078354233384461366137303163353638353435644366634230334663423837356635366265646443340000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000617468697369736d79616464726573737a6c756c7468697369736d79616464726573737a6c756c7468697369736d79616464726573737a6c756c7468697369736d79616464726573737a6c756c7468697369736d79616464726573737a6c756c626200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005a4578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c44617461000000000000";
        let multiplex_pack = MultiplexPacket {
            sender: b"0x5B38Da6a701c568545dCfcB03FcB875f56beddC4",
            eureka: true,
            contract_address: b"thisismyaddresszlulthisismyaddresszlulthisismyaddresszlulthisismyaddresszlulthisismyaddresszlulbb",
            contract_calldata: b"ExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallData"
        };

        let multiplex_decoded = decode_multiplex(output);
        assert!(multiplex_decoded.sender == multiplex_pack.sender, 0);
        assert!(multiplex_decoded.eureka == multiplex_pack.eureka, 1);
        assert!(multiplex_decoded.contract_address == multiplex_pack.contract_address, 2);
        assert!(
            multiplex_decoded.contract_calldata == multiplex_pack.contract_calldata, 3
        );
    }

    #[test]
    fun test_decode_fungible_asset_transfer_pack() {

        let output =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000007c37bdc730000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000007a1200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f307853656e646572416464726573730000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001130785265636569766572416464726573730000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012307853656e74546f6b656e4164647265737300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014535353535353535353535353535353594d424f4c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000155454545454545454545454546f6b656e204e616d6500000000000000000000000000000000000000000000000000000000000000000000000000000000000011307841736b546f6b656e41646472657373000000000000000000000000000000";
        let fatp = FungibleAssetTransferPacket {
            sender: b"0xSenderAddress",
            receiver: b"0xReceiverAddress",
            sent_token: b"0xSentTokenAddress",
            sent_token_prefix: 33344445555,
            sent_symbol: string::utf8(b"SSSSSSSSSSSSSSSYMBOL"),
            sent_name: string::utf8(b"TTTTTTTTTTTToken Name"),
            sent_amount: 1000000, // Example token amount
            ask_token: b"0xAskTokenAddress",
            ask_amount: 500000, // Example ask amount
            only_maker: false
        };

        let fatp_decoded = decode_fungible_asset_transfer(output);
        assert!(fatp.sender == fatp_decoded.sender, 0);
        assert!(fatp.receiver == fatp_decoded.receiver, 0);
        assert!(fatp.sent_token == fatp_decoded.sent_token, 0);
        assert!(fatp.sent_token_prefix == fatp_decoded.sent_token_prefix, 0);
        assert!(fatp.sent_symbol == fatp_decoded.sent_symbol, 0);
        assert!(fatp.sent_name == fatp_decoded.sent_name, 0);
        assert!(fatp.sent_amount == fatp_decoded.sent_amount, 0);
        assert!(fatp.ask_token == fatp_decoded.ask_token, 0);
        assert!(fatp.ask_amount == fatp_decoded.ask_amount, 0);
        assert!(fatp.only_maker == fatp_decoded.only_maker, 0);
    }

}