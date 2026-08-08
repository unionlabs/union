module zkgm::lib {
    use std::aptos_hash;
    use std::bcs;
    use std::from_bcs;

    friend zkgm::ibc_app;

    const E_INVALID_HOPS: u64 = 2;

    const OP_FORWARD: u8 = 0x00;
    const OP_MULTIPLEX: u8 = 0x01;
    const OP_BATCH: u8 = 0x02;
    const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;

    const FORWARD_SALT_MAGIC: u256 =
        0xC0DE00000000000000000000000000000000000000000000000000000000BABE;

    // TODO(aeryz): test this well
    public(friend) fun is_forwarded_packet(salt: vector<u8>): bool {
        let salt = from_bcs::to_u256(salt);

        (salt & FORWARD_SALT_MAGIC) == FORWARD_SALT_MAGIC
    }

    /// Find last set (most significant bit).
    /// Returns the index of the most significant bit of `x`.
    /// If `x` is zero, returns 256.
    public(friend) fun fls(x: u256): u256 {
        if (x == 0) {
            return 256
        };

        let r: u256 = 0;

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

    public(friend) fun dequeue_channel_from_path(path: u256): (u256, u32) {
        (path >> 32, (path as u32))
    }

    public(friend) fun reverse_channel_path(path: u256): u256 {
        (((path >> 0) as u32) as u256) << 224 | (((path >> 32) as u32) as u256) << 192
            | (((path >> 64) as u32) as u256) << 160
            | (((path >> 96) as u32) as u256) << 128
            | (((path >> 128) as u32) as u256) << 96
            | (((path >> 160) as u32) as u256) << 64
            | (((path >> 192) as u32) as u256) << 32
            | (((path >> 224) as u32) as u256) << 0
    }

    public(friend) fun pop_channel_from_path(path: u256): (u256, u32) {
        if (path == 0) {
            return (0, 0)
        };
        let current_hop_index = fls(path) / 32;
        let clear_shift = (((8 - current_hop_index) * 32) as u8);
        ((path << clear_shift) >> clear_shift,
        ((path >> ((current_hop_index * 32) as u8)) as u32))
    }

    public(friend) fun update_channel_path(
        path: u256, next_channel_id: u32
    ): u256 {
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

    public(friend) fun is_allowed_batch_instruction(opcode: u8): bool {
        opcode == OP_MULTIPLEX || opcode == OP_FUNGIBLE_ASSET_ORDER
    }

    public(friend) fun is_allowed_forward_instruction(opcode: u8): bool {
        opcode == OP_MULTIPLEX
            || opcode == OP_FUNGIBLE_ASSET_ORDER
            || opcode == OP_BATCH
    }

    public(friend) fun tint_forward_salt(salt: vector<u8>): vector<u8> {
        let salt = from_bcs::to_u256(salt);
        let salt =
            FORWARD_SALT_MAGIC
                | (
                    salt
                        & (
                            FORWARD_SALT_MAGIC
                                ^
                                0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
                        )
                );
        bcs::to_bytes(&salt)
    }

    public(friend) fun derive_forward_salt(salt: vector<u8>): vector<u8> {
        tint_forward_salt(aptos_hash::keccak256(salt))
    }

    #[test]
    public fun test_update_channel_path() {
        assert!(update_channel_path(0, 0) == 0, 1);
        assert!(update_channel_path(0, 34) == 34, 1);
        assert!(update_channel_path(12414123, 111) == 476753783979, 1);
        assert!(update_channel_path(44, 22) == 94489280556, 1);
    }

    #[test]
    fun test_fls() {
        assert!(fls(0) == 256, 1);
        assert!(fls(22) == 4, 23);
        assert!(fls(32) == 5, 33);
        assert!(fls(444) == 8, 33);
        assert!(fls(6671) == 12, 33);
        assert!(fls(33334411) == 24, 33);
    }
}
