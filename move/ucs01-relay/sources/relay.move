module UCS01::Relay {    
    use IBC::ibc;
    use IBC::channel;
    use IBC::height;
    use IBC::packet::{Packet};
    use aptos_framework::primary_fungible_store;
    use aptos_framework::object::{Self, Object};
    use std::event;

    use std::string::{Self, String};
    use std::string_utils;
    use std::from_bcs;
    use std::bcs;
    use aptos_framework::fungible_asset::{Metadata};
    use aptos_framework::signer;
    use aptos_std::smart_table::{Self, SmartTable};
    use std::vector;
    use UCS01::EthABI;

    const ASSET_SYMBOL: vector<u8> = b"FA";
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

    struct Token has copy, drop, store {
        denom: String,
        amount: u64,
    }

    struct LocalToken has copy, drop, store {
        denom: address,
        amount: u64,
    }

    struct RelayPacket has copy, drop, store {
        sender: vector<u8>,
        receiver: address,
        tokens: vector<Token>,
        extension: String,
    }

    struct DenomToAddressPair has copy, drop, store {
        source_channel: String,
        denom: String,
    }

    struct AddressToDenomPair has copy, drop, store {
        source_channel: String,
        denom: address,
    }

    struct OutstandingPair has copy, drop, store {
        source_channel: String,
        token: address,
    }

    struct RelayStore has key {
        denom_to_address: SmartTable<DenomToAddressPair, address>,
        address_to_denom: SmartTable<AddressToDenomPair, String>,
        outstanding: SmartTable<OutstandingPair, u64>,
    }



    struct SignerRef has key {
        self_ref: object::ExtendRef,
        self_address: address
    }

    // Events
    #[event]
    struct DenomCreated has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        denom: String,
        token: address,
    }

    #[event]
    struct Received has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        sender: vector<u8>,
        receiver: address,
        denom: String,
        token: address,
        amount: u64,
    }

    #[event]
    struct Sent has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        sender: vector<u8>,
        receiver: address,
        denom: String,
        token: address,
        amount: u64,
    }

    #[event]
    struct Refunded has copy, drop, store {
        packet_sequence: u64,
        channel_id: String,
        sender: address,
        receiver: address,
        denom: String,
        token: address,
        amount: u64,
    }

    // View/Pure Functions
    public fun is_valid_version(version: String): bool {
        let version_bytes = string::bytes(&version);
        *version_bytes == VERSION
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

    public fun is_from_channel(port_id: String, channel_id: String, denom: String): bool {
        let prefix = make_denom_prefix(port_id, channel_id);
        starts_with(denom, prefix)
    }

    public fun make_denom_prefix(port_id: String, channel_id: String): String {
        let prefix = port_id;
        string::append_utf8(&mut prefix, b"/");
        string::append(&mut prefix, channel_id);
        string::append_utf8(&mut prefix, b"/");
        prefix
    }

    public fun make_foreign_denom(port_id: String, channel_id: String, denom: String): String {
        let foreign_denom = make_denom_prefix(port_id, channel_id);
        string::append(&mut foreign_denom, denom);
        foreign_denom
    }

    public fun get_denom_address(source_channel: String, denom: String): address acquires RelayStore {
        let store = borrow_global<RelayStore>(get_vault_addr());
        let pair = DenomToAddressPair { source_channel, denom };
        *smart_table::borrow_with_default(&store.denom_to_address, pair, &@0x0)
    }

    public fun get_outstanding(source_channel: String, token: address): u64 acquires RelayStore {
        let store = borrow_global<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        *smart_table::borrow_with_default(&store.outstanding, pair, &0)
    }

    #[view]
    public fun get_vault_addr(): address {
        object::create_object_address(&@UCS01, IBC_APP_SEED)
    }

    public fun get_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    public fun get_self_address(): address acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        vault.self_address
    }

    public fun increase_outstanding(source_channel: String, token: address, amount: u64) acquires RelayStore {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        let current_outstanding = smart_table::borrow_mut_with_default(&mut store.outstanding, pair, 0);
        *current_outstanding = *current_outstanding + amount;
    }

    public fun decrease_outstanding(source_channel: String, token: address, amount: u64) acquires RelayStore {
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        let pair = OutstandingPair { source_channel, token };
        let current_outstanding = smart_table::borrow_mut(&mut store.outstanding, pair);
        *current_outstanding = *current_outstanding - amount;
    }

    // Initialize the RelayStore and SignerRef
    fun init_module(account: &signer) {
        assert!(signer::address_of(account) == @UCS01, E_UNAUTHORIZED);

        let vault_constructor_ref = &object::create_named_object(account, IBC_APP_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        let store = RelayStore {
            denom_to_address: smart_table::new<DenomToAddressPair, address>(),
            address_to_denom: smart_table::new(),
            outstanding: smart_table::new(),
        };

        move_to(vault_signer, store);

        move_to(vault_signer, SignerRef {
            self_ref: object::generate_extend_ref(vault_constructor_ref),
            self_address: signer::address_of(account),
        });
    }

    public entry fun channel_open_init(
        connection_hops: vector<String>,
        ordering: u8,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        version: String,
    ) acquires SignerRef {
        let counterparty = channel::new_counterparty(counterparty_port_id, counterparty_channel_id);
        ibc::channel_open_init(
            &get_signer(),
            get_self_address(),
            connection_hops,
            ordering,
            counterparty,
            version,
        );

        if (!is_valid_version(version)) {
            abort E_INVALID_PROTOCOL_VERSION
        };

        if (ordering != ORDER_UNORDERED) {
            abort E_INVALID_PROTOCOL_ORDERING
        };
    }


    public entry fun chan_open_try(
        connection_hops: vector<String>,
        ordering: u8,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        version: String,
        proof_init: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires SignerRef {
        ibc::channel_open_try(
            &get_signer(),
            get_self_address(),
            connection_hops,
            ordering,
            channel::new_counterparty(counterparty_port_id, counterparty_channel_id),
            counterparty_version,
            version,
            proof_init,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );

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

    public entry fun channel_open_ack(
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires SignerRef {
        // Store the channel_id
        ibc::channel_open_ack(
            &get_signer(),
            get_self_address(),
            // port_id,
            channel_id,
            counterparty_channel_id,
            counterparty_version,
            proof_try,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );
        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_COUNTERPARTY_PROTOCOL_VERSION
        };
    }

    public entry fun channel_open_confirm(
        channel_id: String,
        proof_ack: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires SignerRef {
        ibc::channel_open_confirm(
            &get_signer(),
            get_self_address(),
            // port_id,
            channel_id,
            proof_ack,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );
    }
    public entry fun channel_close_init(
        _channel_id: String
    ) {
        abort E_UNSTOPPABLE
    }

    public entry fun channel_close_confirm(
        _channel_id: String
    ) {
        abort E_UNSTOPPABLE
    }

    public fun timeout_packet(
        sequence: u64,
        _source_port: String,
        source_channel: String,
        _destination_port: String,
        _destination_channel: String,
        data: vector<u8>, 
        _timeout_height: height::Height,
        _timeout_timestamp: u64,
    ) acquires RelayStore, SignerRef {
        // Decode the packet data
        let relay_packet = decode_packet(data);

        // Call the refund_tokens function to refund the sender
        refund_tokens(sequence, source_channel, &relay_packet);
    }


    // TODO: It works but there are couple unknown datas.
    // tbh i don't know if it will work for every possible relaypacket struct
    public fun encode_packet(packet: &RelayPacket): vector<u8> {
        let buf = vector::empty<u8>();

        // TODO(aeryz): document
        EthABI::encode_uint<u64>(&mut buf, 32 * 4);
        EthABI::encode_uint<u64>(&mut buf, 32 * 6);
        EthABI::encode_uint<u64>(&mut buf, 32 * 8);
        EthABI::encode_uint<u64>(&mut buf, 32 * 9 + (32 * 6 * vector::length(&packet.tokens)));


        // TODO: how to encode senders now?
        EthABI::encode_vector<u8>(&mut buf, packet.sender, |some_variable, data| {
            EthABI::encode_u8(some_variable, data);
        });
        
        let receiver_bytes = bcs::to_bytes(&packet.receiver);
        
        // TODO: how to encode senders now?
        EthABI::encode_vector<u8>(&mut buf, receiver_bytes, |some_variable, data| {
            EthABI::encode_u8(some_variable, data);
        });

        let num_tokens = vector::length(&packet.tokens);
        EthABI::encode_uint<u64>(&mut buf, num_tokens);

        let i = 0;
        while (i < num_tokens) {
            let cursor = 32 + (6 * 32 * (num_tokens - 1));
            EthABI::encode_uint<u64>(&mut buf, cursor);
            i = i + 1;
        };

        let i = 0;
        while (i < num_tokens) {
            let token = vector::borrow(&packet.tokens, i);
            
            EthABI::encode_uint<u64>(&mut buf, 96);
            EthABI::encode_uint<u64>(&mut buf, token.amount);
            // TODO(aeryz): handle fee
            EthABI::encode_uint<u64>(&mut buf, 0);

            EthABI::encode_string(&mut buf, token.denom);

            i = i + 1;
        };

        EthABI::encode_string(&mut buf, packet.extension);

        buf
    }

    public fun decode_packet(buf: vector<u8>): RelayPacket {
        let index = 128;

        // let _unknown_data_32 = EthABI::decode_uint(buf, &mut index);
        // let _unknown_data_32 = EthABI::decode_uint(buf, &mut index);
        // let _unknown_data_32 = EthABI::decode_uint(buf, &mut index);
        // let _unknown_data_32 = EthABI::decode_uint(buf, &mut index);

        // Decoding sender address
        let sender = EthABI::decode_vector<u8>(buf, &mut index, |buf, index| {
            (EthABI::decode_u8(buf, index) as u8)
        });
        std::debug::print(&sender);
        std::debug::print(&index);

        let receiver_vec = EthABI::decode_vector<u8>(buf, &mut index, |buf, index| {
            (EthABI::decode_u8(buf, index) as u8)
        });
        std::debug::print(&receiver_vec);
        std::debug::print(&index);

        let receiver = from_bcs::to_address(receiver_vec);

        // Decoding the number of tokens
        let num_tokens = (EthABI::decode_uint(buf, &mut index) as u64);
        std::debug::print(&num_tokens);

        index = index + num_tokens * 32;

        let tokens = vector::empty<Token>();
        // Decoding the token starting point and sequence
        let i = 0;
        while (i < num_tokens) {
            // dynamic data prefix
            index = index + 32;

            let amount = EthABI::decode_uint(buf, &mut index);
            std::debug::print(&amount);
            let _fee = EthABI::decode_uint(buf, &mut index); 
            std::debug::print(&_fee);
            let denom = EthABI::decode_string(buf, &mut index);
            std::debug::print(&denom);

            let token = Token {
                amount: (amount as u64),
                denom: denom,
            };
            vector::push_back(&mut tokens, token);

            i = i + 1;
        };

        // // Decoding the tokens
        // let i = 0;
        // while (i < num_tokens) {
        //     // Decoding unknown data (64 as u256)
        //     let _unknown_data_64 = EthABI::decode_uint(buf, &mut index);

        //     // Decoding the amount of token
        //     let amount = EthABI::decode_uint(buf, &mut index);

        //     // Decoding the token denomination string
        //     let denom = EthABI::decode_string(buf, &mut index);


        //     i = i + 1;
        // };

        // Decoding the extension string
        let extension = EthABI::decode_string(buf, &mut index);

        // Returning the decoded RelayPacket
        RelayPacket {
            sender: sender,
            receiver: receiver,
            tokens: tokens,
            extension: extension,
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

        // Convert the trimmed vector back to a string
        string::utf8(trimmed_bytes)
    }

    public fun on_recv_packet_processing(
        ibc_packet: Packet // representing the IBC Packet
    ) acquires RelayStore, SignerRef {
        // Decode the RelayPacket from the IBC packet data
        let packet = decode_packet(*IBC::packet::data(&ibc_packet));
        let source_channel = *IBC::packet::source_channel(&ibc_packet);
        let destination_channel = *IBC::packet::destination_channel(&ibc_packet);


        // Create the denomination prefix based on source port and channel
        let prefix = make_denom_prefix(*IBC::packet::source_port(&ibc_packet), source_channel);

        // Get the receiver's address from the packet
        let receiver = packet.receiver;//from_bcs::to_address(*string::bytes(packet.receiver));

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

                if(starts_with(trimmed_denom, string::utf8(b"@"))) {
                    trimmed_denom = string::sub_string(&trimmed_denom, 1, 65);
                };
                let denom_address = from_bcs::to_address(hex_to_bytes(trimmed_denom));

                // Decrease the outstanding amount of the token
                decrease_outstanding(source_channel, denom_address, token.amount);

                // Transfer the unescrowed tokens to the receiver
                primary_fungible_store::transfer(&get_signer(), get_metadata(denom_address), receiver, token.amount);

            } else {

                // Token originated from the counterparty chain, we need to mint the amount

                // Construct the foreign denomination using the source and destination channels
                let denom = make_foreign_denom(
                    *IBC::packet::destination_port(&ibc_packet),
                    destination_channel,
                    token.denom
                );

                // Create a DenomToAddressPair for the foreign denomination
                let pair = DenomToAddressPair {
                    source_channel: source_channel,
                    denom: denom,
                };
                                
                // Check if the denomination address exists in the store
                let store = borrow_global_mut<RelayStore>(get_vault_addr());
                let denom_address = *smart_table::borrow_with_default(&store.denom_to_address, pair, &@0x0);


                if (denom_address == @0x0) {
                                        
                    // TODO CHANGE THOSE, in here we are creating(?) a new token
                    UCS01::fa_coin::initialize(
                        &get_signer(),
                        string::utf8(b"TODO"),
                        string::utf8(b"TODO"),
                        18,
                        string::utf8(b"TODO"),
                        string::utf8(b"TODO"),
                        *string::bytes(&denom),
                    );

                    denom_address = UCS01::fa_coin::get_metadata_address(*string::bytes(&denom));

                    let pair = DenomToAddressPair {
                        source_channel: source_channel,
                        denom: denom,
                    };
                    smart_table::upsert(&mut store.denom_to_address, pair, denom_address);

                    // Also update the reverse mapping (address -> denom)
                    let pair = AddressToDenomPair {
                        source_channel: destination_channel,
                        denom: denom_address,
                    };
                    smart_table::upsert(&mut store.address_to_denom, pair, denom);

                    // Emit the DenomCreated event
                    event::emit(DenomCreated {
                        packet_sequence: IBC::packet::sequence(&ibc_packet),
                        channel_id: source_channel,
                        denom: denom,
                        token: denom_address,
                    });
                };

                // Mint tokens to the receiver's account
                let asset = get_metadata(denom_address);
                UCS01::fa_coin::mint_with_metadata(&get_signer(), receiver, token.amount, asset);
            };

            // Emit the Received event
            event::emit(Received {
                packet_sequence: IBC::packet::sequence(&ibc_packet),
                channel_id: destination_channel,
                sender: packet.sender,
                receiver: receiver,
                denom: token.denom,
                token: denom_address,
                amount: token.amount
            });

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
            0x00
        }
    }

    fun refund_tokens(
        sequence: u64,
        channel_id: String,
        packet: &RelayPacket
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
                UCS01::fa_coin::mint_with_metadata(&get_signer(), user_to_refund, token_from_vec.amount, token);
            } else {
                let token_denom = token_from_vec.denom;

                if(starts_with(token_from_vec.denom, string::utf8(b"@"))) {
                    token_denom = string::sub_string(&token_denom, 1, 65);
                };
                let denom_address = from_bcs::to_address(hex_to_bytes(token_denom));
                let token = get_metadata(denom_address);
                decrease_outstanding(channel_id, denom_address, token_from_vec.amount);
                primary_fungible_store::transfer(&get_signer(), token, @zero_account, token_from_vec.amount);
            };

            // Emit a Refunded event
            event::emit(Refunded {
                packet_sequence: sequence,
                channel_id: channel_id,
                sender: user_to_refund,
                receiver: receiver,
                denom: token_from_vec.denom,
                token: denom_address,
                amount: token_from_vec.amount
            });

            i = i + 1;
        }
    }

    
    public entry fun recv_packet(
        packet_sequence: u64,
        packet_source_port: String,
        packet_source_channel: String,
        packet_destination_port: String,
        packet_destination_channel: String,
        packet_data: vector<u8>,
        packet_timeout_revision_num: u64,
        packet_timeout_revision_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires RelayStore, SignerRef {
        let timeout_height = height::new(packet_timeout_revision_num, packet_timeout_revision_height);
        let packet = IBC::packet::new(
            packet_sequence,
            packet_source_port,
            packet_source_channel,
            packet_destination_port,
            packet_destination_channel,
            packet_data,
            height::new(packet_timeout_revision_num, packet_timeout_revision_height),
            packet_timeout_timestamp,
        );

        ibc::recv_packet(
            &get_signer(),
            get_self_address(),
            packet,
            proof,
            height::new(proof_height_revision_num, proof_height_revision_height),
            vector[1]
        );
        
        on_recv_packet_processing(packet);
    }

    public entry fun acknowledge_packet(
        packet_sequence: u64,
        packet_source_port: String,
        packet_source_channel: String,
        packet_destination_port: String,
        packet_destination_channel: String,
        packet_data: vector<u8>,
        packet_timeout_revision_num: u64,
        packet_timeout_revision_height: u64,
        packet_timeout_timestamp: u64,
        acknowledgement: vector<u8>,
        proof: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires RelayStore, SignerRef {
        let timeout_height = height::new(packet_timeout_revision_num, packet_timeout_revision_height);
        let proof_height = height::new(proof_height_revision_num, proof_height_revision_height);

        let packet = IBC::packet::new(
            packet_sequence,
            packet_source_port,
            packet_source_channel,
            packet_destination_port,
            packet_destination_channel,
            packet_data,
            timeout_height,
            packet_timeout_timestamp,
        );

        ibc::acknowledge_packet(
            &get_signer(),
            get_self_address(),
            packet,
            acknowledgement,
            proof,
            proof_height,
        );
        
        if (vector::length(&acknowledgement) != ACK_LENGTH || (*vector::borrow(&acknowledgement, 0) != ACK_FAILURE && *vector::borrow(&acknowledgement, 0) != ACK_SUCCESS)) {
            abort E_INVALID_ACKNOWLEDGEMENT
        };

        if (*vector::borrow(&acknowledgement, 0) == ACK_FAILURE) {
            let relay_packet = decode_packet(*IBC::packet::data(&packet));
            refund_tokens(IBC::packet::sequence(&packet), *IBC::packet::source_channel(&packet), &relay_packet);
        };
    }



    public entry fun send(
        sender: &signer,
        source_channel: String,
        receiver: address,
        denom_list: vector<address>, 
        amount_list: vector<u64>, 
        extension: String,
        timeout_height_number: u64,
        timeout_height_height: u64,
        timeout_timestamp: u64
    ) acquires RelayStore, SignerRef {
        let num_tokens = vector::length(&denom_list);
        
        if(vector::length(&amount_list) != num_tokens) {
            abort E_INVALID_BYTES_ADDRESS
        };

        let normalized_tokens: vector<Token> = vector::empty<Token>();

        let i = 0;
        while (i < num_tokens) {
            let local_token_denom = *vector::borrow(&denom_list, i);
            let local_token_amount = *vector::borrow(&amount_list, i);
            
            let token_address = send_token(
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

        let timeout_height = height::new(timeout_height_number, timeout_height_height);

        let packet_sequence = IBC::ibc::send_packet(
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
            
            event::emit(Sent {
                packet_sequence: packet_sequence,
                channel_id: source_channel,
                sender: bcs::to_bytes(&signer::address_of(sender)),
                receiver: receiver,
                denom: normalizedToken.denom,
                token: local_token_denom,
                amount: local_token_amount
            });

        }
    }

     public fun send_token(
        sender: &signer,
        source_channel: String,
        denom: address,
        amount: u64,
    ): String acquires RelayStore {
        if (amount == 0) {
            abort E_INVALID_AMOUNT
        };

        let store = borrow_global<RelayStore>(get_vault_addr());

        let pair = AddressToDenomPair { source_channel, denom };
        let token_address = *smart_table::borrow_with_default(&store.address_to_denom, pair, &string::utf8(b""));

        let token = get_metadata(denom);
        if (string::length(&token_address) == 0) {
            // transferring to the zero address is basically burning
            primary_fungible_store::transfer(sender, token, @zero_account, amount);
        } else {
            primary_fungible_store::transfer(sender, token, @UCS01, amount);
            increase_outstanding(source_channel, denom, amount);
            token_address = string_utils::to_string_with_canonical_addresses(&denom);
        };
        token_address
    }

    #[test]
    public fun decode_test() {
        let relay = decode_packet(x"000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002a000000000000000000000000000000000000000000000000000000000000000030102030000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000003e80000000000000000000000000000000000000000000000000000000000000bb800000000000000000000000000000000000000000000000000000000000000064141414141410000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000007d00000000000000000000000000000000000000000000000000000000000000fa0000000000000000000000000000000000000000000000000000000000000000442424242000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        std::debug::print(&relay);
    }
    
    #[test]
    public fun test_is_valid_version() {
        let valid_version = string::utf8(b"ucs01-relay-1");
        let invalid_version = string::utf8(b"invalid-version");
        
        // Test with valid version
        assert!(is_valid_version(valid_version), 100);

        // Test with invalid version
        assert!(!is_valid_version(invalid_version), 101);
    }


    #[test]
    public fun test_is_from_channel() {
        let port_id = string::utf8(b"port-1");
        let channel_id = string::utf8(b"channel-1");
        let valid_denom = string::utf8(b"port-1/channel-1/denom");
        let invalid_denom = string::utf8(b"other-port/other-channel/denom");
        
        // Test with valid denom
        assert!(is_from_channel(port_id, channel_id, valid_denom), 200);

        // Test with invalid denom
        assert!(!is_from_channel(port_id, channel_id, invalid_denom), 201);
    }

    #[test]
    public fun test_make_denom_prefix() {
        let port_id = string::utf8(b"port-1");
        let channel_id = string::utf8(b"channel-1");
        let expected_prefix = string::utf8(b"port-1/channel-1/");
        
        let result = make_denom_prefix(port_id, channel_id);
        assert!(result == expected_prefix, 300);
    }

    #[test]
    public fun test_make_foreign_denom() {
        let port_id = string::utf8(b"port-1");
        let channel_id = string::utf8(b"channel-1");
        let denom = string::utf8(b"denom");
        let expected_foreign_denom = string::utf8(b"port-1/channel-1/denom");
        
        let result = make_foreign_denom(port_id, channel_id, denom);
        assert!(result == expected_foreign_denom, 400);
    }

    #[test(admin = @UCS01)]
    public fun test_get_denom_address(admin: &signer) acquires RelayStore {
        // Initialize the store in the admin's account
        init_module(admin);

        let source_channel = string::utf8(b"channel-1");
        let denom = string::utf8(b"denom-1");
        let expected_address: address = @0x1;

        let pair = DenomToAddressPair {
            source_channel,
            denom,
        };
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.denom_to_address, pair, expected_address);
    
        // Test getting the address
        let result = get_denom_address(source_channel, denom);
        assert!(result == expected_address, 500);
    }

    #[test(admin = @UCS01)]    
    public fun test_get_outstanding(admin: &signer) acquires RelayStore {
        // Initialize the store in the admin's account
        init_module(admin);

        let source_channel = string::utf8(b"channel-1");
        let token = @0x1;
        let expected_amount: u64 = 1000;

        // Set up the mapping in the Relay module (this is usually done through an entry function)
        let pair = OutstandingPair {
            source_channel: source_channel,
            token: token,
        };
        
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.outstanding, pair, expected_amount);

        // Test getting the outstanding amount
        let result = get_outstanding(source_channel, token);
        assert!(result == expected_amount, 600);
    }
    
    #[test(admin = @UCS01)]    
    public fun test_increase_outstanding(admin: &signer) acquires RelayStore {
        // Initialize the store
        let source_channel = string::utf8(b"channel-1");
        let token_address: address = @0x1;
        let initial_amount: u64 = 1000;
        
        // Initialize the store in the admin's account
        init_module(admin);

        // Increase outstanding amount
        increase_outstanding(source_channel, token_address, initial_amount);

        // Verify that the outstanding amount is updated correctly
        let outstanding_amount = get_outstanding(source_channel, token_address);
        assert!(outstanding_amount == initial_amount, 700);
    }

    #[test(admin = @UCS01)]    
    public fun test_decrease_outstanding(admin: &signer) acquires RelayStore {
        // Initialize the store
        let source_channel = string::utf8(b"channel-1");
        let token_address: address = @0x1;
        let initial_amount: u64 = 1000;
        let decrease_amount: u64 = 400;

        // Initialize the store in the admin's account
        init_module(admin);

        // First, increase outstanding amount
        increase_outstanding(source_channel, token_address, initial_amount);

        // Decrease the outstanding amount
        decrease_outstanding(source_channel, token_address, decrease_amount);

        // Verify that the outstanding amount is updated correctly
        let outstanding_amount = get_outstanding(source_channel, token_address);
        let expected_amount = initial_amount - decrease_amount;
        assert!(outstanding_amount == expected_amount, 701);
    }

    const TEST_NAME: vector<u8> = b"Test Coin";
    const TEST_SYMBOL: vector<u8> = b"TST";
    const TEST_DECIMALS: u8 = 8;
    const TEST_ICON: vector<u8> = b"https://example.com/icon.png";
    const TEST_PROJECT: vector<u8> = b"Test Project";



    #[test(admin = @UCS01, bob = @0x1235)]
    public fun test_send_token_valid_address(admin: &signer, bob: &signer) acquires RelayStore, SignerRef {
        // Initialize the store
        init_module(admin);

        let source_channel = string::utf8(b"channel-1");
        let denom = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        let amount: u64 = 1000;

        // Upsert denom to address pair
        let pair = AddressToDenomPair {
            source_channel,
            denom,
        };

        let new_denom = string::utf8(b"new-denom");
        let denom_str = string_utils::to_string_with_canonical_addresses(&denom);
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.address_to_denom, pair, new_denom);
        let admin = &get_signer();

        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            TEST_SYMBOL
        );

        let asset_addr = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        let asset = get_metadata(asset_addr);
        let bob_addr = signer::address_of(bob);
        UCS01::fa_coin::mint_with_metadata(admin, bob_addr, amount, asset);

        // Send tokens
        let result_address = send_token(bob, source_channel, asset_addr, amount);

        // Verify the result and outstanding balance
        assert!(result_address == denom_str, 100);
        let outstanding_balance = get_outstanding(source_channel, denom);
        assert!(outstanding_balance == amount, 101);

        let bob_balance = primary_fungible_store::balance(bob_addr, asset);
        let ucs01_balance = primary_fungible_store::balance(@UCS01, asset);
        assert!(bob_balance == 0, 102);
        assert!(ucs01_balance == 1000, 102);
    }

    #[test(admin = @UCS01, bob = @0x1235)]
    public fun test_send_token_burn(admin: &signer, bob: &signer) acquires RelayStore, SignerRef {
        // Initialize the store
        init_module(admin);

        let source_channel = string::utf8(b"channel-1");
        let denom = @0x111111;
        let amount: u64 = 1000;

        // Upsert denom to address pair
        let _pair = AddressToDenomPair {
            source_channel,
            denom,
        };
        let admin = &get_signer();
        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            TEST_SYMBOL
        );

        
        let asset_addr = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        let asset = get_metadata(asset_addr);
        let bob_addr = signer::address_of(bob);
        UCS01::fa_coin::mint_with_metadata(admin, bob_addr, amount, asset);

        // Send tokens
        let result_address = send_token(bob, source_channel, asset_addr, amount);

        // Verify the result and outstanding balance
        assert!(string::length(&result_address) == 0, 100); 
        let outstanding_balance = get_outstanding(source_channel, denom);
        assert!(outstanding_balance == 0, 101);

        let bob_balance = primary_fungible_store::balance(bob_addr, asset);
        let ucs01_balance = primary_fungible_store::balance(@UCS01, asset);
        assert!(bob_balance == 0, 102);
        assert!(ucs01_balance == 0, 102);
    }

    #[test(admin = @UCS01)]
    #[expected_failure(abort_code = E_INVALID_AMOUNT)]
    public fun test_send_zero_amount(admin: &signer) acquires RelayStore, SignerRef {
        // Initialize the store
        init_module(admin);

        let source_channel = string::utf8(b"channel-1");
        let _denom = @0x111111;
        let admin = &get_signer();
        UCS01::fa_coin::initialize(
            admin,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            TEST_SYMBOL
        );

        
        let asset_addr = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        let _asset = get_metadata(asset_addr);

        // Attempt to send zero amount
        send_token(admin, source_channel, asset_addr, 0);
    }

    #[test]
    public fun test_encode() {
        let token = Token {
            denom: string::utf8(b"denom"),
            amount: 1000,
        };
        let token2 = Token {
            denom: string::utf8(b"hebelelelele"),
            amount: 3000,
        };
        let token3 = Token {
            denom: string::utf8(b"weweweweewew"),
            amount: 3,
        };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);
        vector::push_back(&mut tokens, token2);
        vector::push_back(&mut tokens, token3);

        let sender = bcs::to_bytes(&@0x111111111111111111111);
        let receiver = @0x0000000000000000000000000000000000000033;
        let extension = string::utf8(b"extension");
        let packet = RelayPacket {
            sender: sender,
            receiver: receiver,
            tokens: tokens,
            extension: extension,
        };
        let encoded = encode_packet(&packet);
        std::debug::print(&encoded);
        let decoded = decode_packet(encoded);

        assert!(decoded.sender == sender, 100);
        assert!(decoded.receiver == receiver, 101);
        assert!(decoded.extension == extension, 102);
        let token = vector::borrow(&decoded.tokens, 0);
        assert!(token.denom == string::utf8(b"denom"), 103);
        assert!(token.amount == 1000, 104);
        let token2 = vector::borrow(&decoded.tokens, 1);
        assert!(token2.denom == string::utf8(b"hebelelelele"), 105);
        assert!(token2.amount == 3000, 106);
        let token3 = vector::borrow(&decoded.tokens, 2);
        assert!(token3.denom == string::utf8(b"weweweweewew"), 107);
        assert!(token3.amount == 3, 108);
    }



    #[test(admin = @UCS01, alice = @0x1234)]
    public fun test_refund_tokens(admin: &signer, alice: address) acquires RelayStore, SignerRef {
        init_module(admin);

        let source_channel = string::utf8(b"channel-1");
        let amount: u64 = 1000;

        let token_owner = &get_signer();

        // Step 2: Mint some tokens to Alice
        UCS01::fa_coin::initialize(
            token_owner,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            TEST_SYMBOL
        );

        let asset_addr = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        let asset = get_metadata(asset_addr);

        // Step 3: Simulate sending tokens (for refund purposes)
        let token = Token {
            denom: string_utils::to_string_with_canonical_addresses(&asset_addr),
            amount: amount,
        };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);

        let relay_packet = RelayPacket {
            sender: bcs::to_bytes(&alice),
            receiver: @0x0000000000000000000000000000000000000022,
            tokens: tokens,
            extension: string::utf8(b"extension"),
        };

        // Insert mapping for the denom -> address
        let pair = DenomToAddressPair {
            source_channel,
            denom: string_utils::to_string_with_canonical_addresses(&asset_addr),
        };
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.denom_to_address, pair, asset_addr);

        // Step 4: Call the refund function
        let sequence = 1;
        refund_tokens(sequence, source_channel, &relay_packet);

        // Step 5: Verify the results
        let alice_balance = primary_fungible_store::balance(alice, asset);
        assert!(alice_balance == amount, 100);  // Alice should have received the refund

    }

    
    #[test(admin = @UCS01, alice = @0x1234)]
    public fun test_refund_tokens_zero_address(admin: &signer, alice: address) acquires RelayStore, SignerRef {
        init_module(admin);

        let source_channel = string::utf8(b"channel-1");
        let amount: u64 = 1000;

        let token_owner = &get_signer();

        UCS01::fa_coin::initialize(
            token_owner,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            TEST_SYMBOL
        );
        let asset_addr = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);

        let asset = get_metadata(asset_addr);

        UCS01::fa_coin::mint_with_metadata(token_owner, signer::address_of(token_owner), 1000, asset);

        

        // Step 3: Simulate sending tokens (for refund purposes)
        let token = Token {
            denom: string_utils::to_string_with_canonical_addresses(&asset_addr),
            amount: amount,
        };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);

        let relay_packet = RelayPacket {
            sender: bcs::to_bytes(&alice),
            receiver: @0x0000000000000000000000000000000000000022,
            tokens: tokens,
            extension: string::utf8(b"extension"),
        };
        increase_outstanding(source_channel, asset_addr, amount);

        let outstanding_balance = get_outstanding(source_channel, asset_addr);
        assert!(outstanding_balance == 1000, 200);  // The outstanding balance should be reduced to 0.

        let sequence = 1;
        refund_tokens(sequence, source_channel, &relay_packet);

        let outstanding_balance = get_outstanding(source_channel, asset_addr);
        assert!(outstanding_balance == 0, 200);  // The outstanding balance should be reduced to 0.

        let alice_balance = primary_fungible_store::balance(alice, asset);
        assert!(alice_balance == 0, 201);  // Alice should not receive any tokens in this case.
    }


    #[test(admin = @UCS01, alice = @0x1234, bob = @0x1235)]
    public fun test_on_recv_packet_processing_local_token(admin: &signer, alice: address, bob: address) acquires RelayStore, SignerRef {
        // Step 1: Initialize the store
        init_module(admin);
        
        // Step 2: Setup the initial mappings for local tokens
        let source_channel = string::utf8(b"channel-1");
        let destination_channel = string::utf8(b"dest-channel");
        let port_id = string::utf8(b"port-1");
        let local_token_address = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        
        let token_owner = &get_signer();

        // Mint some tokens to simulate the token creation on this chain
        UCS01::fa_coin::initialize(
            token_owner,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            TEST_SYMBOL
        );

        let asset_addr = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);

        let asset = get_metadata(asset_addr);
        // Step 3: Mint tokens to the relay module's account (escrow)
        UCS01::fa_coin::mint_with_metadata(token_owner, signer::address_of(token_owner), 1000, asset);
        increase_outstanding(source_channel, local_token_address, 1000);

        // Step 4: Create the RelayPacket with a local token
        let new_denom = make_denom_prefix(port_id, source_channel);
        string::append(&mut new_denom, string_utils::to_string_with_canonical_addresses(&local_token_address));
        let token = Token {
            denom: new_denom,
            amount: 500,
        };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);

        let relay_packet = RelayPacket {
            sender: bcs::to_bytes(&alice),
            receiver: bob,
            tokens: tokens,
            extension: string::utf8(b""),
        };

        // Step 5: Create the IBC packet
        let ibc_packet = IBC::packet::new(
            1,  // sequence
            port_id,
            source_channel,
            port_id,
            destination_channel,
            encode_packet(&relay_packet),
            height::new(1, 100),
            1000000
        );

        // Step 6: Process the IBC packet
        on_recv_packet_processing(ibc_packet);

        // Step 7: Verify the token was transferred to Bob
        let bob_balance = primary_fungible_store::balance(bob, get_metadata(local_token_address));
        assert!(bob_balance == 500, 100); // Bob should have received the token

        // Step 8: Verify the outstanding amount was decreased
        let outstanding_balance = get_outstanding(source_channel, local_token_address);
        assert!(outstanding_balance == 500, 101); // Outstanding should be reduced by 500
    }
    #[test(admin = @UCS01, alice = @0x1234, bob = @0x1235)]
    public fun test_on_recv_packet_processing_foreign_token_denom_address_zero(admin: &signer, alice: address, bob: address) acquires RelayStore, SignerRef {
        // Step 1: Initialize the store
        init_module(admin);

        // Step 2: Set up mappings and mint tokens on the counterparty chain
        let source_channel = string::utf8(b"channel-1");
        let destination_channel = string::utf8(b"dest-channel");
        let port_id = string::utf8(b"port-1");

        // Step 3: Create a RelayPacket with a foreign token denomination
        let foreign_denom = string::utf8(b"foreign-token-denom");
        let token = Token {
            denom: foreign_denom,
            amount: 500,
        };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);

        let relay_packet = RelayPacket {
            sender: bcs::to_bytes(&alice),
            receiver: bob,
            tokens: tokens,
            extension: string::utf8(b""),
        };

        // Step 4: Create the IBC packet
        let ibc_packet = IBC::packet::new(
            1,  // sequence
            port_id,
            source_channel,
            port_id,
            destination_channel,
            encode_packet(&relay_packet),
            height::new(1, 100),
            1000000
        );

        // Step 5: Process the IBC packet (will enter the 'else' block since it's a foreign token)
        on_recv_packet_processing(ibc_packet);

        // Step 6: Verify that a new denomination was created and minting occurred
        let _store = borrow_global_mut<RelayStore>(get_vault_addr());
        // Construct the foreign denomination using the source and destination channels
        let our_denom = make_foreign_denom(
            port_id,
            destination_channel,
            foreign_denom
        );
        let denom_address = get_denom_address(source_channel, our_denom);
        assert!(denom_address != @0x0, 100); // The new token address should have been created

        // Step 7: Verify the token was minted to Bob's account
        let bob_balance = primary_fungible_store::balance(bob, get_metadata(denom_address));
        assert!(bob_balance == 500, 101); // Bob should have received the minted foreign token
    }

    #[test(admin = @UCS01, alice = @0x1234, bob = @0x1235)]
    public fun test_on_recv_packet_processing_foreign_token_existing_denom_address(admin: &signer, alice: address, bob: address) acquires RelayStore, SignerRef {
        // Step 1: Initialize the store
        init_module(admin);

        // Step 2: Set up mappings and mint tokens on the counterparty chain
        let source_channel = string::utf8(b"channel-1");
        let destination_channel = string::utf8(b"dest-channel");
        let port_id = string::utf8(b"port-1");
        let _foreign_denom = string::utf8(b"foreign-token-denom");
        let token_owner = &get_signer();

        UCS01::fa_coin::initialize(
            token_owner,
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            TEST_SYMBOL
        );
        let asset_addr = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        let _asset = get_metadata(asset_addr);

        let new_denom = make_denom_prefix(port_id, destination_channel);
        string::append(&mut new_denom, string_utils::to_string_with_canonical_addresses(&asset_addr));
        // Step 3: Insert an existing foreign token address into the store
        let _existing_denom_address: address = @0x1;
        let pair = DenomToAddressPair {
            source_channel: source_channel,
            denom: new_denom,
        };
            
        let store = borrow_global_mut<RelayStore>(get_vault_addr());
        smart_table::upsert(&mut store.denom_to_address, pair, asset_addr);

        // Step 4: Create a RelayPacket with the foreign token denomination
        let token = Token {
            denom: string_utils::to_string_with_canonical_addresses(&asset_addr),
            amount: 500,
        };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);

        let relay_packet = RelayPacket {
            sender: bcs::to_bytes(&alice),
            receiver: bob,
            tokens: tokens,
            extension: string::utf8(b""),
        };

        // Step 5: Create the IBC packet
        let ibc_packet = IBC::packet::new(
            1,  // sequence
            port_id,
            source_channel,
            port_id,
            destination_channel,
            encode_packet(&relay_packet),
            height::new(1, 100),
            1000000
        );

        // Step 6: Process the IBC packet (will use the existing token address in the 'else' block)
        on_recv_packet_processing(ibc_packet);

        // Step 7: Verify that the existing token address was used

        let denom_address = get_denom_address(source_channel, new_denom);
        assert!(denom_address == asset_addr, 100); // It should be the same as the existing address

        // Step 8: Verify the token was minted to Bob's account
        let bob_balance = primary_fungible_store::balance(bob, get_metadata(asset_addr));
        assert!(bob_balance == 500, 101); // Bob should have received the minted foreign token
    }


    #[test(admin = @UCS01, alice = @0x1234, bob = @0x1235)]
    #[expected_failure(abort_code = E_INVALID_AMOUNT)]
    public fun test_on_recv_packet_processing_local_token_revert_amount_zero(admin: &signer, alice: address, bob: address) acquires RelayStore, SignerRef {
        // Step 1: Initialize the store
        init_module(admin);
        
        // Step 2: Setup the initial mappings for local tokens
        let source_channel = string::utf8(b"channel-1");
        let destination_channel = string::utf8(b"dest-channel");
        let port_id = string::utf8(b"port-1");
        let local_token_address = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        
        // Mint some tokens to simulate the token creation on this chain
        UCS01::fa_coin::initialize(
            &get_signer(),
            string::utf8(TEST_NAME),
            string::utf8(TEST_SYMBOL),
            TEST_DECIMALS,
            string::utf8(TEST_ICON),
            string::utf8(TEST_PROJECT),
            TEST_SYMBOL
        );

        let asset_addr = UCS01::fa_coin::get_metadata_address(TEST_SYMBOL);
        let asset = get_metadata(asset_addr);
        // Step 3: Mint tokens to the relay module's account (escrow)
        UCS01::fa_coin::mint_with_metadata(&get_signer(), @UCS01, 1000, asset);
        increase_outstanding(source_channel, local_token_address, 1000);

        // Step 4: Create the RelayPacket with a local token
        let new_denom = make_denom_prefix(port_id, source_channel);
        string::append(&mut new_denom, string_utils::to_string_with_canonical_addresses(&local_token_address));
        let token = Token {
            denom: new_denom,
            amount: 0,
        };
        let tokens = vector::empty<Token>();
        vector::push_back(&mut tokens, token);

        let relay_packet = RelayPacket {
            sender: bcs::to_bytes(&alice), 
            receiver: bob,
            tokens: tokens,
            extension: string::utf8(b""),
        };

        // Step 5: Create the IBC packet
        let ibc_packet = IBC::packet::new(
            1,  // sequence
            port_id,
            source_channel,
            port_id,
            destination_channel,
            encode_packet(&relay_packet),
            height::new(1, 100),
            1000000
        );

        // Step 6: Process the IBC packet
        on_recv_packet_processing(ibc_packet);

        // Step 7: Verify the token was transferred to Bob
        let bob_balance = primary_fungible_store::balance(bob, get_metadata(local_token_address));
        assert!(bob_balance == 500, 100); // Bob should have received the token

        // Step 8: Verify the outstanding amount was decreased
        let outstanding_balance = get_outstanding(source_channel, local_token_address);
        assert!(outstanding_balance == 500, 101); // Outstanding should be reduced by 500
    }
}
