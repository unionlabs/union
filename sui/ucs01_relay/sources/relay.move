module ucs01::relay_app {
    use ibc::ibc;
    use ibc::packet::{Self, Packet};
    use ucs01::fungible_token::{Self, FUNGIBLE_TOKEN};
    use std::string::{Self, String, utf8};
    use sui::table::{Self, Table};
    use sui::bcs;
    use sui::clock;
    use sui::address::{to_string};
    use sui::event;
    use sui::coin::{Self, Coin, TreasuryCap, CoinMetadata};

        // Constants
    const ORDER_UNORDERED: u8 = 1;
    const VERSION: vector<u8> = b"ucs01-relay-1";
    const ACK_SUCCESS: u8 = 1;
    const ACK_FAILURE: u8 = 0;
    const ACK_LENGTH: u64 = 1;

    // Errors
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
    const E_INVALID_BYTES_ADDRESS: u64 = 1;
    const E_UNAUTHORIZED: u64 = 2;
    const E_INVALID_ACKNOWLEDGEMENT: u64 = 3;
    const E_INVALID_PROTOCOL_VERSION: u64 = 4;
    const E_INVALID_PROTOCOL_ORDERING: u64 = 5;
    const E_INVALID_COUNTERPARTY_PROTOCOL_VERSION: u64 = 6;
    const E_INVALID_AMOUNT: u64 = 7;
    const E_UNSTOPPABLE: u64 = 8;

    public struct Token has copy, drop, store {
        denom: String,
        amount: u64
    }

    public struct LocalToken has copy, drop, store {
        denom: address,
        amount: u64
    }

    public struct RelayPacket has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        tokens: vector<Token>,
        extension: String
    }

    public struct DenomToAddressPair has copy, drop, store {
        source_channel: u32,
        denom: String
    }

    public struct AddressToDenomPair has copy, drop, store {
        source_channel: u32,
        denom: address
    }

    public struct OutstandingPair has copy, drop, store {
        source_channel: u32,
        token: address
    }

    public struct RelayStore has key {
        id: UID,
        denom_to_address: Table<DenomToAddressPair, address>,
        address_to_denom: Table<AddressToDenomPair, String>,
        outstanding: Table<OutstandingPair, u64>,
        address_to_treasurycap: Table<address, TreasuryCap<FUNGIBLE_TOKEN>>,
        address_to_coin: Table<address, Coin<FUNGIBLE_TOKEN>>
    }

    // Events
    #[event]
    public struct DenomCreated has copy, drop, store {
        packet_sequence: u64,
        channel_id: u32,
        denom: String,
        token: address
    }

    #[event]
    public struct Received has copy, drop, store {
        packet_sequence: u64,
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }

    #[event]
    public struct Sent has copy, drop, store {
        packet_sequence: u64,
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }

    #[event]
    public struct Refunded has copy, drop, store {
        packet_sequence: u64,
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
            denom_to_address: table::new(ctx),
            address_to_denom: table::new(ctx),
            outstanding: table::new(ctx),
            address_to_treasurycap: table::new(ctx),
            address_to_coin: table::new(ctx)
        });
    }


    public fun get_denom_address(relay_store: &mut RelayStore, source_channel: u32, denom: String): address {
        let pair = DenomToAddressPair { source_channel, denom };

        let denom_addr = if (relay_store.denom_to_address.contains(pair)) {
            relay_store.denom_to_address.borrow(pair)
        } else {
            &@0x0
        };
        *denom_addr
    }

    fun get_treasury_cap_mut(relay_store: &mut RelayStore, denom_address: address): &mut TreasuryCap<FUNGIBLE_TOKEN> {
        relay_store.address_to_treasurycap.borrow_mut(denom_address)
    }

    fun get_coin_mut(relay_store: &mut RelayStore, denom_address: address): &mut Coin<FUNGIBLE_TOKEN> {
        relay_store.address_to_coin.borrow_mut(denom_address)
    }


    public entry fun insert_pair(
        relay_store: &mut RelayStore,
        denom_address: address,
        treasury_cap: TreasuryCap<FUNGIBLE_TOKEN>,
        coin: Coin<FUNGIBLE_TOKEN>
    ) {
        relay_store.address_to_treasurycap.add(denom_address, treasury_cap);
        relay_store.address_to_coin.add(denom_address, coin);
    }


    // View/Pure Functions
    public fun is_valid_version(version_bytes: vector<u8>): bool {
        version_bytes == VERSION
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
    public fun is_from_channel(channel_id: u32, denom: String): bool {
        let prefix = make_denom_prefix(channel_id);
        starts_with(denom, prefix)
    }

    public fun make_denom_prefix(channel_id: u32): String {
        
        let channel_id_bytes = bcs::to_bytes<u32>(&channel_id);
        let mut prefix = string::utf8(channel_id_bytes);
        string::append_utf8(&mut prefix, b"/");
        prefix
    }

    public fun make_foreign_denom(channel_id: u32, denom: String): String {
        let mut foreign_denom = make_denom_prefix(channel_id);
        string::append(&mut foreign_denom, denom);
        foreign_denom
    }

    public fun get_outstanding(relay_store: &mut RelayStore, source_channel: u32, token: address): u64 {
        let pair = OutstandingPair { source_channel, token };
        
        let outstanding = if (relay_store.outstanding.contains(pair)) {
            relay_store.outstanding.borrow(pair)
        } else {
            &0
        };
        *outstanding
    }

    public fun increase_outstanding(relay_store: &mut RelayStore, 
        source_channel: u32, token: address, amount: u64
    ) {
        let pair = OutstandingPair { source_channel, token };

        if (relay_store.outstanding.contains(pair)) {
            let val = relay_store.outstanding.borrow_mut(pair);
            *val = *val + amount;
        } else {
            relay_store.outstanding.add(pair, amount);
        };
    }

    public fun decrease_outstanding(
        relay_store: &mut RelayStore, 
        source_channel: u32,
        token: address,
        amount: u64
    ) {
        let pair = OutstandingPair { source_channel, token };
        let val = relay_store.outstanding.borrow_mut(pair);
        *val = *val - amount;
        // Abort if pair does not exist
    }

    public entry fun channel_open_init(
        ibc_store: &mut ibc::IBCStore,
        connection_id: u32, 
        ordering: u8, 
        version: vector<u8>
    ) {
        ibc::channel_open_init(
            ibc_store,
            utf8(b"&get_signer()"),
            connection_id,
            ordering,
            version
        );

        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

        if (ordering != ORDER_UNORDERED) {
            abort E_INVALID_PROTOCOL_ORDERING
        };
    }
    public entry fun chan_open_try(
        ibc_store: &mut ibc::IBCStore,
        channel_state: u8,
        channel_order: u8,
        connection_id: u32,
        counterparty_channel_id: u32,
        version: vector<u8>,
        counterparty_version: vector<u8>,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        ibc::channel_open_try(
            ibc_store,
            utf8(b"&get_signer()"),
            channel_state,
            channel_order,
            connection_id,
            counterparty_channel_id,
            version,
            counterparty_version,
            proof_init,
            proof_height
        );

        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

        if (channel_order != ORDER_UNORDERED) {
            abort E_INVALID_PROTOCOL_ORDERING
        };

        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_COUNTERPARTY_PROTOCOL_VERSION
        };
    }

    public entry fun channel_open_ack(
        ibc_store: &mut ibc::IBCStore,
        channel_id: u32,
        counterparty_version: vector<u8>,
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

    public fun encode_packet(packet: &RelayPacket): vector<u8> {
        // TODO: Fix here
        let buf = vector::empty<u8>();
        buf

        // // TODO(aeryz): document
        // // Offset of `packet.sender`
        // ethabi::encode_uint<u64>(&mut buf, 32 * 4);
        // // Offset of `packet.receiver`
        // ethabi::encode_uint<u64>(&mut buf, 32 * 6);
        // // Offset of `packet.tokens`
        // ethabi::encode_uint<u64>(&mut buf, 32 * 8);
        // // Offset of `packet.extension`. We temporarily write `0` here because
        // // `packet.tokens` contain arbitrary-length fields. Hence we can't calculate
        // // the offset at this point without recursing on the tokens.
        // ethabi::encode_uint<u64>(&mut buf, 0);

        // // bytes encoded `packet.sender`
        // ethabi::encode_vector<u8>(
        //     &mut buf,
        //     packet.sender,
        //     |some_variable, data| {
        //         ethabi::encode_u8(some_variable, data);
        //     }
        // );

        // // bytes encoded `packet.receiver`
        // ethabi::encode_vector<u8>(
        //     &mut buf,
        //     packet.receiver,
        //     |some_variable, data| {
        //         ethabi::encode_u8(some_variable, data);
        //     }
        // );

        // // length prefix of the tokens array
        // let num_tokens = vector::length(&packet.tokens);
        // ethabi::encode_uint<u64>(&mut buf, num_tokens);

        // let tokens_buf = vector::empty();
        // let i = 0;
        // let prev_len = 0;
        // while (i < num_tokens) {
        //     let token = vector::borrow(&packet.tokens, i);

        //     // TODO(aeryz): this should be 96 when fee is enabled
        //     // TODO(aeryz): handle fee
        //     /*
        //     ethabi::encode_uint<u64>(&mut tokens_buf, 96);
        //     ethabi::encode_uint<u64>(&mut tokens_buf, 0);
        //     */

        //     ethabi::encode_uint<u64>(&mut tokens_buf, 64);
        //     ethabi::encode_uint<u64>(&mut tokens_buf, token.amount);

        //     ethabi::encode_string(&mut tokens_buf, token.denom);

        //     i = i + 1;

        //     let cursor = 32 + ((num_tokens - 1) * 32);
        //     ethabi::encode_uint<u64>(&mut buf, cursor + prev_len);
        //     prev_len = prev_len + vector::length(&tokens_buf);
        //     i = i + 1;
        // };

        // vector::append(&mut buf, tokens_buf);

        // let offset_buf = vector::empty();
        // ethabi::encode_uint<u64>(&mut offset_buf, vector::length(&buf));

        // let i = 96;
        // while (i < 128) {
        //     let b = vector::borrow_mut(&mut buf, i);
        //     *b = *vector::borrow(&offset_buf, i - 96);
        //     i = i + 1;
        // };
        // ethabi::encode_string(&mut buf, packet.extension);

        // buf
    }


    public fun decode_packet(buf: vector<u8>): RelayPacket {
        // TODO: Fix here
        RelayPacket {
            sender: b"",
            receiver: b"", 
            tokens: vector::empty(),
            extension: utf8(b"")
        }
        // let index = 128;

        // // let _unknown_data_32 = ethabi::decode_uint(buf, &mut index);
        // // let _unknown_data_32 = ethabi::decode_uint(buf, &mut index);
        // // let _unknown_data_32 = ethabi::decode_uint(buf, &mut index);
        // // let _unknown_data_32 = ethabi::decode_uint(buf, &mut index);

        // // Decoding sender address
        // let sender =
        //     ethabi::decode_vector<u8>(
        //         buf,
        //         &mut index,
        //         |buf, index| {
        //             (ethabi::decode_u8(buf, index) as u8)
        //         }
        //     );

        // let receiver =
        //     ethabi::decode_vector<u8>(
        //         buf,
        //         &mut index,
        //         |buf, index| {
        //             (ethabi::decode_u8(buf, index) as u8)
        //         }
        //     );

        // // let receiver = from_bcs::to_address(receiver_vec);

        // // Decoding the number of tokens
        // let num_tokens = (ethabi::decode_uint(buf, &mut index) as u64);

        // index = index + num_tokens * 32;

        // let tokens = vector::empty<Token>();
        // // Decoding the token starting point and sequence
        // let i = 0;
        // while (i < num_tokens) {
        //     // dynamic data prefix
        //     index = index + 32;

        //     let amount = ethabi::decode_uint(buf, &mut index);
        //     // let _fee = ethabi::decode_uint(buf, &mut index);
        //     let denom = ethabi::decode_string(buf, &mut index);

        //     let token = Token { amount: (amount as u64), denom: denom };
        //     vector::push_back(&mut tokens, token);

        //     i = i + 1;
        // };

        // // Decoding the extension string
        // let extension = ethabi::decode_string(buf, &mut index);

        // // Returning the decoded RelayPacket
        // RelayPacket {
        //     sender: sender,
        //     receiver: receiver,
        //     tokens: tokens,
        //     extension: extension
        // }
    }

    public fun hex_to_bytes(hex_str: String): vector<u8> {
        let hex_str_bytes = string::bytes(&hex_str);
        let mut byte_vector = vector::empty<u8>();

        let mut i = 0;
        while (i < vector::length(hex_str_bytes)) {
            let high_nibble = char_to_nibble(*vector::borrow(hex_str_bytes, i));
            let low_nibble = char_to_nibble(*vector::borrow(hex_str_bytes, i + 1));

            let byte = (high_nibble << 4) | low_nibble;
            vector::push_back(&mut byte_vector, byte);

            i = i + 2;
        };

        byte_vector
    }

    public fun char_to_nibble(char_byte: u8): u8 {
        if (char_byte >= 0x30 && char_byte <= 0x39) {
            // '0' to '9'
            char_byte - 0x30
        } else if (char_byte >= 0x41 && char_byte <= 0x46) {
            // 'A' to 'F'
            char_byte - 0x41 + 10
        } else if (char_byte >= 0x61 && char_byte <= 0x66) {
            // 'a' to 'f'
            char_byte - 0x61 + 10
        } else {
            abort 1;
            0
        }
    }

    fun refund_tokens(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sequence: u64,
        channel_id: u32, 
        packet: &RelayPacket,
        ctx: &mut TxContext
    ) {
        let receiver = packet.receiver;

        let user_to_refund = bcs::new(packet.sender).peel_address(); 

        let packet_tokens_length = vector::length(&packet.tokens);
        let mut i = 0;
        while (i < packet_tokens_length) {
            let token_from_vec = packet.tokens[i];
            let mut denom_address = get_denom_address(
                relay_store, 
            channel_id, 
                token_from_vec.denom
            );

            if (denom_address != @0x0) {
                let treasury_cap = get_treasury_cap_mut(relay_store, denom_address);
                ucs01::fungible_token::mint(
                    treasury_cap,
                    token_from_vec.amount,
                    user_to_refund,
                    ctx
                );
            } else {
                let mut token_denom = token_from_vec.denom;

                if (starts_with(token_from_vec.denom, string::utf8(b"@"))) {
                    token_denom = string::sub_string(&token_denom, 1, 65);
                };
                denom_address = bcs::new(hex_to_bytes(token_denom)).peel_address();
                let coin = get_coin_mut(relay_store, denom_address);

                decrease_outstanding(relay_store, channel_id, denom_address, token_from_vec.amount);
                // TODO: implement here, need to transfer
                // primary_fungible_store::transfer(
                //     &get_signer(),
                //     token,
                //     @zero_account,
                //     token_from_vec.amount
                // );
            };

            // Emit a Refunded event
            event::emit(
                Refunded {
                    packet_sequence: sequence,
                    channel_id: channel_id,
                    sender: packet.sender,
                    receiver: receiver,
                    denom: token_from_vec.denom,
                    token: denom_address,
                    amount: token_from_vec.amount
                }
            );

            i = i + 1;
        }
    }

    public entry fun timeout_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet_sequence: u64,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64,
        next_sequence_receive: u64,
        ctx: &mut TxContext
    ) {
        // Decode the packet data
        let relay_packet = decode_packet(packet_data);

        // Call the refund_tokens function to refund the sender
        refund_tokens(
            ibc_store, 
            relay_store, 
            packet_sequence, 
            packet_source_channel,
            &relay_packet,
            ctx
        );

        let packet =
            packet::new(
                packet_sequence,
                packet_source_channel,
                packet_destination_channel,
                packet_data,
                packet_timeout_height,
                packet_timeout_timestamp
            );

        ibc::timeout_packet(
            ibc_store,
            packet,
            proof,
            proof_height,
            next_sequence_receive
        );
    }

    public entry fun recv_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        clock: &clock::Clock,
        packet_sequences: vector<u64>,
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
        while (i < vector::length(&packet_sequences)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_sequences, i),
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
            on_recv_packet_processing(ibc_store, relay_store, packet, ctx);
        }
    }


    public fun on_recv_packet_processing(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet, // representing the IBC Packet,
        ctx: &mut TxContext
    ) {
        // Decode the RelayPacket from the IBC packet data
        let packet = decode_packet(*packet::data(&ibc_packet));
        let source_channel = packet::source_channel(&ibc_packet);
        let destination_channel = packet::destination_channel(&ibc_packet);

        // Create the denomination prefix based on source port and channel
        let prefix = make_denom_prefix(source_channel);

        // Get the receiver's address from the packet
        let receiver = bcs::new(packet.receiver).peel_address();

        let mut i = 0;
        let packet_tokens_length = vector::length(&packet.tokens);

        while (i < packet_tokens_length) {
            let token = vector::borrow(&packet.tokens, i);

            if (token.amount == 0) {
                abort E_INVALID_AMOUNT // Abort if the amount is 0
            };

            // Create the denomination slice to check the prefix
            let denom_slice = token.denom;
            let mut denom_address = @0x0;

            // Check if the denomination has the expected prefix (originated from this chain)
            if (starts_with(denom_slice, prefix)) {
                // Token originated from this chain, we need to unescrow the amount

                // Trim the prefix from the denom to get the actual denom
                let mut trimmed_denom = trim_prefix(denom_slice, prefix);

                // REVIEW(aeryz): maybe we should just avoid putting '@' in the first place when sending
                if (starts_with(trimmed_denom, string::utf8(b"@"))) {
                    trimmed_denom = string::sub_string(&trimmed_denom, 1, 65);
                };
                denom_address = bcs::new(hex_to_bytes(trimmed_denom)).peel_address();

                // Decrease the outstanding amount of the token
                decrease_outstanding(relay_store, source_channel, denom_address, token.amount);

                let coin = get_coin_mut(relay_store, denom_address);
                ucs01::fungible_token::transfer_with_split(coin, receiver, token.amount, ctx);

            } else {

                // Token originated from the counterparty chain, we need to mint the amount

                // Construct the foreign denomination using the source and destination channels
                let denom = make_foreign_denom(destination_channel, token.denom);

                // Create a DenomToAddressPair for the foreign denomination
                let pair = DenomToAddressPair {
                    source_channel: source_channel,
                    denom: denom
                };

                let denom_address = if (relay_store.denom_to_address.contains(pair)) {
                    *relay_store.denom_to_address.borrow(pair)
                } else {
                    @0x0
                };

                if (denom_address == @0x0) {
                    // TODO: Here we will raise event for token creation
                    // ucs01::fa_coin::initialize(
                    //     &get_signer(),
                    //     string::utf8(b""),
                    //     string::utf8(b""),
                    //     18,
                    //     string::utf8(b""),
                    //     string::utf8(b""),
                    //     *string::bytes(&denom)
                    // );
                    // TODO: i think also we need to handle that addr_to_denom somehow?
                    // denom_address = ucs01::fa_coin::get_metadata_address(
                    //     *string::bytes(&denom)
                    // );

                    // let pair = DenomToAddressPair {
                    //     source_channel: source_channel,
                    //     denom: denom
                    // };
                    // smart_table::upsert(&mut store.denom_to_address, pair, denom_address);

                    // // Also update the reverse mapping (address -> denom)
                    // let pair = AddressToDenomPair {
                    //     source_channel: destination_channel,
                    //     denom: denom_address
                    // };
                    // smart_table::upsert(&mut store.address_to_denom, pair, denom);

                    // Emit the DenomCreated event
                    event::emit(
                        DenomCreated {
                            packet_sequence: packet::sequence(&ibc_packet),
                            channel_id: source_channel,
                            denom: denom,
                            token: denom_address
                        }
                    );
                };

                // Mint tokens to the receiver's account
                let treasury_cap = get_treasury_cap_mut(relay_store, denom_address);
                ucs01::fungible_token::mint(treasury_cap, token.amount, receiver, ctx)

            };

            // Emit the Received event
            event::emit(
                Received {
                    packet_sequence: packet::sequence(&ibc_packet),
                    channel_id: destination_channel,
                    sender: packet.sender,
                    receiver: packet.receiver,
                    denom: token.denom,
                    token: denom_address,
                    amount: token.amount
                }
            );

            i = i + 1;
        };
    }

    public entry fun acknowledge_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet_sequences: vector<u64>,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
        proof: vector<u8>,
        proof_height: u64,
        ctx: &mut TxContext
    ) {
        let mut packets: vector<Packet> = vector::empty();
        let mut i = 0;
        while (i < vector::length(&packet_sequences)) {
            vector::push_back(
                &mut packets,
                packet::new(
                    *vector::borrow(&packet_sequences, i),
                    *vector::borrow(&packet_source_channels, i),
                    *vector::borrow(&packet_destination_channels, i),
                    *vector::borrow(&packet_datas, i),
                    *vector::borrow(&packet_timeout_heights, i),
                    *vector::borrow(&packet_timeout_timestamps, i)
                )
            );
            i = i + 1;
        };

        ibc::acknowledge_packet(
            ibc_store,
            packets,
            acknowledgements,
            proof,
            proof_height
        );

        i = 0;
        while (i < vector::length(&acknowledgements)) {
            let acknowledgement = *vector::borrow(&acknowledgements, i);
            let packet = *vector::borrow(&packets, i);

            if (vector::length(&acknowledgement) != ACK_LENGTH
                || (
                    *vector::borrow(&acknowledgement, 0) != ACK_FAILURE
                        && *vector::borrow(&acknowledgement, 0) != ACK_SUCCESS
                )) {
                abort E_INVALID_ACKNOWLEDGEMENT
            };

            if (*vector::borrow(&acknowledgement, 0) == ACK_FAILURE) {
                let relay_packet = decode_packet(*packet::data(&packet));
                refund_tokens(
                    ibc_store,
                    relay_store,
                    packet::sequence(&packet),
                    packet::source_channel(&packet),
                    &relay_packet,
                    ctx
                );
            };

            i = i + 1;
        };
    }

    public entry fun send(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        source_channel: u32,
        mut coins: vector<Coin<FUNGIBLE_TOKEN>>,
        receiver: vector<u8>,
        denom_list: vector<address>,
        amount_list: vector<u64>,
        extension: String,
        timeout_height: u64,
        timeout_timestamp: u64,
        ctx: &mut TxContext
    )  {
        let sender = tx_context::sender(ctx);
        let num_tokens = vector::length(&denom_list);

        if (vector::length(&amount_list) != num_tokens) {
            abort E_INVALID_BYTES_ADDRESS
        };

        let mut normalized_tokens: vector<Token> = vector::empty<Token>();

        let mut i = num_tokens-1;
        while (i >= 0) {
            let local_token_denom = *vector::borrow(&denom_list, i);
            let local_token_amount = *vector::borrow(&amount_list, i);
            // let coin = *vector::borrow(&coins, i);
            let coin = coins.pop_back();
            let token_address =
                send_token(
                    relay_store,
                    source_channel,
                    local_token_denom,
                    coin,
                    local_token_amount,
                    ctx
                );

            // Create a normalized Token struct and push to the vector
            let normalized_token = Token {
                denom: token_address,
                amount: local_token_amount
            };
            vector::push_back(&mut normalized_tokens, normalized_token);
            i = i - 1;
        };

        vector::destroy_empty(coins);

        let packet: RelayPacket = RelayPacket {
            sender: bcs::to_bytes(&sender),
            receiver,
            tokens: normalized_tokens,
            extension
        };

        let packet_sequence =
            ibc::send_packet(
                ibc_store,
                source_channel,
                timeout_height,
                timeout_timestamp,
                encode_packet(&packet)
            );

        let mut i = 0;
        while (i < num_tokens) {
            let local_token_denom = *vector::borrow(&denom_list, i);
            let local_token_amount = *vector::borrow(&amount_list, i);
            let normalizedToken = *vector::borrow(&normalized_tokens, i);

            event::emit(
                Sent {
                    packet_sequence: packet_sequence,
                    channel_id: source_channel,
                    sender: bcs::to_bytes(&sender),
                    receiver: receiver,
                    denom: normalizedToken.denom,
                    token: local_token_denom,
                    amount: local_token_amount
                }
            );
            i = i + 1;
        }
    }

    public fun send_token(
        relay_store: &mut RelayStore,
        source_channel: u32,
        denom: address,
        coin: Coin<FUNGIBLE_TOKEN>,
        amount: u64,
    ): String {
        let pair = AddressToDenomPair { source_channel, denom };

        let mut token_address = if (relay_store.address_to_denom.contains(pair)) {
            relay_store.address_to_denom.borrow(pair)
        } else {
            &string::utf8(b"")
        };

        let treasury_cap = relay_store.address_to_treasurycap.borrow_mut(denom);

        if (!string::is_empty(token_address)) {
            ucs01::fungible_token::burn(treasury_cap, coin);
        } else {
            let self_coin = get_coin_mut(relay_store, denom);
            ucs01::fungible_token::join(self_coin, coin);

            increase_outstanding(relay_store, source_channel, denom, amount);
            token_address = &to_string(denom);
        };
        *token_address
    }
}