module ucs03::zkgm_relay {
    use ibc::ibc;
    use ibc::packet::{Self, Packet};
    use std::string::{Self, String, utf8};
    use sui::table::{Self, Table};
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


    // //TODO: Its not working, why?
    // #[test]
    // public fun test_encode() {
    //     let token = Token { denom: string::utf8(b"denom"), amount: 1000, fee: 15 };
    //     let token2 = Token { denom: string::utf8(b"this is amazing"), amount: 3000, fee: 22 };
    //     let token3 = Token { denom: string::utf8(b"insane cool"), amount: 3, fee: 0 };
    //     let mut tokens = vector::empty<Token>();
    //     vector::push_back(&mut tokens, token);
    //     vector::push_back(&mut tokens, token2);
    //     vector::push_back(&mut tokens, token3);

    //     let sender = bcs::to_bytes(&@0x111111111111111111111);
    //     let receiver = bcs::to_bytes(&@0x0000000000000000000000000000000000000033);
    //     let extension = string::utf8(b"extension");
    //     let packet = RelayPacket {
    //         sender: sender,
    //         receiver: receiver,
    //         tokens: tokens,
    //         extension: extension
    //     };
    //     let encoded = encode_packet(&packet);
    //     std::debug::print(&string::utf8(b"encoded packet is: " ));
    //     std::debug::print(&encoded);
    //     let decoded = decode_packet(&encoded);

    //     assert!(decoded.sender == sender, 100);
    //     assert!(decoded.receiver == receiver, 101);
    //     assert!(decoded.extension == extension, 102);
    //     let token = vector::borrow(&decoded.tokens, 0);
    //     assert!(token.denom == string::utf8(b"denom"), 103);
    //     assert!(token.amount == 1000, 104);
    //     let token2 = vector::borrow(&decoded.tokens, 1);
    //     assert!(token2.denom == string::utf8(b"this is amazing"), 105);
    //     assert!(token2.amount == 3000, 106);
    //     let token3 = vector::borrow(&decoded.tokens, 2);
    //     assert!(token3.denom == string::utf8(b"insane cool"), 107);
    //     assert!(token3.amount == 3, 108);
    // }
}