module zkgm::zkgm_relay {
    use ibc::ibc;
    use ibc::packet::{Self, Packet};
    use zkgm::fungible_token::{Self, FUNGIBLE_TOKEN};
    use sui::coin::{Self, Coin, TreasuryCap, CoinMetadata};

    use std::string::{Self, String, utf8};
    use sui::table::{Self, Table};
    use zkgm::zkgm_ethabi;
    use ibc::commitment;
    use sui::bcs;
    use sui::clock;
    use sui::address::{to_string};
    use sui::event;

    // Constants
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";
    const VERSION: vector<u8> = b"zkgm-zkgm-0";
    const ACK_SUCCESS: u256 = 1;
    const ACK_FAILURE: u256 = 0;
    const ACK_LENGTH: u64 = 1;
    const ZKGM_VERSION_0: u8 = 0x00;
    const SYSCALL_FORWARD: u8 = 0x00;
    const SYSCALL_MULTIPLEX: u8 = 0x01;
    const SYSCALL_BATCH: u8 = 0x02;
    const SYSCALL_FUNGIBLE_ASSET_TRANSFER: u8 = 0x03;
    const FILL_TYPE_PROTOCOL: u256 = 0xB0CAD0;
    const FILL_TYPE_MARKETMAKER: u256 = 0xD1CEC45E;
    const ACK_EMPTY: vector<u8> = x"";


    // Errors
    const ACK_ERR_ONLYMAKER: vector<u8> = b"DEADC0DE";
    const E_UNAUTHORIZED: u64 = 1;
    const E_INVALID_HOPS: u64 = 2;
    const E_INVALID_IBC_VERSION: u64 = 3;
    const E_INFINITE_GAME: u64 = 4;
    const E_UNSUPPORTED_VERSION: u64 = 5;
    const E_UNKNOWN_SYSCALL: u64 = 6;
    const E_INVALID_ASSET_NAME: u64 = 7;
    const E_INVALID_ASSET_SYMBOL: u64 = 8;
    const E_INVALID_ASSET_ORIGIN: u64 = 9;
    const E_INVALID_AMOUNT: u64 = 10;
    const E_BATCH_MUST_BE_SYNC: u64 = 11;
    const E_INVALID_FILL_TYPE: u64 = 12;
    const E_UNIMPLEMENTED: u64 = 13;
    const E_ACK_EMPTY: u64 = 14;
    const E_ONLY_MAKER: u64 = 15;
    const E_BATCH_MISMATCH: u64 = 16;

    public struct ZkgmPacket has copy, drop, store {
        salt: vector<u8>,
        path: u256,
        instruction: Instruction
    }

    // struct Instruction has copy, drop, store {
    //     version: u8,
    //     opcode: u8,
    //     operand: vector<u8>
    // }

    public struct Instruction has copy, drop, store {
        version: u8,
        opcode: u8,
        operand: vector<u8>
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
        instruction: Instruction
    }

    public struct MultiplexPacket has copy, drop, store {
        sender: vector<u8>,
        eureka: bool,
        contract_address: vector<u8>,
        contract_calldata: vector<u8>
    }

    #[event]
    public struct OnZkgmCall has copy, drop, store {
        sender: vector<u8>,
        contract_calldata: vector<u8>,
        contract_address: vector<u8>
    }

    public struct Batch has copy, drop, store {
        instructions: vector<Instruction>
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

    #[event]
    public struct OnIIBCModuleOnRecvPacketCall has copy, drop, store {
        packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        contract_address: vector<u8>
    }

    public struct IIBCModuleOnAcknowledgementPacketParams has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address
    }


    #[event]
    public struct OnIIBCModuleOnAcknowledgementPacketCall has copy, drop, store {
        packet: Packet,
        acknowledgement: vector<u8>,
        relayer: address,
        contract_address: vector<u8>
    }

    public struct IIBCModuleOnTimeoutPacketParams has copy, drop, store {
        packet: Packet,
        relayer: address
    }

    #[event]
    public struct OnIIBCModuleOnTimeoutPacketCall has copy, drop, store {
        packet: Packet,
        relayer: address,
        contract_address: vector<u8>
    }

