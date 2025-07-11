// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's sui subdirectory                      
//                       The Licensed Work is (c) 2024 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
// 

// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.

// Notice

// Business Source License 1.1

// Terms

// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.

// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.

// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.

// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.

// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.

// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.

// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).

// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

#[allow(unused_const)]
module zkgm::helper {
    use sui::bcs;
    use sui::hash;

    const E_INVALID_HOPS: u64 = 2;

    const OP_FORWARD: u8 = 0x00;
    const OP_MULTIPLEX: u8 = 0x01;
    const OP_BATCH: u8 = 0x02;
    const OP_FUNGIBLE_ASSET_ORDER: u8 = 0x03;
    const OP_STAKE: u8 = 0x04;
    const OP_UNSTAKE: u8 = 0x05;
    const OP_WITHDRAW_STAKE: u8 = 0x06;
    const OP_WITHDRAW_REWARDS: u8 = 0x07;

    const FORWARD_SALT_MAGIC: u256 = 0xC0DE00000000000000000000000000000000000000000000000000000000BABE;
    
    public(package) fun is_allowed_batch_instruction(
        opcode: u8
    ): bool {
        opcode == OP_MULTIPLEX || opcode == OP_FUNGIBLE_ASSET_ORDER
            || opcode == OP_STAKE || opcode == OP_UNSTAKE
            || opcode == OP_WITHDRAW_STAKE
    }

    public(package) fun is_allowed_forward(
        opcode: u8
    ): bool {        
        opcode == OP_MULTIPLEX || 
            opcode == OP_BATCH || 
            opcode == OP_FUNGIBLE_ASSET_ORDER
    }

    // TODO(aeryz): test this
    public(package) fun is_forwarded_packet(mut salt: vector<u8>): bool {
        salt.reverse();
        let salt_u256 = bcs::new(salt).peel_u256();

        (salt_u256 & FORWARD_SALT_MAGIC) == FORWARD_SALT_MAGIC
    }

    public(package) fun derive_batch_salt(
        index: u64,
        salt: vector<u8>
    ): vector<u8> {
        let mut data: vector<u8> = bcs::to_bytes(&(index as u256));
        data.append(salt);
        hash::keccak256(&data)
    }

    public(package) fun derive_forward_salt(
        salt: vector<u8>
    ): vector<u8> {
        tint_forward_salt(hash::keccak256(&salt))
    }

    // TODO(aeryz): test this
    public(package) fun tint_forward_salt(
        mut salt: vector<u8>
    ): vector<u8> {
        salt.reverse();
        let salt_u256 = bcs::new(salt).peel_u256();

        let salt = FORWARD_SALT_MAGIC | (salt_u256 & (FORWARD_SALT_MAGIC ^ 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
        bcs::to_bytes(&salt)
    }

    public(package) fun dequeue_channel_from_path(
        path: u256,
    ): (u256, u32) {
        (path >> 32, path as u32)
    }



    public(package) fun update_channel_path(path: u256, next_channel_id: u32): u256 {
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

    public(package) fun reverse_channel_path(mut path: u256): u256 {
        let mut reversed_path = 0;
        loop {
            let (tail, head) = pop_channel_from_path(path);
            reversed_path = update_channel_path(reversed_path, head);
            path = tail;
            if (path == 0) {
                break
            }
        };
        reversed_path
    }


    public(package) fun pop_channel_from_path(path: u256) : (u256, u32){
        if (path == 0) {
            return (0, 0)
        };
        let current_hop_index = ((fls(path) / 32)) as u8;
        let clear_shift = ((8-current_hop_index) * 32) as u8;
        return (
            (path << clear_shift) >> clear_shift,
            (path >> (current_hop_index * 32)) as u32
        )
    }

    /// Find last set (most significant bit).
    /// Returns the index of the most significant bit of `x`.
    /// If `x` is zero, returns 256.
    public(package) fun fls(mut x: u256): u256 {
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
}
