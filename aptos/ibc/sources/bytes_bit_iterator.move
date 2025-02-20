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

module ibc::bytes_bit_iterator {
    use std::vector;

    struct BytesBitIterator has drop {
        bz: vector<u8>,
        pos: u32,
        little_endian: bool,
        reverse: bool
    }

    public fun new(bytes: vector<u8>, skip: u32, little_endian: bool): BytesBitIterator {
        // TODO(aeryz): assert skip
        BytesBitIterator { bz: bytes, pos: skip, little_endian, reverse: false }
    }

    public fun new_rev(bytes: vector<u8>, skip: u32, little_endian: bool): BytesBitIterator {
        // TODO(aeryz): assert skip
        BytesBitIterator {
            bz: bytes,
            pos: ((vector::length(&bytes) * 8) as u32) - skip,
            little_endian,
            reverse: true
        }
    }

    public fun get_bit(self: &BytesBitIterator, index: u64): bool {
        let pos = index / 8;
        let bit = ((index % 8) as u8);
        if (self.little_endian) {
            bit = 7 - bit;
        };
        (*vector::borrow(&self.bz, pos) >> bit)
        & 1 != 0
    }

    public fun next(self: &mut BytesBitIterator): bool {
        self.pos = if (self.reverse) {
            self.pos - 1
        } else {
            self.pos + 1
        };
        let pos = self.pos;
        get_bit(self, (pos as u64))
    }
}