    public struct FungibleAssetTransferPacket has copy, drop, store {
        sender: vector<u8>,
        receiver: vector<u8>,
        sent_token: address,
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
        token_origin: Table<address, u256>,
        address_to_treasurycap: Table<address, TreasuryCap<FUNGIBLE_TOKEN>>,
        // address_to_coin: Table<address, Coin<FUNGIBLE_TOKEN>>

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
            token_origin: table::new(ctx),
            address_to_treasurycap: table::new(ctx),
            // address_to_coin: table::new(ctx)
        });
    }
    fun get_treasury_cap_mut(relay_store: &mut RelayStore, denom_address: address): &mut TreasuryCap<FUNGIBLE_TOKEN> {
        relay_store.address_to_treasurycap.borrow_mut(denom_address)
    }

    // fun get_coin_mut(relay_store: &mut RelayStore, denom_address: address): &mut Coin<FUNGIBLE_TOKEN> {
    //     relay_store.address_to_coin.borrow_mut(denom_address)
    // }


    // public entry fun insert_pair(
    //     relay_store: &mut RelayStore,
    //     denom_address: address,
    //     treasury_cap: TreasuryCap<FUNGIBLE_TOKEN>,
    //     coin: Coin<FUNGIBLE_TOKEN>
    // ) {
    //     relay_store.address_to_treasurycap.add(denom_address, treasury_cap);
    //     relay_store.address_to_coin.add(denom_address, coin);
    // }

    public fun decode_ack(buf: vector<u8>): Acknowledgement {
        let mut index = 0x20;
        let tag = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let inner_ack =
            zkgm_ethabi::decode_vector!<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (zkgm_ethabi::decode_uint(buf, index) as u8)
                }
            );

        Acknowledgement { tag: tag, inner_ack: inner_ack }
    }

    public fun encode_ack(packet: &Acknowledgement): vector<u8> {
        let mut buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_uint<u256>(&mut buf, packet.tag);

        let version_offset = 0x40;
        zkgm_ethabi::encode_uint<u32>(&mut buf, version_offset);

        zkgm_ethabi::encode_vector!<u8>(
            &mut buf,
            &packet.inner_ack,
            |some_variable, data| {
                zkgm_ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun encode_asset_transfer_ack(
        ack: &AssetTransferAcknowledgement
    ): vector<u8> {
        let mut buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_uint<u256>(&mut buf, ack.fill_type);

        let version_offset = 0x40;
        zkgm_ethabi::encode_uint<u32>(&mut buf, version_offset);

        zkgm_ethabi::encode_vector!<u8>(
            &mut buf,
            &ack.market_maker,
            |some_variable, data| {
                zkgm_ethabi::encode_uint<u8>(some_variable, *data);
            }
        );

        buf
    }

    public fun decode_asset_transfer_ack(buf: vector<u8>): AssetTransferAcknowledgement {
        let mut index = 0x20;
        let fill_type = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let market_maker =
            zkgm_ethabi::decode_vector!<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (zkgm_ethabi::decode_uint(buf, index) as u8)
                }
            );

        AssetTransferAcknowledgement { fill_type: fill_type, market_maker: market_maker }
    }

    public fun decode_batch_ack(buf: vector<u8>): BatchAcknowledgement {
        let mut index = 0x40;
        let main_arr_length = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + (0x20 * main_arr_length as u64);

        let mut idx = 0;
        let mut acknowledgements = vector::empty();
        while (idx < main_arr_length) {
            let inner_vec =
                zkgm_ethabi::decode_vector!<u8>(
                    &buf,
                    &mut index,
                    |buf, index| {
                        (zkgm_ethabi::decode_uint(buf, index) as u8)
                    }
                );
            vector::push_back(&mut acknowledgements, inner_vec);
            idx = idx + 1;
        };

        BatchAcknowledgement { acknowledgements: acknowledgements }
    }

    public fun encode_batch_ack(ack: &BatchAcknowledgement): vector<u8> {
        let mut buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        let ack_arr_len = vector::length(&ack.acknowledgements);
        zkgm_ethabi::encode_uint<u64>(&mut buf, ack_arr_len);
        if (ack_arr_len < 2) {
            if (ack_arr_len == 1) {
                zkgm_ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
                zkgm_ethabi::encode_vector!<u8>(
                    &mut buf,
                    vector::borrow(&ack.acknowledgements, 0),
                    |some_variable, data| {
                        zkgm_ethabi::encode_uint<u8>(some_variable, *data);
                    }
                );
                return buf
            };
            return buf
        };

        let initial_stage = 0x20 * (ack_arr_len as u32);
        let mut idx = 1;
        let mut prev_val = initial_stage;
        zkgm_ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
        while (idx < ack_arr_len) {
            let prev_length = vector::length(
                vector::borrow(&ack.acknowledgements, idx - 1)
            );
            zkgm_ethabi::encode_uint<u32>(&mut buf, prev_val
                + 0x20 * (prev_length + 1 as u32));
            prev_val = prev_val + 0x20 * (prev_length + 1 as u32);
            idx = idx + 1;
        };
        idx = 0;
        while (idx < ack_arr_len) {
            zkgm_ethabi::encode_vector!<u8>(
                &mut buf,
                vector::borrow(&ack.acknowledgements, idx),
                |some_variable, data| {
                    zkgm_ethabi::encode_uint<u8>(some_variable, *data);
                }
            );
            idx = idx + 1;
        };

        buf
    }

    public fun decode_batch_packet(buf: vector<u8>): Batch {
        let mut index = 0x20;
        let main_arr_length = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + (0x20 * main_arr_length as u64);

        let mut idx = 0;
        let mut instructions = vector::empty<Instruction>();
        while (idx < main_arr_length) {
            let version = (zkgm_ethabi::decode_uint(&buf, &mut index) as u8);
            let opcode = (zkgm_ethabi::decode_uint(&buf, &mut index) as u8);
            index = index + 0x20;
            let operand = zkgm_ethabi::decode_bytes(&buf, &mut index);

            let instruction = Instruction {
                version: (version as u8),
                opcode: (opcode as u8),
                operand: operand
            };

            vector::push_back(&mut instructions, instruction);
            idx = idx + 1;
        };

        Batch { instructions: instructions }
    }

    public fun encode_batch_packet(pack: &Batch): vector<u8> {
        let mut buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);

        let ack_arr_len = vector::length(&pack.instructions);
        zkgm_ethabi::encode_uint<u64>(&mut buf, ack_arr_len);
        if (ack_arr_len < 2) {
            if (ack_arr_len == 1) {
                zkgm_ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
                let instructions_encoded =
                    encode_instruction(*vector::borrow(&pack.instructions, 0));
                vector::append(&mut buf, instructions_encoded);
                return buf
            };
            return buf
        };

        let initial_stage = 0x20 * (ack_arr_len as u32);
        let mut idx = 1;
        let mut prev_val = initial_stage;
        zkgm_ethabi::encode_uint<u32>(&mut buf, 0x20 * (ack_arr_len as u32));
        while (idx < ack_arr_len) {
            let prev_length =
                ((
                    vector::length(&vector::borrow(&pack.instructions, idx - 1).operand)
                        / 32
                ) as u32) + 1;
            zkgm_ethabi::encode_uint<u32>(
                &mut buf,
                prev_val + (0x20 * 4) + ((prev_length * 0x20) as u32)
            );
            prev_val = prev_val + (4 * 0x20) + (((prev_length * 0x20) as u32));
            idx = idx + 1;
        };
        idx = 0;
        while (idx < ack_arr_len) {
            let instructions_encoded =
                encode_instruction(*vector::borrow(&pack.instructions, idx));
            vector::append(&mut buf, instructions_encoded);
            idx = idx + 1;
        };

        buf
    }

    public fun decode_syscall(buf: vector<u8>): SyscallPacket {
        let mut index = 0x20;
        let version = zkgm_ethabi::decode_uint(&buf, &mut index);
        let mut index_syscall = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;

        let packet =
            zkgm_ethabi::decode_vector!<u8>(
                &buf,
                &mut index,
                |buf, index| {
                    (zkgm_ethabi::decode_uint(buf, index) as u8)
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
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_uint<u32>(&mut buf, packet.channel_id);
        zkgm_ethabi::encode_uint<u64>(&mut buf, packet.timeout_height);
        zkgm_ethabi::encode_uint<u64>(&mut buf, packet.timeout_timestamp);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x80);
        let ins_buf = encode_instruction(packet.instruction);
        vector::append(&mut buf, ins_buf);
        buf
    }

    public fun decode_forward(buf: vector<u8>): ForwardPacket {
        let mut index = 0x20;
        let channel_id = zkgm_ethabi::decode_uint(&buf, &mut index);
        let timeout_height = zkgm_ethabi::decode_uint(&buf, &mut index);
        let timeout_timestamp = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;

        let version = (zkgm_ethabi::decode_uint(&buf, &mut index) as u8);
        let opcode = (zkgm_ethabi::decode_uint(&buf, &mut index) as u8);
        index = index + 0x20;
        let operand = zkgm_ethabi::decode_bytes(&buf, &mut index);

        let instruction = Instruction { version: version, opcode: opcode, operand: operand };

        ForwardPacket {
            channel_id: (channel_id as u32),
            timeout_height: (timeout_height as u64),
            timeout_timestamp: (timeout_timestamp as u64),
            instruction: instruction
        }
    }

    public fun decode_multiplex(buf: vector<u8>): MultiplexPacket {
        let mut index = 0x40;
        let eureka = zkgm_ethabi::decode_uint(&buf, &mut index) == 1;
        index = index + 0x20 * 2;
        let sender = zkgm_ethabi::decode_bytes(&buf, &mut index);
        let contract_address = zkgm_ethabi::decode_bytes(&buf, &mut index);
        let contract_calldata = zkgm_ethabi::decode_bytes(&buf, &mut index);

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
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x40);
        let length_of_first = vector::length(&sender);
        zkgm_ethabi::encode_uint<u64>(&mut buf, ((length_of_first / 32) * 0x20) + 0x80);
        zkgm_ethabi::encode_bytes(&mut buf, &sender);
        zkgm_ethabi::encode_bytes(&mut buf, &contract_calldata);
        buf
    }

    public fun decode_fungible_asset_transfer(buf: vector<u8>): FungibleAssetTransferPacket {
        let mut index = 0x80;
        let sent_token_prefix = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + 0x40;
        let sent_amount = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let ask_amount = zkgm_ethabi::decode_uint(&buf, &mut index);
        let only_maker = (zkgm_ethabi::decode_uint(&buf, &mut index) == 1);
        let sender = zkgm_ethabi::decode_bytes(&buf, &mut index);
        let receiver = zkgm_ethabi::decode_bytes(&buf, &mut index);
        let sent_token = zkgm_ethabi::decode_address(&buf, &mut index);
        let sent_symbol = zkgm_ethabi::decode_string(&buf, &mut index);
        let sent_name = zkgm_ethabi::decode_string(&buf, &mut index);
        let ask_token = zkgm_ethabi::decode_bytes(&buf, &mut index);

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

    public fun encode_instruction(instruction: Instruction): vector<u8> {
        let mut buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, instruction.version);
        zkgm_ethabi::encode_uint<u8>(&mut buf, instruction.opcode);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x60);
        zkgm_ethabi::encode_bytes(&mut buf, &instruction.operand);

        buf
    }


    public fun encode_packet(packet: &ZkgmPacket): vector<u8> {
        let mut buf = vector::empty<u8>();
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x20);
        zkgm_ethabi::encode_bytes32(&mut buf, &packet.salt);
        zkgm_ethabi::encode_uint<u256>(&mut buf, packet.path);
        zkgm_ethabi::encode_uint<u8>(&mut buf, 0x60);

        let ins_buf = encode_instruction(packet.instruction);
        vector::append(&mut buf, ins_buf);

        buf
    }

    public fun decode_packet(buf: vector<u8>): ZkgmPacket {
        let mut index = 0x20;
        let salt = zkgm_ethabi::decode_bytes32(&buf, &mut index);
        let path = zkgm_ethabi::decode_uint(&buf, &mut index);
        index = index + 0x20;
        let version = (zkgm_ethabi::decode_uint(&buf, &mut index) as u8);
        let opcode = (zkgm_ethabi::decode_uint(&buf, &mut index) as u8);
        index = index + 0x20;
        let operand = zkgm_ethabi::decode_bytes(&buf, &mut index);

        let instruction = Instruction { version: version, opcode: opcode, operand: operand };

        ZkgmPacket { salt: salt, path: path, instruction: instruction }
    }
    
    public fun is_valid_version(version_bytes: String): bool {
        version_bytes == string::utf8(VERSION)
    }
    public fun update_channel_path(path: u256, next_channel_id: u32): u256 {
        if (path == 0) {
            return (next_channel_id as u256)
        };
        let next_hop_index = ((fls(path) / 32) as u8) + 1;
        if (next_hop_index > 7) {
            abort E_INVALID_HOPS
        };

        let next_channel = (((next_channel_id as u256) << (next_hop_index * 32)) as u256)
            | path;
        (next_channel as u256)
    }


        /// Find last set (most significant bit).
    /// Returns the index of the most significant bit of `x`.
    /// If `x` is zero, returns 256.
    public fun fls(mut x: u256): u256 {
        if (x == 0) {
            return 256
        };

        let mut r: u256 = 0;

        // Check higher 128 bits
        if (x > 0xffffffffffffffffffffffffffffffff) {
            r = 128;
            x = x >> 128;
        };

        // Check higher 64 bits
        if (x > 0xffffffffffffffff) {
            r = r + 64;
            x = x >> 64;
        };

        // Check higher 32 bits
        if (x > 0xffffffff) {
            r = r + 32;
            x = x >> 32;
        };

        // Check higher 16 bits
        if (x > 0xffff) {
            r = r + 16;
            x = x >> 16;
        };

        // Check higher 8 bits
        if (x > 0xff) {
            r = r + 8;
            x = x >> 8;
        };

        // Check higher 4 bits
        if (x > 0xf) {
            r = r + 4;
            x = x >> 4;
        };

        // Check higher 2 bits
        if (x > 0x3) {
            r = r + 2;
            x = x >> 2;
        };

        // Check higher 1 bit
        if (x > 0x1) {
            r = r + 1;
        };

        r
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
            abort E_INVALID_IBC_VERSION
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
            abort E_INVALID_IBC_VERSION
        };

        if (!is_valid_version(counterparty_version)) {
            abort E_INVALID_IBC_VERSION
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
            abort E_INVALID_IBC_VERSION
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
        abort E_INFINITE_GAME
    }

    public entry fun channel_close_confirm(_channel_id: u32) {
        abort E_INFINITE_GAME
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


    public entry fun recv_packet(
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
        relayer: address,
        relayer_msg: vector<u8>,
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
            let ibc_packet = *vector::borrow(&packets, i);
            let raw_zkgm_packet = packet::data(&ibc_packet);
            let zkgm_packet = decode_packet(*raw_zkgm_packet);
            let acknowledgement =
                execute_internal(
                    ibc_store,
                    relay_store,
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    zkgm_packet.salt,
                    zkgm_packet.path,
                    zkgm_packet.instruction,
                    ctx
                );

            if (vector::length(&acknowledgement) == 0) {
                abort E_ACK_EMPTY
            } else if (acknowledgement == ACK_ERR_ONLYMAKER) {
                abort E_ONLY_MAKER
            } else {
                // TODO: what to do here?
                let return_value =
                    encode_ack(
                        &Acknowledgement { tag: ACK_SUCCESS, inner_ack: acknowledgement }
                    );
                // dispatcher_zkgm::set_return_value<ZKGMProof>(
                //     new_ucs_relay_proof(), return_value
                // );
            }

        };
    }


    fun execute_internal(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ): (vector<u8>)  {
        if (instruction.version != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };

        if (instruction.opcode == SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            execute_fungible_asset_transfer(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                decode_fungible_asset_transfer(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_BATCH) {
            execute_batch(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                path,
                decode_batch_packet(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_FORWARD) {
            execute_forward(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer_msg,
                salt,
                path,
                decode_forward(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_MULTIPLEX) {
            execute_multiplex(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                relayer_msg,
                salt,
                decode_multiplex(instruction.operand),
                ctx
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    // TODO: Voyager need to call this after token deployment
    // with wrapped address & value
    public entry fun update_token_origin(
        relay_store: &mut RelayStore,
        token: address,
        channel_id: u256
    ) {
        add_or_update_table<address, u256>(&mut relay_store.token_origin, token, channel_id);
    }

    fun execute_fungible_asset_transfer(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        transfer_packet: FungibleAssetTransferPacket,
        ctx: &mut TxContext
    ): (vector<u8>) {
        // TODO: this function will be a problem for us
        // investigate it later.
        vector::empty()
    }

    fun execute_batch(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        batch_packet: Batch,
        ctx: &mut TxContext
    ): (vector<u8>) {
        let l = vector::length(&batch_packet.instructions);
        let mut acks = vector::empty();
        let mut i = 0;
        while (i < l) {
            let instruction = *vector::borrow(&batch_packet.instructions, i);
            vector::push_back(
                &mut acks,
                execute_internal(
                    ibc_store,
                    relay_store,
                    ibc_packet,
                    relayer,
                    relayer_msg,
                    salt,
                    path,
                    instruction,
                    ctx
                )
            );
            if (vector::length(vector::borrow(&acks, i)) == 0) {
                abort E_BATCH_MUST_BE_SYNC
            };
        };
        encode_batch_ack(&BatchAcknowledgement { acknowledgements: acks })

    }

    fun execute_forward(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        path: u256,
        forward_packet: ForwardPacket,
        ctx: &mut TxContext
    ): (vector<u8>) {
        let sent_packet =
            ibc::send_packet(
                ibc_store,
                forward_packet.channel_id,
                forward_packet.timeout_height,
                forward_packet.timeout_timestamp,
                encode_packet(
                    &ZkgmPacket {
                        salt: salt,
                        path: update_channel_path(
                            path, packet::destination_channel(&ibc_packet)
                        ),
                        instruction: forward_packet.instruction
                    }
                )
            );
        let packet_hash = commitment::commit_packet(&sent_packet);
        add_or_update_table<vector<u8>, Packet>(&mut relay_store.in_flight_packet, packet_hash, sent_packet);
        ACK_EMPTY
    }

    fun add_or_update_table<T: drop + store + copy, P: drop + store>(table: &mut Table<T, P>, key: T, mut value: P) {
        if (table.contains(key)) {
            let mut val = table.borrow_mut(key);
            *val = value;
        } else {
            table.add(key, value);
        }
    }

    fun execute_multiplex(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        relayer_msg: vector<u8>,
        salt: vector<u8>,
        multiplex_packet: MultiplexPacket,
        ctx: &mut TxContext
    ): (vector<u8>) {
        if (multiplex_packet.eureka) {
            // TODO: discuss this, is it ok to do?
            event::emit(
                OnZkgmCall {
                    sender: multiplex_packet.sender,
                    contract_calldata: multiplex_packet.contract_calldata,
                    contract_address: multiplex_packet.contract_address
                }
            );
            return bcs::to_bytes(&ACK_SUCCESS)
        };
        let multiplex_ibc_packet =
            packet::new(
                packet::source_channel(&ibc_packet),
                packet::destination_channel(&ibc_packet),
                encode_multiplex_sender_and_calldata(
                    multiplex_packet.sender, multiplex_packet.contract_calldata
                ),
                packet::timeout_height(&ibc_packet),
                packet::timeout_timestamp(&ibc_packet)
            );

        // TODO: How do we return something from this? investigate
        event::emit(
            OnIIBCModuleOnRecvPacketCall {
                packet: multiplex_ibc_packet,
                relayer: relayer,
                relayer_msg: relayer_msg,
                contract_address: multiplex_packet.contract_address
            }
        );
        vector::empty() // TODO: investigate
        // Here is the original implementation in aptos
        // let param =
        //     copyable_any::pack<IIBCModuleOnRecvPacketParams>(
        //         IIBCModuleOnRecvPacketParams {
        //             packet: multiplex_ibc_packet,
        //             relayer: relayer,
        //             relayer_msg: relayer_msg
        //         }
        //     );

        // engine_zkgm::dispatch(param, contract_address);

        // let acknowledgement = dispatcher_zkgm::get_return_value(contract_address);

        // if (vector::length(&acknowledgement) == 0) {
        //     abort E_UNIMPLEMENTED
        // };
        // acknowledgement
    }

    public entry fun send(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        channel_id: u32,
        timeout_height: u64,
        timeout_timestamp: u64,
        salt: vector<u8>,
        version: u8,
        opcode: u8,
        operand: vector<u8>,
        ctx: &mut TxContext
    ) {
        let instruction = Instruction { version: version, opcode: opcode, operand: operand };
        let sender = tx_context::sender(ctx);
        verify_internal(ibc_store, relay_store, sender, channel_id, 0, instruction, ctx);
        ibc::send_packet(
            ibc_store,
            channel_id,
            timeout_height,
            timeout_timestamp,
            encode_packet(&ZkgmPacket { salt: salt, path: 0, instruction }),
        );
    }
    fun verify_internal(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        instruction: Instruction,
        ctx: &mut TxContext
    ){
        if (instruction.version != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (instruction.opcode == SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            verify_fungible_asset_transfer(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                decode_fungible_asset_transfer(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_BATCH) {
            verify_batch(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                decode_batch_packet(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_FORWARD) {
            verify_forward(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                decode_forward(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_MULTIPLEX) {
            verify_multiplex(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                decode_multiplex(instruction.operand),
                ctx
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        };
    }

    fun verify_fungible_asset_transfer(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        transfer_packet: FungibleAssetTransferPacket,
        ctx: &mut TxContext
    ){
        let sent_token = transfer_packet.sent_token;
        let treasury_cap = relay_store.address_to_treasurycap.borrow_mut(sent_token);

        // TODO: implement this further, can't take coin as argument because of that
        // copy issue

    }

    fun verify_batch(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        batch_packet: Batch,
        ctx: &mut TxContext
    ){
        let l = vector::length(&batch_packet.instructions);

        let mut i = 0;
        while (i < l) {
            verify_internal(
                ibc_store,
                relay_store,
                sender,
                channel_id,
                path,
                *vector::borrow(&batch_packet.instructions, i),
                ctx
            );
            i = i + 1;
        }

    }

    fun verify_forward(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        forward_packet: ForwardPacket,
        ctx: &mut TxContext
    ){
        verify_internal(
            ibc_store,
            relay_store,
            sender,
            channel_id,
            update_channel_path(path, forward_packet.channel_id),
            forward_packet.instruction,
            ctx
        );
    }

    fun verify_multiplex(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        sender: address,
        channel_id: u32,
        path: u256,
        multiplex_packet: MultiplexPacket,
        ctx: &mut TxContext
    ){

    }

    public entry fun acknowledge_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet_source_channels: vector<u32>,
        packet_destination_channels: vector<u32>,
        packet_datas: vector<vector<u8>>,
        packet_timeout_heights: vector<u64>,
        packet_timeout_timestamps: vector<u64>,
        acknowledgements: vector<vector<u8>>,
        relayer: address,
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
        ibc::acknowledge_packet(
            ibc_store,
            packets,
            acknowledgements,
            proof,
            proof_height
        );

        let mut i = 0;
        while (i < vector::length(&packet_source_channels)) {
            let ibc_packet = *vector::borrow(&packets, i);
            let acknowledgement = *vector::borrow(&acknowledgements, i);
            let packet_hash = commitment::commit_packet(&ibc_packet);
            let parent = relay_store.in_flight_packet.borrow(packet_hash);

            if (packet::timeout_timestamp(parent) != 0 ||
                packet::timeout_height(parent) != 0) {
                    ibc::write_acknowledgement(
                        ibc_store,
                        *parent,
                        acknowledgement
                    );
                    add_or_update_table<vector<u8>, Packet>(
                        &mut relay_store.in_flight_packet,
                        packet_hash,
                        packet::default()
                    );
            } else {
                let zkgm_packet = decode_packet(*packet::data(&ibc_packet));
                let zkgm_ack = decode_ack(acknowledgement);
                acknowledge_internal(
                    ibc_store,
                    relay_store,
                    ibc_packet,
                    relayer,
                    zkgm_packet.salt,
                    zkgm_packet.instruction,
                    zkgm_ack.tag == ACK_SUCCESS,
                    zkgm_ack.inner_ack,
                    ctx
                )
            };
            i = i + 1;
        };
    }

    fun acknowledge_internal(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        instruction: Instruction,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        if (instruction.version != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (instruction.opcode == SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            acknowledge_fungible_asset_transfer(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                decode_fungible_asset_transfer(instruction.operand),
                success,
                ack,
                ctx
            )
        } else if (instruction.opcode == SYSCALL_BATCH) {
            acknowledge_batch(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                decode_batch_packet(instruction.operand),
                success,
                ack,
                ctx
            )
        } else if (instruction.opcode == SYSCALL_FORWARD) {
            acknowledge_forward(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                decode_forward(instruction.operand),
                success,
                ack,
                ctx
            )
        } else if (instruction.opcode == SYSCALL_MULTIPLEX) {
            acknowledge_multiplex(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                decode_multiplex(instruction.operand),
                success,
                ack,
                ctx
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun acknowledge_fungible_asset_transfer(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        transfer_packet: FungibleAssetTransferPacket,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        // TODO: fill it later
    }

    fun acknowledge_batch(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        batch_packet: Batch,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        let l = vector::length(&batch_packet.instructions);
        let batch_ack = decode_batch_ack(ack);
        let mut i = 0;
        while (i < l) {
            let mut syscall_ack = ack;
            if (success) {
                syscall_ack = *vector::borrow(&batch_ack.acknowledgements, i);
            };
            acknowledge_internal(
                ibc_store,
                relay_store,
                ibc_packet,
                relayer,
                salt,
                *vector::borrow(&batch_packet.instructions, i),
                success,
                syscall_ack,
                ctx
            );
            i = i + 1;
        }
    }

    fun acknowledge_forward(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        forward_packet: ForwardPacket,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {

    }

    fun acknowledge_multiplex(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        salt: vector<u8>,
        multiplex_packet: MultiplexPacket,
        success: bool,
        ack: vector<u8>,
        ctx: &mut TxContext
    ) {
        if (success && !multiplex_packet.eureka) {
            let multiplex_ibc_packet =
                packet::new(
                    packet::source_channel(&ibc_packet),
                    packet::destination_channel(&ibc_packet),
                    encode_multiplex_sender_and_calldata(
                        multiplex_packet.contract_address,
                        multiplex_packet.contract_calldata
                    ),
                    packet::timeout_height(&ibc_packet),
                    packet::timeout_timestamp(&ibc_packet)
                );
            // TODO: verify this
            event::emit(
                OnIIBCModuleOnAcknowledgementPacketCall {
                    packet: multiplex_ibc_packet,
                    acknowledgement: ack,
                    relayer: relayer,
                    contract_address: multiplex_packet.sender
                }
            )
            // let param =
            //     copyable_any::pack<IIBCModuleOnAcknowledgementPacketParams>(
            //         IIBCModuleOnAcknowledgementPacketParams {
            //             packet: multiplex_ibc_packet,
            //             acknowledgement: ack,
            //             relayer: relayer
            //         }
            //     );
            // let contract_address = from_bcs::to_address(multiplex_packet.sender);

            // engine_zkgm::dispatch(param, contract_address);
        }
    }
    public entry fun timeout_packet(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet_source_channel: u32,
        packet_destination_channel: u32,
        packet_data: vector<u8>,
        packet_timeout_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height: u64,
        next_sequence_receive: u64,
        relayer: address,
        ctx: &mut TxContext
    ) {
        let packet =
            packet::new(
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

        let packet_hash = commitment::commit_packet(&packet);

        let parent = relay_store.in_flight_packet.borrow(packet_hash);
        let parent = relay_store.in_flight_packet.borrow(packet_hash);
        if (packet::timeout_timestamp(parent) != 0 ||
            packet::timeout_height(parent) != 0) {
                let ack = Acknowledgement { tag: ACK_FAILURE, inner_ack: ACK_EMPTY };
                ibc::write_acknowledgement(
                    ibc_store,
                    *parent,
                    encode_ack(&ack)
                );
                add_or_update_table<vector<u8>, Packet>(
                    &mut relay_store.in_flight_packet,
                    packet_hash,
                    packet::default()
                );
        } else {

            let zkgm_packet = decode_packet(packet_data);

            timeout_internal(
                ibc_store,
                relay_store,
                packet,
                relayer,
                zkgm_packet.instruction,
                ctx
            )
        }
    }

    fun timeout_internal(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet: Packet,
        relayer: address,
        instruction: Instruction,
        ctx: &mut TxContext
    ) {
        if (instruction.version != ZKGM_VERSION_0) {
            abort E_UNSUPPORTED_VERSION
        };
        if (instruction.opcode == SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            timeout_fungible_asset_transfer(
                ibc_store,
                relay_store,
                packet,
                relayer,
                decode_fungible_asset_transfer(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_BATCH) {
            timeout_batch(
                ibc_store,
                relay_store,
                packet,
                relayer,
                decode_batch_packet(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_FORWARD) {
            timeout_forward(
                ibc_store,
                relay_store,
                packet,
                relayer,
                decode_forward(instruction.operand),
                ctx
            )
        } else if (instruction.opcode == SYSCALL_MULTIPLEX) {
            timeout_multiplex(
                ibc_store,
                relay_store,
                packet,
                relayer,
                decode_multiplex(instruction.operand),
                ctx
            )
        } else {
            abort E_UNKNOWN_SYSCALL
        }
    }

    fun timeout_fungible_asset_transfer(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet: Packet,
        relayer: address,
        transfer_packet: FungibleAssetTransferPacket,
        ctx: &mut TxContext
    ) {
        refund(packet::source_channel(&packet), transfer_packet, ctx);
    }

    fun refund(
        channel_id: u32,
        transfer_packet: FungibleAssetTransferPacket,
        ctx: &mut TxContext
    ) {
        // TOOD: Fill it later
    }

    fun timeout_batch(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet: Packet,
        relayer: address,
        batch_packet: Batch,
        ctx: &mut TxContext
    ) {
        let l = vector::length(&batch_packet.instructions);
        let mut i = 0;
        while (i < l) {
            timeout_internal(
                ibc_store,
                relay_store,
                packet,
                relayer,
                 *vector::borrow(&batch_packet.instructions, i),
                ctx
            );
            i = i + 1;
        }
    }

    fun timeout_forward(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        packet: Packet,
        relayer: address,
        forward_packet: ForwardPacket,
        ctx: &mut TxContext
    ) {

    }

    fun timeout_multiplex(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        ibc_packet: Packet,
        relayer: address,
        multiplex_packet: MultiplexPacket,
        ctx: &mut TxContext
    ) {
        if (!multiplex_packet.eureka) {
            let multiplex_ibc_packet =
                packet::new(
                    packet::source_channel(&ibc_packet),
                    packet::destination_channel(&ibc_packet),
                    encode_multiplex_sender_and_calldata(
                        multiplex_packet.contract_address,
                        multiplex_packet.contract_calldata
                    ),
                    packet::timeout_height(&ibc_packet),
                    packet::timeout_timestamp(&ibc_packet)
                );
            // TODO: verify this

            event::emit(
                OnIIBCModuleOnTimeoutPacketCall{
                    packet: multiplex_ibc_packet,
                    relayer: relayer,
                    contract_address: multiplex_packet.sender
                }
            );
            // let param =
            //     copyable_any::pack<IIBCModuleOnTimeoutPacketParams>(
            //         IIBCModuleOnTimeoutPacketParams {
            //             packet: multiplex_ibc_packet,
            //             relayer: relayer
            //         }
            //     );
            // let contract_address = from_bcs::to_address(multiplex_packet.sender);

            // engine_zkgm::dispatch(param, contract_address);

        }
    }

    public entry fun execute(
        ibc_store: &mut ibc::IBCStore,
        relay_store: &mut RelayStore,
        source_channel: u32,
        destination_channel: u32,
        data: vector<u8>,
        timeout_height: u64,
        timeout_timestamp: u64,
        relayer: address,
        relayer_msg: vector<u8>,
        raw_zkgm_packet: vector<u8>,
        ctx: &mut TxContext
    ) {
        let ibc_packet =
            packet::new(
                source_channel,
                destination_channel,
                data,
                timeout_height,
                timeout_timestamp
            );

        let zkgm_packet = decode_packet(raw_zkgm_packet);
        execute_internal(
            ibc_store,
            relay_store,
            ibc_packet,
            relayer,
            relayer_msg,
            zkgm_packet.salt,
            zkgm_packet.path,
            zkgm_packet.instruction,
            ctx
        );
    }


    #[test]
    fun test_zkgm_encode_decode() {
        let output =
            x"000000000000000000000000000000000000000000000000000000000000002068656c6c6f6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000032dcd60000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000de0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000";

        let instruction1 = Instruction {
            version: 111,
            opcode: 222,
            operand: b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world"
        };

        let zkgm_data = ZkgmPacket {
            salt: x"68656c6c6f6f0000000000000000000000000000000000000000000000000000",
            path: 3333334,
            instruction: instruction1
        };

        let zkgm_bytes = encode_packet(&zkgm_data);
        assert!(zkgm_bytes == output, 0);

        let zkgm_data_decoded = decode_packet(zkgm_bytes);

        assert!(
            zkgm_data_decoded.salt
                == x"68656c6c6f6f0000000000000000000000000000000000000000000000000000",
            1
        );
        assert!(zkgm_data_decoded.path == 3333334, 2);
        assert!(zkgm_data_decoded.instruction == instruction1, 3);
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
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000de0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002668686820776f726c6468656c6c6f20777777776c6f20776f726c6468656c6c6f20776f726c64000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000370000000000000000000000000000000000000000000000000000000000000042000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000086272726168686868000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();

        let instruction1 = Instruction {
            version: 111,
            opcode: 222,
            operand: b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world"
        };

        let instruction2 = Instruction {
            version: 1,
            opcode: 2,
            operand: b"hhh worldhello wwwwlo worldhello world"
        };

        let instruction3 = Instruction { version: 55, opcode: 66, operand: b"brrahhhh" };
        vector::push_back(&mut outer_arr, instruction1);
        vector::push_back(&mut outer_arr, instruction2);
        vector::push_back(&mut outer_arr, instruction3);
        let ack_data = Batch { instructions: outer_arr };
        let ack_bytes = encode_batch_packet(&ack_data);
        assert!(ack_bytes == output, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes);
        assert!(vector::length(&ack_data_decoded.instructions) == 3, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 1) == instruction2, 3);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 2) == instruction3, 4);

        // ---------------- TEST 2 ----------------
        let output2 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000162000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();

        let instruction1 = Instruction { version: 3, opcode: 5, operand: b"b" };

        let instruction2 = Instruction { version: 2, opcode: 4, operand: b"" };
        vector::push_back(&mut outer_arr, instruction1);
        vector::push_back(&mut outer_arr, instruction2);
        let ack_data2 = Batch { instructions: outer_arr };
        let ack_bytes2 = encode_batch_packet(&ack_data2);
        assert!(ack_bytes2 == output2, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes2);
        assert!(vector::length(&ack_data_decoded.instructions) == 2, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 1) == instruction2, 3);

        // ---------------- TEST 3 ----------------
        let output3 =
            x"000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000007b00000000000000000000000000000000000000000000000000000000000000df000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000bd617764617764617764617764776164616161616161612061616161616161616161616161616161616161616120626262622064616477647720772077777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777000000";
        let mut outer_arr = vector::empty();

        let instruction1 = Instruction {
            version: 123,
            opcode: 223,
            operand: b"awdawdawdawdwadaaaaaaa aaaaaaaaaaaaaaaaaaaaa bbbb dadwdw w wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww"
        };

        vector::push_back(&mut outer_arr, instruction1);

        let ack_data3 = Batch { instructions: outer_arr };
        let ack_bytes3 = encode_batch_packet(&ack_data3);
        assert!(ack_bytes3 == output3, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes3);
        assert!(vector::length(&ack_data_decoded.instructions) == 1, 1);
        assert!(*vector::borrow(&ack_data_decoded.instructions, 0) == instruction1, 2);

        // ---------------- TEST 4 ----------------
        let output4 =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000";
        let mut outer_arr = vector::empty();

        let ack_data4 = Batch { instructions: outer_arr };
        let ack_bytes4 = encode_batch_packet(&ack_data4);
        assert!(ack_bytes4 == output4, 0);
        let ack_data_decoded = decode_batch_packet(ack_bytes4);
        assert!(vector::length(&ack_data_decoded.instructions) == 0, 1);

    }

    #[test]
    fun test_encode_decode_forward_packet() {
        let output =
            x"0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000003700000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000006f00000000000000000000000000000000000000000000000000000000000000de0000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000007968656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6468656c6c6f20776f726c6400000000000000";

        let instruction = Instruction {
            version: 111,
            opcode: 222,
            operand: b"hello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello worldhello world"
        };

        let forward_data = ForwardPacket {
            channel_id: 44,
            timeout_height: 55,
            timeout_timestamp: 66,
            instruction: instruction
        };

        let ack_bytes = encode_forward(&forward_data);
        std::debug::print(&string::utf8(b"ack bytes: "));
        std::debug::print(&ack_bytes);
        assert!(ack_bytes == output, 0);

        let forward_data_decoded = decode_forward(ack_bytes);
        assert!(forward_data_decoded.channel_id == forward_data.channel_id, 0);
        assert!(forward_data_decoded.timeout_height == forward_data.timeout_height, 1);
        assert!(
            forward_data_decoded.timeout_timestamp == forward_data.timeout_timestamp, 2
        );
        assert!(forward_data_decoded.instruction == forward_data.instruction, 3);
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

    // #[test]
    // fun test_decode_fungible_asset_transfer_pack() {

    //     let output =
    //         x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000007c37bdc730000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000024000000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000007a1200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f307853656e646572416464726573730000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001130785265636569766572416464726573730000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000012307853656e74546f6b656e4164647265737300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014535353535353535353535353535353594d424f4c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000155454545454545454545454546f6b656e204e616d6500000000000000000000000000000000000000000000000000000000000000000000000000000000000011307841736b546f6b656e41646472657373000000000000000000000000000000";
    //     let fatp = FungibleAssetTransferPacket {
    //         sender: b"0xSenderAddress",
    //         receiver: b"0xReceiverAddress",
    //         sent_token: b"0xSentTokenAddress",
    //         sent_token_prefix: 33344445555,
    //         sent_symbol: string::utf8(b"SSSSSSSSSSSSSSSYMBOL"),
    //         sent_name: string::utf8(b"TTTTTTTTTTTToken Name"),
    //         sent_amount: 1000000, // Example token amount
    //         ask_token: b"0xAskTokenAddress",
    //         ask_amount: 500000, // Example ask amount
    //         only_maker: false
    //     };

    //     let fatp_decoded = decode_fungible_asset_transfer(output);
    //     assert!(fatp.sender == fatp_decoded.sender, 0);
    //     assert!(fatp.receiver == fatp_decoded.receiver, 0);
    //     assert!(fatp.sent_token == fatp_decoded.sent_token, 0);
    //     assert!(fatp.sent_token_prefix == fatp_decoded.sent_token_prefix, 0);
    //     assert!(fatp.sent_symbol == fatp_decoded.sent_symbol, 0);
    //     assert!(fatp.sent_name == fatp_decoded.sent_name, 0);
    //     assert!(fatp.sent_amount == fatp_decoded.sent_amount, 0);
    //     assert!(fatp.ask_token == fatp_decoded.ask_token, 0);
    //     assert!(fatp.ask_amount == fatp_decoded.ask_amount, 0);
    //     assert!(fatp.only_maker == fatp_decoded.only_maker, 0);
    // }

    fun test_encode_multiplex_sender_and_calldata() {
        let output =
            x"00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000000d23078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443343078354233384461366137303163353638353435644366634230334663423837356635366265646443340000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010e4578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c446174614578616d706c6553797363616c6c44617461000000000000000000000000000000000000";

        let ack_bytes =
            encode_multiplex_sender_and_calldata(
                b"0x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC40x5B38Da6a701c568545dCfcB03FcB875f56beddC4",
                b"ExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallDataExampleSyscallData"
            );
        assert!(ack_bytes == output, 0);
    }
}