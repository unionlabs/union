module ibc::relay_app {
    use ibc::ibc;
    use ibc::helpers;
    use ibc::packet::{Packet};
    use ibc::ibc;
    use ibc::dispatcher;
    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};
    use aptos_std::copyable_any;
    use std::event;
    use aptos_framework::function_info;

    use std::string::{Self, String};
    use std::string_utils;
    use std::from_bcs;
    use std::bcs;
    use aptos_framework::fungible_asset::{Metadata};
    use aptos_framework::signer;
    use aptos_std::smart_table::{Self, SmartTable};
    use std::vector;
    use ucs01::ethabi;

    // Constants
    const ORDER_UNORDERED: u8 = 1;
    const VERSION: vector<u8> = b"ucs01-relay-1";
    const ACK_SUCCESS: vector<u8> = b"1";
    const ACK_FAILURE: u8 = 0;
    const ACK_LENGTH: u64 = 1;
    use std::option;

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

    struct UcsRelayProof has drop, store, key {}

    public(friend) fun new_ucs_relay_proof(): UcsRelayProof {
        UcsRelayProof {}
    }

    struct Token has copy, drop, store {
        denom: String,
        amount: u64
    }

    struct LocalToken has copy, drop, store {
        denom: address,
        amount: u64
    }

    struct RelayPacket has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        tokens: vector<Token>,
        extension: String
    }

    struct DenomToAddressPair has copy, drop, store {
        source_channel: u32,
        denom: String
    }

    struct AddressToDenomPair has copy, drop, store {
        source_channel: u32,
        denom: address
    }

    struct OutstandingPair has copy, drop, store {
        source_channel: u32,
        token: address
    }

    struct RelayStore has key {
        denom_to_address: SmartTable<DenomToAddressPair, address>,
        address_to_denom: SmartTable<AddressToDenomPair, String>,
        outstanding: SmartTable<OutstandingPair, u64>
    }

    struct SignerRef has key {
        self_ref: object::ExtendRef,
        self_address: address
    }

    // Events
    #[event]
    struct DenomCreated has copy, drop, store {
        packet_sequence: u64,
        channel_id: u32,
        denom: String,
        token: address
    }

    #[event]
    struct Received has copy, drop, store {
        packet_sequence: u64,
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }

    #[event]
    struct Sent has copy, drop, store {
        packet_sequence: u64,
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
    }

    #[event]
    struct Refunded has copy, drop, store {
        packet_sequence: u64,
        channel_id: u32,
        sender: vector<u8>,
        receiver: vector<u8>,
        denom: String,
        token: address,
        amount: u64
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

        let i = 0;
        while (i < prefix_len) {
            if (vector::borrow(&s_bytes, i) != vector::borrow(&prefix_bytes, i)) {
                return false
            };
            i = i + 1;
        };
        true
    }

    public fun get_metadata(asset_addr: address): Object<Metadata> {
        object::address_to_object<Metadata>(asset_addr)
    }

    public fun is_from_channel(channel_id: u32, denom: String): bool {
        let prefix = make_denom_prefix(channel_id);
        starts_with(denom, prefix)
    }

    public fun make_denom_prefix(channel_id: u32): String {
        let channel_id_bytes = bcs::to_bytes<u32>(&channel_id);
        let prefix = string::utf8(channel_id_bytes);
        string::append_utf8(&mut prefix, b"/");
        prefix
    }

    public fun make_foreign_denom(channel_id: u32, denom: String): String {
        let foreign_denom = make_denom_prefix(channel_id);
        string::append(&mut foreign_denom, denom);
        foreign_denom
    }

    public fun get_denom_address(source_channel: u32, denom: String): address acquires RelayStore {
        let store = borrow_global<RelayStore>(get_vault_addr());
        let pair = DenomToAddressPair { source_channel, denom };
        *smart_table::borrow_with_default(&store.denom_to_address, pair, &@0x0)
    }

    public fun get_outstanding(source_channel: u32, token: address): u64 acquires RelayStore {
        let store = borrow_global<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        *smart_table::borrow_with_default(&store.outstanding, pair, &0)
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@ucs01, IBC_APP_SEED)
    }

    public fun get_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    public fun get_self_address(): address acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        vault.self_address
    }

    public fun increase_outstanding(
        source_channel: u32, token: address, amount: u64
    ) acquires RelayStore {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        let current_outstanding =
            smart_table::borrow_mut_with_default(&mut store.outstanding, pair, 0);
        *current_outstanding = *current_outstanding + amount;
    }

    public fun decrease_outstanding(
        source_channel: u32, token: address, amount: u64
    ) acquires RelayStore {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        let current_outstanding = smart_table::borrow_mut(&mut store.outstanding, pair);
        *current_outstanding = *current_outstanding - amount;
    }

    // Initialize the RelayStore and SignerRef
    fun init_module(account: &signer) {
        assert!(signer::address_of(account) == @ucs01, E_UNAUTHORIZED);

        let vault_constructor_ref = &object::create_named_object(account, IBC_APP_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let store = RelayStore {
            denom_to_address: smart_table::new<DenomToAddressPair, address>(),
            address_to_denom: smart_table::new(),
            outstanding: smart_table::new()
        };

        move_to(vault_signer, store);

        move_to(
            vault_signer,
            SignerRef {
                self_ref: object::generate_extend_ref(vault_constructor_ref),
                self_address: signer::address_of(account)
            }
        );

        let cb =
            function_info::new_function_info(
                account,
                string::utf8(b"relay_app"),
                string::utf8(b"on_packet")
            );

        ibc::register_application<UcsRelayProof>(account, cb, new_ucs_relay_proof());
    }

    public fun on_channel_open_init(
        ordering: u8,
        connection_id: u32,
        channel_id: u32,
        version: vector<u8>
    ) {
        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

        if (ordering != ORDER_UNORDERED) {
            abort E_INVALID_PROTOCOL_ORDERING
        };
    }

    public fun on_recv_intent_packet(packet: Packet): vector<u8> {
        std::debug::print(&string::utf8(b"NOT IMPLEMENTED"));
        abort 0
    }

    public fun on_channel_open_try(
        ordering: u8,
        _connection_id: u32,
        _channel_id: u32,
        _counterparty_channel_id: u32,
        version: vector<u8>,
        counterparty_version: vector<u8>
    ) {
        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

        if (ordering != ORDER_UNORDERED) {
            abort E_INVALID_PROTOCOL_ORDERING
        };

        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_COUNTERPARTY_PROTOCOL_VERSION
        };
    }

    public fun on_channel_open_ack(
        _channel_id: u32, _counterparty_channel_id: u32, counterparty_version: vector<u8>
    ) {
        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_COUNTERPARTY_PROTOCOL_VERSION
        };
    }

    public fun on_channel_open_confirm(_channel_id: u32) {}

    public fun on_channel_close_init(_channel_id: u32) {
        abort E_UNSTOPPABLE
    }

    public fun on_channel_close_confirm(_channel_id: u32) {
        abort E_UNSTOPPABLE
    }

    public fun on_timeout_packet(packet: Packet) acquires RelayStore, SignerRef {
        // Decode the packet data
        let packet_data = ibc::packet::data(&packet);

        let relay_packet = decode_packet(*packet_data);

        // Call the refund_tokens function to refund the sender
        refund_tokens(
            ibc::packet::sequence(&packet),
            ibc::packet::source_channel(&packet),
            &relay_packet
        );

    }

    public fun encode_packet(packet: &RelayPacket): vector<u8> {
        let buf = vector::empty<u8>();

        // TODO(aeryz): document
        // Offset of `packet.sender`
        ethabi::encode_uint<u64>(&mut buf, 32 * 4);
        // Offset of `packet.receiver`
        ethabi::encode_uint<u64>(&mut buf, 32 * 6);
        // Offset of `packet.tokens`
        ethabi::encode_uint<u64>(&mut buf, 32 * 8);
        // Offset of `packet.extension`. We temporarily write `0` here because
        // `packet.tokens` contain arbitrary-length fields. Hence we can't calculate
        // the offset at this point without recursing on the tokens.
        ethabi::encode_uint<u64>(&mut buf, 0);

        // bytes encoded `packet.sender`
        ethabi::encode_vector<u8>(
            &mut buf,
            &packet.sender,
            |some_variable, data| {
                ethabi::encode_u8(some_variable, *data);
            }
        );

        // bytes encoded `packet.receiver`
        ethabi::encode_vector<u8>(
            &mut buf,
            &packet.receiver,
            |some_variable, data| {
                ethabi::encode_u8(some_variable, *data);
            }
        );

        // length prefix of the tokens array
        let num_tokens = vector::length(&packet.tokens);
        ethabi::encode_uint<u64>(&mut buf, num_tokens);

        let tokens_buf = vector::empty();
        let i = 0;
        let prev_len = 0;
        while (i < num_tokens) {
            let token = vector::borrow(&packet.tokens, i);

            // TODO(aeryz): this should be 96 when fee is enabled
            // TODO(aeryz): handle fee
            /*
            ethabi::encode_uint<u64>(&mut tokens_buf, 96);
            ethabi::encode_uint<u64>(&mut tokens_buf, 0);
            */

            ethabi::encode_uint<u64>(&mut tokens_buf, 64);
            ethabi::encode_uint<u64>(&mut tokens_buf, token.amount);

            ethabi::encode_string(&mut tokens_buf, &token.denom);

            let cursor = 32 + ((num_tokens - 1) * 32);
            ethabi::encode_uint<u64>(&mut buf, cursor + prev_len);
            prev_len = prev_len + vector::length(&tokens_buf);
            i = i + 1;
        };

        vector::append(&mut buf, tokens_buf);

        let offset_buf = vector::empty();
        ethabi::encode_uint<u64>(&mut offset_buf, vector::length(&buf));

        let i = 96;
        while (i < 128) {
            let b = vector::borrow_mut(&mut buf, i);
            *b = *vector::borrow(&offset_buf, i - 96);
            i = i + 1;
        };
        ethabi::encode_string(&mut buf, &packet.extension);

        buf
    }

    public fun decode_packet(buf: vector<u8>): RelayPacket {
        let index = 128;

        // let _unknown_data_32 = ethabi::decode_uint(buf, &mut index);
        // let _unknown_data_32 = ethabi::decode_uint(buf, &mut index);
        // let _unknown_data_32 = ethabi::decode_uint(buf, &mut index);
        // let _unknown_data_32 = ethabi::decode_uint(buf, &mut index);

        // Decoding sender address
        let sender =
            ethabi::decode_vector<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_u8(buf, index) as u8)
                }
            );

        let receiver =
            ethabi::decode_vector<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (ethabi::decode_u8(buf, index) as u8)
                }
            );

        // let receiver = from_bcs::to_address(receiver_vec);

        // Decoding the number of tokens
        let num_tokens = (ethabi::decode_uint(&buf, &mut index) as u64);

        index = index + num_tokens * 32;

        let tokens = vector::empty<Token>();
        // Decoding the token starting point and sequence
        let i = 0;
        while (i < num_tokens) {
            // dynamic data prefix
            index = index + 32;

            let amount = ethabi::decode_uint(&buf, &mut index);
            // let _fee = ethabi::decode_uint(buf, &mut index);
            let denom = ethabi::decode_string(&buf, &mut index);

            let token = Token { amount: (amount as u64), denom: denom };
            vector::push_back(&mut tokens, token);

            i = i + 1;
        };

        // Decoding the extension string
        let extension = ethabi::decode_string(&buf, &mut index);

        // Returning the decoded RelayPacket
        RelayPacket {
            sender: sender,
            receiver: receiver,
            tokens: tokens,
            extension: extension
        }
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
        let trimmed_bytes = vector::slice(s_bytes, prefix_len, s_len);
        /*
        let mut trimmed_bytes = vector::empty<u8>();

        // Manually copy elements starting from prefix_len to s_len
        let mut i = prefix_len;
        while (i < s_len) {
            vector::push_back(&mut trimmed_bytes, s_bytes[i]);
            i = i + 1;
        };
        */

        // Convert the trimmed vector back to a string
        string::utf8(trimmed_bytes)
    }

    public fun on_recv_packet_processing(
        ibc_packet: Packet // representing the IBC Packet
    ) acquires RelayStore, SignerRef {
        // Decode the RelayPacket from the IBC packet data
        let packet = decode_packet(*ibc::packet::data(&ibc_packet));
        let source_channel = ibc::packet::source_channel(&ibc_packet);
        let destination_channel = ibc::packet::destination_channel(&ibc_packet);

        // Create the denomination prefix based on source port and channel
        let prefix = make_denom_prefix(source_channel);

        // Get the receiver's address from the packet
        let receiver = from_bcs::to_address(packet.receiver);

        let i = 0;
        let packet_tokens_length = vector::length(&packet.tokens);

        while (i < packet_tokens_length) {
            let token = vector::borrow(&packet.tokens, i);

            if (token.amount == 0) {
                abort E_INVALID_AMOUNT // Abort if the amount is 0
            };

            // Create the denomination slice to check the prefix
            let denom_slice = token.denom;
            let denom_address = @0x0;

            // Check if the denomination has the expected prefix (originated from this chain)
            if (starts_with(denom_slice, prefix)) {
                // Token originated from this chain, we need to unescrow the amount

                // Trim the prefix from the denom to get the actual denom
                let trimmed_denom = trim_prefix(denom_slice, prefix);

                // REVIEW(aeryz): maybe we should just avoid putting '@' in the first place when sending
                if (starts_with(trimmed_denom, string::utf8(b"@"))) {
                    trimmed_denom = string::sub_string(&trimmed_denom, 1, 65);
                };
                denom_address = from_bcs::to_address(hex_to_bytes(trimmed_denom));

                // Decrease the outstanding amount of the token
                decrease_outstanding(source_channel, denom_address, token.amount);

                // Transfer the unescrowed tokens to the receiver
                primary_fungible_store::transfer(
                    &get_signer(),
                    get_metadata(denom_address),
                    receiver,
                    token.amount
                );

            } else {

                // Token originated from the counterparty chain, we need to mint the amount

                // Construct the foreign denomination using the source and destination channels
                let denom = make_foreign_denom(destination_channel, token.denom);

                // Create a DenomToAddressPair for the foreign denomination
                let pair = DenomToAddressPair {
                    source_channel: source_channel,
                    denom: denom
                };

                // Check if the denomination address exists in the store
                let store = borrow_global_mut<RelayStore>(get_vault_addr());
                denom_address = *smart_table::borrow_with_default(
                    &store.denom_to_address, pair, &@0x0
                );

                if (denom_address == @0x0) {
                    ucs01::fa_coin::initialize(
                        &get_signer(),
                        string::utf8(b""),
                        string::utf8(b""),
                        18,
                        string::utf8(b""),
                        string::utf8(b""),
                        *string::bytes(&denom)
                    );

                    denom_address = ucs01::fa_coin::get_metadata_address(
                        *string::bytes(&denom)
                    );

                    let pair = DenomToAddressPair {
                        source_channel: source_channel,
                        denom: denom
                    };
                    smart_table::upsert(&mut store.denom_to_address, pair, denom_address);

                    // Also update the reverse mapping (address -> denom)
                    let pair = AddressToDenomPair {
                        source_channel: destination_channel,
                        denom: denom_address
                    };
                    smart_table::upsert(&mut store.address_to_denom, pair, denom);

                    // Emit the DenomCreated event
                    event::emit(
                        DenomCreated {
                            packet_sequence: ibc::packet::sequence(&ibc_packet),
                            channel_id: source_channel,
                            denom: denom,
                            token: denom_address
                        }
                    );
                };

                // Mint tokens to the receiver's account
                let asset = get_metadata(denom_address);
                ucs01::fa_coin::mint_with_metadata(
                    &get_signer(), receiver, token.amount, asset
                );
            };

            // Emit the Received event
            event::emit(
                Received {
                    packet_sequence: ibc::packet::sequence(&ibc_packet),
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

    public fun hex_to_bytes(hex_str: String): vector<u8> {
        let hex_str_bytes = string::bytes(&hex_str);
        let byte_vector = vector::empty<u8>();

        let i = 0;
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
        sequence: u64, channel_id: u32, packet: &RelayPacket
    ) acquires RelayStore, SignerRef {
        let receiver = packet.receiver;
        let user_to_refund = from_bcs::to_address(packet.sender);

        let packet_tokens_length = vector::length(&packet.tokens);
        let i = 0;
        while (i < packet_tokens_length) {
            let token_from_vec = vector::borrow(&packet.tokens, i);
            let denom_address = get_denom_address(channel_id, token_from_vec.denom);

            if (denom_address != @0x0) {
                let token = get_metadata(denom_address);
                ucs01::fa_coin::mint_with_metadata(
                    &get_signer(),
                    user_to_refund,
                    token_from_vec.amount,
                    token
                );
            } else {
                let token_denom = token_from_vec.denom;

                if (starts_with(token_from_vec.denom, string::utf8(b"@"))) {
                    token_denom = string::sub_string(&token_denom, 1, 65);
                };
                denom_address = from_bcs::to_address(hex_to_bytes(token_denom));
                let token = get_metadata(denom_address);
                decrease_outstanding(channel_id, denom_address, token_from_vec.amount);
                primary_fungible_store::transfer(
                    &get_signer(),
                    token,
                    user_to_refund,
                    token_from_vec.amount
                );
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

    public fun on_recv_packet(packet: Packet) acquires RelayStore, SignerRef {
        on_recv_packet_processing(packet);

        dispatcher::set_return_value<UcsRelayProof>(new_ucs_relay_proof(), ACK_SUCCESS);
    }

    public fun on_acknowledge_packet(
        packet: Packet, acknowledgement: vector<u8>
    ) acquires RelayStore, SignerRef {
        if (vector::length(&acknowledgement) != ACK_LENGTH
            || (
                *vector::borrow(&acknowledgement, 0) != 0
                    && *vector::borrow(&acknowledgement, 0) != 1
            )) {
            abort E_INVALID_ACKNOWLEDGEMENT
        };

        if (*vector::borrow(&acknowledgement, 0) == ACK_FAILURE) {
            let relay_packet = decode_packet(*ibc::packet::data(&packet));
            refund_tokens(
                ibc::packet::sequence(&packet),
                ibc::packet::source_channel(&packet),
                &relay_packet
            );
        };

    }

    public entry fun send(
        sender: &signer,
        source_channel: u32,
        receiver: vector<u8>,
        denom_list: vector<address>,
        amount_list: vector<u64>,
        extension: String,
        timeout_height: u64,
        timeout_timestamp: u64
    ) acquires RelayStore, SignerRef {
        let num_tokens = vector::length(&denom_list);

        if (vector::length(&amount_list) != num_tokens) {
            abort E_INVALID_BYTES_ADDRESS
        };

        let normalized_tokens: vector<Token> = vector::empty<Token>();

        let i = 0;
        while (i < num_tokens) {
            let local_token_denom = *vector::borrow(&denom_list, i);
            let local_token_amount = *vector::borrow(&amount_list, i);

            let token_address =
                send_token(
                    sender,
                    source_channel,
                    local_token_denom,
                    local_token_amount
                );

            // Create a normalized Token struct and push to the vector
            let normalized_token = Token {
                denom: token_address,
                amount: local_token_amount
            };
            vector::push_back(&mut normalized_tokens, normalized_token);
            i = i + 1;
        };
        let packet: RelayPacket = RelayPacket {
            sender: bcs::to_bytes(&signer::address_of(sender)),
            receiver,
            tokens: normalized_tokens,
            extension
        };

        let packet_sequence =
            ibc::ibc::send_packet(
                &get_signer(),
                get_self_address(),
                source_channel,
                timeout_height,
                timeout_timestamp,
                encode_packet(&packet)
            );

        let i = 0;
        while (i < num_tokens) {
            let local_token_denom = *vector::borrow(&denom_list, i);
            let local_token_amount = *vector::borrow(&amount_list, i);
            let normalizedToken = *vector::borrow(&normalized_tokens, i);

            event::emit(
                Sent {
                    packet_sequence: packet_sequence,
                    channel_id: source_channel,
                    sender: bcs::to_bytes(&signer::address_of(sender)),
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
        sender: &signer,
        source_channel: u32,
        denom: address,
        amount: u64
    ): String acquires RelayStore, SignerRef {
        if (amount == 0) {
            abort E_INVALID_AMOUNT
        };

        let store = borrow_global<RelayStore>(get_vault_addr());

        let pair = AddressToDenomPair { source_channel, denom };
        let token_address =
            *smart_table::borrow_with_default(
                &store.address_to_denom, pair, &string::utf8(b"")
            );

        let token = get_metadata(denom);
        if (!string::is_empty(&token_address)) {
            ucs01::fa_coin::burn_with_metadata(
                &get_signer(),
                signer::address_of(sender),
                amount,
                token
            );
        } else {
            primary_fungible_store::transfer(
                sender,
                token,
                signer::address_of(&get_signer()),
                amount
            );
            increase_outstanding(source_channel, denom, amount);
            token_address = string_utils::to_string_with_canonical_addresses(&denom);
        };
        token_address
    }

    public fun on_packet<T: key>(_store: Object<T>): u64 acquires RelayStore, SignerRef {
        let value: copyable_any::Any = dispatcher::get_data(new_ucs_relay_proof());
        let type_name_output = *copyable_any::type_name(&value);

        if (type_name_output == std::type_info::type_name<ibc::RecvPacketParams>()) {
            let (pack) =
                helpers::on_recv_packet_deconstruct(
                    copyable_any::unpack<ibc::RecvPacketParams>(value)
                );
            on_recv_packet(pack);
        } else if (type_name_output
            == std::type_info::type_name<ibc::RecvIntentPacketParams>()) {
            let (pack) =
                helpers::on_recv_intent_packet_deconstruct(
                    copyable_any::unpack<ibc::RecvIntentPacketParams>(value)
                );
            on_recv_intent_packet(pack);
        } else if (type_name_output
            == std::type_info::type_name<ibc::AcknowledgePacketParams>()) {
            let (pack, acknowledgement) =
                helpers::on_acknowledge_packet_deconstruct(
                    copyable_any::unpack<ibc::AcknowledgePacketParams>(value)
                );
            on_acknowledge_packet(pack, acknowledgement);
        } else if (type_name_output
            == std::type_info::type_name<ibc::TimeoutPacketParams>()) {
            let (pack) =
                helpers::on_timeout_packet_deconstruct(
                    copyable_any::unpack<ibc::TimeoutPacketParams>(value)
                );
            on_timeout_packet(pack);
        } else if (type_name_output
            == std::type_info::type_name<ibc::ChannelOpenInitParams>()) {
            let (ordering, connection_id, channel_id, version) =
                helpers::on_channel_open_init_deconstruct(
                    copyable_any::unpack<ibc::ChannelOpenInitParams>(value)
                );
            on_channel_open_init(ordering, connection_id, channel_id, version);
        } else if (type_name_output
            == std::type_info::type_name<ibc::ChannelOpenTryParams>()) {
            let (
                ordering,
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            ) =
                helpers::on_channel_open_try_deconstruct(
                    copyable_any::unpack<ibc::ChannelOpenTryParams>(value)
                );
            on_channel_open_try(
                ordering,
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            );
        } else if (type_name_output
            == std::type_info::type_name<ibc::ChannelOpenAckParams>()) {
            let (channel_id, counterparty_channel_id, counterparty_version) =
                helpers::on_channel_open_ack_deconstruct(
                    copyable_any::unpack<ibc::ChannelOpenAckParams>(value)
                );
            on_channel_open_ack(
                channel_id, counterparty_channel_id, counterparty_version
            );
        } else if (type_name_output
            == std::type_info::type_name<ibc::ChannelOpenConfirmParams>()) {
            let channel_id =
                helpers::on_channel_open_confirm_deconstruct(
                    copyable_any::unpack<ibc::ChannelOpenConfirmParams>(value)
                );
            on_channel_open_confirm(channel_id);
        } else if (type_name_output
            == std::type_info::type_name<ibc::ChannelCloseInitParams>()) {
            let channel_id =
                helpers::on_channel_close_init_deconstruct(
                    copyable_any::unpack<ibc::ChannelCloseInitParams>(value)
                );
            on_channel_close_init(channel_id);
        } else if (type_name_output
            == std::type_info::type_name<ibc::ChannelCloseConfirmParams>()) {
            let channel_id =
                helpers::on_channel_close_confirm_deconstruct(
                    copyable_any::unpack<ibc::ChannelCloseConfirmParams>(value)
                );
            on_channel_close_confirm(channel_id);
        } else {
            std::debug::print(
                &string::utf8(b"Invalid function type detected in on_packet function!")
            );
        };
        0
    }

    // #[test]
    // public fun decode_test() {
    //     let relay =
    //         decode_packet(
    //             x"000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001e000000000000000000000000000000000000000000000000000000000000000201363462745291c711144011c1305e737dd74ace69a5576612745e29a2e4fa1b500000000000000000000000000000000000000000000000000000000000000144bde1d877da529ce4f78810b4b746bcc301c93800000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000085000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000414038616365626263323666303631373437383661623637376166323438353033333365646163303663633938633137363535353439666134393862383139393030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
    //         );

    //     assert!(
    //         relay.sender
    //             == bcs::to_bytes(
    //                 &@0x1363462745291c711144011c1305e737dd74ace69a5576612745e29a2e4fa1b5
    //             ),
    //         100
    //     );
    //     assert!(vector::length(&relay.tokens) == 1, 102);

    //     let tok = vector::borrow(&relay.tokens, 0);
    //     assert!(tok.amount == 133, 103);
    //     assert!(
    //         tok.denom
    //             == string::utf8(
    //                 b"@8acebbc26f06174786ab677af24850333edac06cc98c17655549fa498b819900"
    //             ),
    //         104
    //     );
    //     assert!(relay.extension == string::utf8(b""), 105);
    // }

    // #[test]
    // public fun test_is_valid_version() {
    //     let valid_version = string::utf8(b"ucs01-relay-1");
    //     let invalid_version = string::utf8(b"invalid-version");

    //     // Test with valid version
    //     assert!(is_valid_version(valid_version), 100);

    //     // Test with invalid version
    //     assert!(!is_valid_version(invalid_version), 101);
    // }

    // #[test]
    // public fun test_is_from_channel() {
    //     let port_id = string::utf8(b"port-1");
    //     let channel_id = string::utf8(b"channel-1");
    //     let valid_denom = string::utf8(b"port-1/channel-1/denom");
    //     let invalid_denom = string::utf8(b"other-port/other-channel/denom");

    //     // Test with valid denom
    //     assert!(
    //         is_from_channel(port_id, channel_id, valid_denom),
    //         200
    //     );

    //     // Test with invalid denom
    //     assert!(
    //         !is_from_channel(port_id, channel_id, invalid_denom),
    //         201
    //     );
    // }

    // #[test]
    // public fun test_make_foreign_denom() {
    //     let port_id = string::utf8(b"port-1");
    //     let channel_id = string::utf8(b"channel-1");
    //     let denom = string::utf8(b"denom");
    //     let expected_foreign_denom = string::utf8(b"port-1/channel-1/denom");

    //     let result = make_foreign_denom(port_id, channel_id, denom);
    //     assert!(result == expected_foreign_denom, 400);
    // }

    // #[test(admin = @ucs01)]
    // public fun test_get_denom_address(admin: &signer) acquires RelayStore {
    //     // Initialize the store in the admin's account
    //     init_module(admin);

    //     let source_channel = string::utf8(b"channel-1");
    //     let denom = string::utf8(b"denom-1");
    //     let expected_address: address = @0x1;

    //     let pair = DenomToAddressPair { source_channel, denom };
    //     let store = borrow_global_mut<RelayStore>(get_vault_addr());
    //     smart_table::upsert(&mut store.denom_to_address, pair, expected_address);

    //     // Test getting the address
    //     let result = get_denom_address(source_channel, denom);
    //     assert!(result == expected_address, 500);
    // }

    // #[test(admin = @ucs01)]
    // public fun test_get_outstanding(admin: &signer) acquires RelayStore {
    //     // Initialize the store in the admin's account
    //     init_module(admin);

    //     let source_channel = string::utf8(b"channel-1");
    //     let token = @0x1;
    //     let expected_amount: u64 = 1000;

    //     // Set up the mapping in the Relay module (this is usually done through an entry function)
    //     let pair = OutstandingPair { source_channel: source_channel, token: token };

    //     let store = borrow_global_mut<RelayStore>(get_vault_addr());
    //     smart_table::upsert(&mut store.outstanding, pair, expected_amount);

    //     // Test getting the outstanding amount
    //     let result = get_outstanding(source_channel, token);
    //     assert!(result == expected_amount, 600);
    // }

    // #[test(admin = @ucs01)]
    // public fun test_increase_outstanding(admin: &signer) acquires RelayStore {
    //     // Initialize the store
    //     let source_channel = string::utf8(b"channel-1");
    //     let token_address: address = @0x1;
    //     let initial_amount: u64 = 1000;

    //     // Initialize the store in the admin's account
    //     init_module(admin);

    //     // Increase outstanding amount
    //     increase_outstanding(source_channel, token_address, initial_amount);

    //     // Verify that the outstanding amount is updated correctly
    //     let outstanding_amount = get_outstanding(source_channel, token_address);
    //     assert!(outstanding_amount == initial_amount, 700);
    // }

    // #[test(admin = @ucs01)]
    // public fun test_decrease_outstanding(admin: &signer) acquires RelayStore {
    //     // Initialize the store
    //     let source_channel = string::utf8(b"channel-1");
    //     let token_address: address = @0x1;
    //     let initial_amount: u64 = 1000;
    //     let decrease_amount: u64 = 400;

    //     // Initialize the store in the admin's account
    //     init_module(admin);

    //     // First, increase outstanding amount
    //     increase_outstanding(source_channel, token_address, initial_amount);

    //     // Decrease the outstanding amount
    //     decrease_outstanding(source_channel, token_address, decrease_amount);

    //     // Verify that the outstanding amount is updated correctly
    //     let outstanding_amount = get_outstanding(source_channel, token_address);
    //     let expected_amount = initial_amount - decrease_amount;
    //     assert!(outstanding_amount == expected_amount, 701);
    // }

    // const TEST_NAME: vector<u8> = b"Test Coin";
    // const TEST_SYMBOL: vector<u8> = b"TST";
    // const TEST_DECIMALS: u8 = 8;
    // const TEST_ICON: vector<u8> = b"https://example.com/icon.png";
    // const TEST_PROJECT: vector<u8> = b"Test Project";

    // #[test(admin = @ucs01, bob = @0x1235)]
    // public fun test_send_token_valid_address(
    //     admin: &signer, bob: &signer
    // ) acquires RelayStore, SignerRef {
    //     // Initialize the store
    //     init_module(admin);

    //     let source_channel = string::utf8(b"channel-1");
    //     let denom = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);
    //     let amount: u64 = 1000;

    //     // let new_denom = string::utf8(b"new-denom");
    //     let denom_str = string_utils::to_string_with_canonical_addresses(&denom);
    //     // let store = borrow_global_mut<RelayStore>(get_vault_addr());
    //     // smart_table::upsert(&mut store.address_to_denom, pair, new_denom);
    //     let admin = &get_signer();

    //     ucs01::fa_coin::initialize(
    //         admin,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT),
    //         IBC_APP_SEED
    //     );

    //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);
    //     let asset = get_metadata(asset_addr);
    //     let bob_addr = signer::address_of(bob);
    //     ucs01::fa_coin::mint_with_metadata(admin, bob_addr, amount, asset);

    //     // Send tokens
    //     let result_address = send_token(bob, source_channel, asset_addr, amount);

    //     // Verify the result and outstanding balance
    //     assert!(result_address == denom_str, 100);
    //     let outstanding_balance = get_outstanding(source_channel, denom);
    //     assert!(outstanding_balance == amount, 101);

    //     let bob_balance = primary_fungible_store::balance(bob_addr, asset);
    //     let ucs01_balance =
    //         primary_fungible_store::balance(signer::address_of(&get_signer()), asset);
    //     assert!(bob_balance == 0, 102);
    //     assert!(ucs01_balance == 1000, 102);
    // }

    // #[test(admin = @ucs01, bob = @0x1235)]
    // public fun test_send_token_burn(admin: &signer, bob: &signer) acquires RelayStore, SignerRef {
    //     // Initialize the store
    //     init_module(admin);

    //     let source_channel = string::utf8(b"channel-1");
    //     let denom = @0x111111;
    //     let amount: u64 = 1000;

    //     let admin = &get_signer();
    //     ucs01::fa_coin::initialize(
    //         admin,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT),
    //         IBC_APP_SEED
    //     );

    //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);
    //     let asset = get_metadata(asset_addr);
    //     let bob_addr = signer::address_of(bob);
    //     ucs01::fa_coin::mint_with_metadata(admin, bob_addr, amount, asset);

    //     // Upsert denom to address pair
    //     let pair = AddressToDenomPair { source_channel, denom: asset_addr };
    //     let new_denom = string::utf8(b"new-denom");

    //     let store = borrow_global_mut<RelayStore>(get_vault_addr());
    //     smart_table::upsert(&mut store.address_to_denom, pair, new_denom);

    //     // Send tokens

    //     let supply_before = option::extract(&mut fungible_asset::supply(asset));
    //     assert!(supply_before == 1000, 102);

    //     let result_address = send_token(bob, source_channel, asset_addr, amount);

    //     // Verify the result and outstanding balance
    //     assert!(string::length(&result_address) != 0, 100);
    //     let outstanding_balance = get_outstanding(source_channel, denom);
    //     assert!(outstanding_balance == 0, 101);

    //     let bob_balance = primary_fungible_store::balance(bob_addr, asset);
    //     assert!(bob_balance == 0, 102);

    //     let supply_after = option::extract(&mut fungible_asset::supply(asset));
    //     assert!(supply_after == 0, 102);
    // }

    // #[test(admin = @ucs01)]
    // #[expected_failure(abort_code = E_INVALID_AMOUNT)]
    // public fun test_send_zero_amount(admin: &signer) acquires RelayStore, SignerRef {
    //     // Initialize the store
    //     init_module(admin);

    //     let source_channel = string::utf8(b"channel-1");
    //     let _denom = @0x111111;
    //     let admin = &get_signer();
    //     ucs01::fa_coin::initialize(
    //         admin,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT),
    //         IBC_APP_SEED
    //     );

    //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);
    //     let _asset = get_metadata(asset_addr);

    //     // Attempt to send zero amount
    //     send_token(admin, source_channel, asset_addr, 0);
    // }

    #[test]
    public fun test_encode() {
        let token = Token { denom: string::utf8(b"denom"), amount: 1000 };
        let token2 = Token { denom: string::utf8(b"this is amazing"), amount: 3000 };
        let token3 = Token { denom: string::utf8(b"insane cool"), amount: 3 };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);
        vector::push_back(&mut tokens, token2);
        vector::push_back(&mut tokens, token3);

        let sender = bcs::to_bytes(&@0x111111111111111111111);
        let receiver = bcs::to_bytes(&@0x0000000000000000000000000000000000000033);
        let extension = string::utf8(b"extension");
        let packet = RelayPacket {
            sender: sender,
            receiver: receiver,
            tokens: tokens,
            extension: extension
        };
        let encoded = encode_packet(&packet);
        let decoded = decode_packet(encoded);

        assert!(decoded.sender == sender, 100);
        assert!(decoded.receiver == receiver, 101);
        assert!(decoded.extension == extension, 102);
        let token = vector::borrow(&decoded.tokens, 0);
        assert!(token.denom == string::utf8(b"denom"), 103);
        assert!(token.amount == 1000, 104);
        let token2 = vector::borrow(&decoded.tokens, 1);
        assert!(token2.denom == string::utf8(b"this is amazing"), 105);
        assert!(token2.amount == 3000, 106);
        let token3 = vector::borrow(&decoded.tokens, 2);
        assert!(token3.denom == string::utf8(b"insane cool"), 107);
        assert!(token3.amount == 3, 108);
    }

    // #[test(admin = @ucs01, alice = @0x1234)]
    // public fun test_refund_tokens(admin: &signer, alice: address) acquires RelayStore, SignerRef {
    //     init_module(admin);

    //     let source_channel = string::utf8(b"channel-1");
    //     let amount: u64 = 1000;

    //     let token_owner = &get_signer();

    //     // Step 2: Mint some tokens to Alice
    //     ucs01::fa_coin::initialize(
    //         token_owner,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT),
    //         IBC_APP_SEED
    //     );

    //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);
    //     let asset = get_metadata(asset_addr);

    //     // Step 3: Simulate sending tokens (for refund purposes)
    //     let token = Token {
    //         denom: string_utils::to_string_with_canonical_addresses(&asset_addr),
    //         amount: amount
    //     };
    //     let tokens = vector::empty<Token>();
    //     vector::push_back(&mut tokens, token);

    //     let relay_packet = RelayPacket {
    //         sender: bcs::to_bytes(&alice),
    //         receiver: bcs::to_bytes(&@0x0000000000000000000000000000000000000022),
    //         tokens: tokens,
    //         extension: string::utf8(b"extension")
    //     };

    //     // Insert mapping for the denom -> address
    //     let pair = DenomToAddressPair {
    //         source_channel,
    //         denom: string_utils::to_string_with_canonical_addresses(&asset_addr)
    //     };
    //     let store = borrow_global_mut<RelayStore>(get_vault_addr());
    //     smart_table::upsert(&mut store.denom_to_address, pair, asset_addr);

    //     // Step 4: Call the refund function
    //     let sequence = 1;
    //     refund_tokens(sequence, source_channel, &relay_packet);

    //     // Step 5: Verify the results
    //     let alice_balance = primary_fungible_store::balance(alice, asset);
    //     assert!(alice_balance == amount, 100); // Alice should have received the refund

    // }

    // #[test(admin = @ucs01, alice = @0x1234)]
    // public fun test_refund_tokens_zero_address(
    //     admin: &signer, alice: address
    // ) acquires RelayStore, SignerRef {
    //     init_module(admin);

    //     let source_channel = string::utf8(b"channel-1");
    //     let amount: u64 = 1000;

    //     let token_owner = &get_signer();

    //     ucs01::fa_coin::initialize(
    //         token_owner,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT),
    //         IBC_APP_SEED
    //     );
    //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);

    //     let asset = get_metadata(asset_addr);

    //     ucs01::fa_coin::mint_with_metadata(
    //         token_owner,
    //         signer::address_of(token_owner),
    //         1000,
    //         asset
    //     );

    //     // Step 3: Simulate sending tokens (for refund purposes)
    //     let token = Token {
    //         denom: string_utils::to_string_with_canonical_addresses(&asset_addr),
    //         amount: amount
    //     };
    //     let tokens = vector::empty<Token>();
    //     vector::push_back(&mut tokens, token);

    //     let relay_packet = RelayPacket {
    //         sender: bcs::to_bytes(&alice),
    //         receiver: bcs::to_bytes(&@0x0000000000000000000000000000000000000022),
    //         tokens: tokens,
    //         extension: string::utf8(b"extension")
    //     };
    //     increase_outstanding(source_channel, asset_addr, amount);

    //     let outstanding_balance = get_outstanding(source_channel, asset_addr);
    //     assert!(outstanding_balance == 1000, 200); // The outstanding balance should be reduced to 0.

    //     let sequence = 1;
    //     refund_tokens(sequence, source_channel, &relay_packet);

    //     let outstanding_balance = get_outstanding(source_channel, asset_addr);
    //     assert!(outstanding_balance == 0, 200); // The outstanding balance should be reduced to 0.

    //     let alice_balance = primary_fungible_store::balance(alice, asset);
    //     assert!(alice_balance == 0, 201); // Alice should not receive any tokens in this case.
    // }

    // #[test(admin = @ucs01, alice = @0x1234, bob = @0x1235)]
    // public fun test_on_recv_packet_processing_local_token(
    //     admin: &signer, alice: address, bob: address
    // ) acquires RelayStore, SignerRef {
    //     // Step 1: Initialize the store
    //     init_module(admin);

    //     // Step 2: Setup the initial mappings for local tokens
    //     let source_channel = string::utf8(b"channel-1");
    //     let destination_channel = string::utf8(b"dest-channel");
    //     let port_id = string::utf8(b"port-1");
    //     let local_token_address = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);

    //     let token_owner = &get_signer();

    //     // Mint some tokens to simulate the token creation on this chain
    //     ucs01::fa_coin::initialize(
    //         token_owner,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT),
    //         IBC_APP_SEED
    //     );

    //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);

    //     let asset = get_metadata(asset_addr);
    //     // Step 3: Mint tokens to the relay module's account (escrow)
    //     ucs01::fa_coin::mint_with_metadata(
    //         token_owner,
    //         signer::address_of(token_owner),
    //         1000,
    //         asset
    //     );
    //     increase_outstanding(source_channel, local_token_address, 1000);

    //     // Step 4: Create the RelayPacket with a local token
    //     let new_denom = make_denom_prefix(port_id, source_channel);
    //     string::append(
    //         &mut new_denom,
    //         string_utils::to_string_with_canonical_addresses(&local_token_address)
    //     );
    //     let token = Token { denom: new_denom, amount: 500 };
    //     let tokens = vector::empty<Token>();
    //     vector::push_back(&mut tokens, token);

    //     let relay_packet = RelayPacket {
    //         sender: bcs::to_bytes(&alice),
    //         receiver: bcs::to_bytes(&bob),
    //         tokens: tokens,
    //         extension: string::utf8(b"")
    //     };

    //     // Step 5: Create the IBC packet
    //     let ibc_packet =
    //         ibc::packet::new(
    //             1, // sequence
    //             port_id,
    //             source_channel,
    //             port_id,
    //             destination_channel,
    //             encode_packet(&relay_packet),
    //             height::new(1, 100),
    //             1000000
    //         );

    //     // Step 6: Process the IBC packet
    //     on_recv_packet_processing(ibc_packet);

    //     // Step 7: Verify the token was transferred to Bob
    //     let bob_balance =
    //         primary_fungible_store::balance(bob, get_metadata(local_token_address));
    //     assert!(bob_balance == 500, 100); // Bob should have received the token

    //     // Step 8: Verify the outstanding amount was decreased
    //     let outstanding_balance = get_outstanding(source_channel, local_token_address);
    //     assert!(outstanding_balance == 500, 101); // Outstanding should be reduced by 500
    // }

    // #[test(admin = @ucs01, alice = @0x1234, bob = @0x1235)]
    // public fun test_on_recv_packet_processing_foreign_token_denom_address_zero(
    //     admin: &signer, alice: address, bob: address
    // ) acquires RelayStore, SignerRef {
    //     // Step 1: Initialize the store
    //     init_module(admin);

    //     // Step 2: Set up mappings and mint tokens on the counterparty chain
    //     let source_channel = string::utf8(b"channel-1");
    //     let destination_channel = string::utf8(b"dest-channel");
    //     let port_id = string::utf8(b"port-1");

    //     // Step 3: Create a RelayPacket with a foreign token denomination
    //     let foreign_denom = string::utf8(b"foreign-token-denom");
    //     let token = Token { denom: foreign_denom, amount: 500 };
    //     let tokens = vector::empty<Token>();
    //     vector::push_back(&mut tokens, token);

    //     let relay_packet = RelayPacket {
    //         sender: bcs::to_bytes(&alice),
    //         receiver: bcs::to_bytes(&bob),
    //         tokens: tokens,
    //         extension: string::utf8(b"")
    //     };

    //     // Step 4: Create the IBC packet
    //     let ibc_packet =
    //         ibc::packet::new(
    //             1, // sequence
    //             port_id,
    //             source_channel,
    //             port_id,
    //             destination_channel,
    //             encode_packet(&relay_packet),
    //             height::new(1, 100),
    //             1000000
    //         );

    //     // Step 5: Process the IBC packet (will enter the 'else' block since it's a foreign token)
    //     on_recv_packet_processing(ibc_packet);

    //     // Step 6: Verify that a new denomination was created and minting occurred
    //     let _store = borrow_global_mut<RelayStore>(get_vault_addr());
    //     // Construct the foreign denomination using the source and destination channels
    //     let our_denom = make_foreign_denom(port_id, destination_channel, foreign_denom);
    //     let denom_address = get_denom_address(source_channel, our_denom);
    //     assert!(denom_address != @0x0, 100); // The new token address should have been created

    //     // Step 7: Verify the token was minted to Bob's account
    //     let bob_balance =
    //         primary_fungible_store::balance(bob, get_metadata(denom_address));
    //     assert!(bob_balance == 500, 101); // Bob should have received the minted foreign token
    // }

    // #[test(admin = @ucs01, alice = @0x1234, bob = @0x1235)]
    // public fun test_on_recv_packet_processing_foreign_token_existing_denom_address(
    //     admin: &signer, alice: address, bob: address
    // ) acquires RelayStore, SignerRef {
    //     // Step 1: Initialize the store
    //     init_module(admin);

    //     // Step 2: Set up mappings and mint tokens on the counterparty chain
    //     let source_channel = string::utf8(b"channel-1");
    //     let destination_channel = string::utf8(b"dest-channel");
    //     let port_id = string::utf8(b"port-1");
    //     let _foreign_denom = string::utf8(b"foreign-token-denom");
    //     let token_owner = &get_signer();

    //     ucs01::fa_coin::initialize(
    //         token_owner,
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT),
    //         IBC_APP_SEED
    //     );
    //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);
    //     let _asset = get_metadata(asset_addr);

    //     let new_denom = make_denom_prefix(port_id, destination_channel);
    //     string::append(
    //         &mut new_denom,
    //         string_utils::to_string_with_canonical_addresses(&asset_addr)
    //     );
    //     // Step 3: Insert an existing foreign token address into the store
    //     let _existing_denom_address: address = @0x1;
    //     let pair = DenomToAddressPair { source_channel: source_channel, denom: new_denom };

    //     let store = borrow_global_mut<RelayStore>(get_vault_addr());
    //     smart_table::upsert(&mut store.denom_to_address, pair, asset_addr);

    //     // Step 4: Create a RelayPacket with the foreign token denomination
    //     let token = Token {
    //         denom: string_utils::to_string_with_canonical_addresses(&asset_addr),
    //         amount: 500
    //     };
    //     let tokens = vector::empty<Token>();
    //     vector::push_back(&mut tokens, token);

    //     let relay_packet = RelayPacket {
    //         sender: bcs::to_bytes(&alice),
    //         receiver: bcs::to_bytes(&bob),
    //         tokens: tokens,
    //         extension: string::utf8(b"")
    //     };

    //     // Step 5: Create the IBC packet
    //     let ibc_packet =
    //         ibc::packet::new(
    //             1, // sequence
    //             port_id,
    //             source_channel,
    //             port_id,
    //             destination_channel,
    //             encode_packet(&relay_packet),
    //             height::new(1, 100),
    //             1000000
    //         );

    //     // Step 6: Process the IBC packet (will use the existing token address in the 'else' block)
    //     on_recv_packet_processing(ibc_packet);

    //     // Step 7: Verify that the existing token address was used

    //     let denom_address = get_denom_address(source_channel, new_denom);
    //     assert!(denom_address == asset_addr, 100); // It should be the same as the existing address

    //     // Step 8: Verify the token was minted to Bob's account
    //     let bob_balance = primary_fungible_store::balance(bob, get_metadata(asset_addr));
    //     assert!(bob_balance == 500, 101); // Bob should have received the minted foreign token
    // }

    // #[test(admin = @ucs01, alice = @0x1234, bob = @0x1235)]
    // #[expected_failure(abort_code = E_INVALID_AMOUNT)]
    // public fun test_on_recv_packet_processing_local_token_revert_amount_zero(
    //     admin: &signer, alice: address, bob: address
    // ) acquires RelayStore, SignerRef {
    //     // Step 1: Initialize the store
    //     init_module(admin);

    //     // Step 2: Setup the initial mappings for local tokens
    //     let source_channel = string::utf8(b"channel-1");
    //     let destination_channel = string::utf8(b"dest-channel");
    //     let port_id = string::utf8(b"port-1");
    //     let local_token_address = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);

    //     // Mint some tokens to simulate the token creation on this chain
    //     ucs01::fa_coin::initialize(
    //         &get_signer(),
    //         string::utf8(TEST_NAME),
    //         string::utf8(TEST_SYMBOL),
    //         TEST_DECIMALS,
    //         string::utf8(TEST_ICON),
    //         string::utf8(TEST_PROJECT),
    //         IBC_APP_SEED
    //     );

    //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);
    //     let asset = get_metadata(asset_addr);
    //     // Step 3: Mint tokens to the relay module's account (escrow)
    //     ucs01::fa_coin::mint_with_metadata(&get_signer(), @ucs01, 1000, asset);
    //     increase_outstanding(source_channel, local_token_address, 1000);

    //     // Step 4: Create the RelayPacket with a local token
    //     let new_denom = make_denom_prefix(port_id, source_channel);
    //     string::append(
    //         &mut new_denom,
    //         string_utils::to_string_with_canonical_addresses(&local_token_address)
    //     );
    //     let token = Token { denom: new_denom, amount: 0 };
    //     let tokens = vector::empty<Token>();
    //     vector::push_back(&mut tokens, token);

    //     let relay_packet = RelayPacket {
    //         sender: bcs::to_bytes(&alice),
    //         receiver: bcs::to_bytes(&bob),
    //         tokens: tokens,
    //         extension: string::utf8(b"")
    //     };

    //     // Step 5: Create the IBC packet
    //     let ibc_packet =
    //         ibc::packet::new(
    //             1, // sequence
    //             port_id,
    //             source_channel,
    //             port_id,
    //             destination_channel,
    //             encode_packet(&relay_packet),
    //             height::new(1, 100),
    //             1000000
    //         );

    //     // Step 6: Process the IBC packet
    //     on_recv_packet_processing(ibc_packet);

    //     // Step 7: Verify the token was transferred to Bob
    //     let bob_balance =
    //         primary_fungible_store::balance(bob, get_metadata(local_token_address));
    //     assert!(bob_balance == 500, 100); // Bob should have received the token

    //     // Step 8: Verify the outstanding amount was decreased
    //     let outstanding_balance = get_outstanding(source_channel, local_token_address);
    //     assert!(outstanding_balance == 500, 101); // Outstanding should be reduced by 500
    // }

    // // #[test(admin = @ucs01, alice = @0x1234, bob = @0x1235, ibc_admin = @ibc)]
    // // public fun test_send_valid(admin: &signer, ibc_admin: &signer, alice: &signer, bob: address) acquires RelayStore, SignerRef {
    // //     // Initialize the store
    // //     init_module(admin);
    // //     ibc::init_module_public(ibc_admin);

    // //     let port_id = string::utf8(b"0x0000000000000000000000000000000000000000000000000004444444444444");
    // //     let channel_id = string::utf8(b"channel-1");
    // //     let connection_id = string::utf8(b"connection-1");
    // //     let client_id = string::utf8(b"client-1");

    // //     ibc::test_fill_all_states(port_id, channel_id, connection_id, client_id);

    // //     let source_channel = string::utf8(b"channel-1");
    // //     let denom_list = vector::empty<address>();
    // //     let amount_list = vector::empty<u64>();

    // //     ucs01::fa_coin::initialize(
    // //         &get_signer(),
    // //         string::utf8(TEST_NAME),
    // //         string::utf8(TEST_SYMBOL),
    // //         TEST_DECIMALS,
    // //         string::utf8(TEST_ICON),
    // //         string::utf8(TEST_PROJECT),
    // //         IBC_APP_SEED
    // //     );

    // //     let asset_addr = ucs01::fa_coin::get_metadata_address(IBC_APP_SEED);

    // //     ucs01::fa_coin::mint_with_metadata(&get_signer(), signer::address_of(alice), 1000, get_metadata(asset_addr));

    // //     vector::push_back(&mut denom_list, asset_addr);
    // //     vector::push_back(&mut amount_list, 500);  // Send half of the minted amount

    // //     let extension = string::utf8(b"optional-extension");
    // //     let timeout_height_number = 1;
    // //     let timeout_height_height = 100;
    // //     let timeout_timestamp = 100000;

    // //     send(
    // //         alice,
    // //         source_channel,
    // //         bob,
    // //         denom_list,
    // //         amount_list,
    // //         extension,
    // //         timeout_height_number,
    // //         timeout_height_height,
    // //         timeout_timestamp
    // //     );

    // //     let outstanding_balance = get_outstanding(source_channel, asset_addr);
    // //     assert!(outstanding_balance == 500, 100);  // Only 500 tokens were sent

    // //     let ucs01_balance = primary_fungible_store::balance(@ucs01, get_metadata(asset_addr));
    // //     assert!(ucs01_balance == 500, 101);  // 500 tokens should be transferred to ucs01

    // //     let alice_balance = primary_fungible_store::balance(signer::address_of(alice), get_metadata(asset_addr));
    // //     assert!(alice_balance == 500, 102);  // Alice should have 500 tokens left after sending
    // // }
}
