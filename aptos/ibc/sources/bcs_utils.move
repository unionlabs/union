// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.

// Parameters

// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's aptos subdirectory
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

module ibc::bcs_utils {
    use std::string::String;
    use std::from_bcs;
    use std::vector;
    use std::option::{Self, Option};

    const E_INVALID_PREFIX: u64 = 9000;

    struct BcsBuf has drop {
        inner: vector<u8>,
        cursor: u64
    }

    public fun new(buf: vector<u8>): BcsBuf {
        BcsBuf { inner: buf, cursor: 0 }
    }

    public fun remaining(buf: &BcsBuf): vector<u8> {
        vector::slice(&buf.inner, buf.cursor, vector::length(&buf.inner) - 1)
    }

    /// Parses the length prefix of a variable length type such as vector and string
    ///
    /// Returns the parsed length and the number of bytes read
    fun parse_length_prefix(buf: &BcsBuf): (u32, u64) {
        let value: u64 = 0;
        let shift = 0;
        let i = buf.cursor;
        let first_pos = i;
        while (shift < 32) {
            let byte = vector::borrow(&buf.inner, i);
            let digit = *byte & 0x7f;
            value = value | ((digit as u64) << shift);
            if (digit == *byte) {
                if (shift > 0 && digit == 0) {
                    abort E_INVALID_PREFIX
                };

                return ((value as u32), i + 1 - first_pos)
            };

            i = i + 1;
            shift = shift + 7;
        };

        abort E_INVALID_PREFIX
    }

    /// Peel a variable-length length prefix of variable-sized types
    public fun peel_length_prefix(buf: &mut BcsBuf): u32 {
        let (length, n_read) = parse_length_prefix(buf);
        buf.cursor = buf.cursor + n_read;
        length
    }

    /// Peel a u8
    public fun peel_u8(buf: &mut BcsBuf): u8 {
        buf.cursor = buf.cursor;
        from_bcs::to_u8(vector::slice(&buf.inner, buf.cursor - 1, buf.cursor))
    }

    /// Peel a u16
    public fun peel_u16(buf: &mut BcsBuf): u16 {
        buf.cursor = buf.cursor + 2;
        from_bcs::to_u16(vector::slice(&buf.inner, buf.cursor - 2, buf.cursor))
    }

    /// Peel a u32
    public fun peel_u32(buf: &mut BcsBuf): u32 {
        buf.cursor = buf.cursor + 4;
        from_bcs::to_u32(vector::slice(&buf.inner, buf.cursor - 4, buf.cursor))
    }

    /// Peel a u64
    public fun peel_u64(buf: &mut BcsBuf): u64 {
        buf.cursor = buf.cursor + 8;
        from_bcs::to_u64(vector::slice(&buf.inner, buf.cursor - 8, buf.cursor))
    }

    /// Peel a string
    public fun peel_string(buf: &mut BcsBuf): String {
        let (length, n_read) = parse_length_prefix(buf);
        buf.cursor = buf.cursor + n_read + (length as u64);
        from_bcs::to_string(
            vector::slice(
                &buf.inner,
                buf.cursor - (length as u64) - n_read,
                buf.cursor
            )
        )
    }

    /// Peel a vector<u8>
    public fun peel_bytes(buf: &mut BcsBuf): vector<u8> {
        let (length, n_read) = parse_length_prefix(buf);
        buf.cursor = buf.cursor + n_read + (length as u64);
        from_bcs::to_bytes(
            vector::slice(
                &buf.inner,
                buf.cursor - (length as u64) - n_read,
                buf.cursor
            )
        )
    }

    /// Peel an array of `length` bytes
    public fun peel_fixed_bytes(buf: &mut BcsBuf, length: u32): vector<u8> {
        buf.cursor = buf.cursor + (length as u64);
        vector::slice(&buf.inner, buf.cursor - (length as u64), buf.cursor)
    }

    /// Peel a vector of T
    public inline fun peel_vector<T>(
        buf: &mut BcsBuf, parse_fn: |&mut BcsBuf| T
    ): vector<T> {
        let length = peel_length_prefix(buf);
        let i = 0;
        let vec: vector<T> = vector::empty();
        while (i < length) {
            vector::push_back(&mut vec, parse_fn(buf));
            i = i + 1;
        };
        vec
    }

    /// Peel an option of T
    public inline fun peel_option<T>(
        buf: &mut BcsBuf, parse_fn: |&mut BcsBuf| T
    ): Option<T> {
        let is_some = peel_u8(buf);
        // NOTE(aeryz): you can't throw a constant that is defined in this module
        // since this is an inline function
        assert!(is_some == 1 || is_some == 0, 9001);
        if (is_some == 1) {
            option::some(parse_fn(buf))
        } else {
            option::none()
        }
    }
}
