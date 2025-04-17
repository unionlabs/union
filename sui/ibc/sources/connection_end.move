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

module ibc::connection_end {
    use std::string::{String, utf8};
    use std::vector;
    use ibc::ethabi;

    public struct ConnectionEnd has copy, store, drop {
        state: u64,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    }

    // Getters
    public fun state(connection: &ConnectionEnd): u64 {
        connection.state
    }

    public fun client_id(connection: &ConnectionEnd): u32 {
        connection.client_id
    }

    public fun counterparty_client_id(connection: &ConnectionEnd): u32 {
        connection.counterparty_client_id
    }

    public fun counterparty_connection_id(connection: &ConnectionEnd): u32 {
        connection.counterparty_connection_id
    }

    // Setters
    public fun set_state(connection: &mut ConnectionEnd, new_state: u64) {
        connection.state = new_state;
    }

    public fun set_client_id(
        connection: &mut ConnectionEnd, new_client_id: u32
    ) {
        connection.client_id = new_client_id;
    }

    public fun set_counterparty_client_id(
        connection: &mut ConnectionEnd, new_id: u32
    ) {
        connection.counterparty_client_id = new_id;
    }

    public fun set_counterparty_connection_id(
        connection: &mut ConnectionEnd, new_id: u32
    ) {
        connection.counterparty_connection_id = new_id;
    }


    // Encode and decode functions
    public fun encode(connection: &ConnectionEnd): vector<u8> {
        // TODO: test this later
        let mut buf = vector::empty();

        ethabi::encode_uint(&mut buf, connection.state);
        ethabi::encode_uint(&mut buf, connection.client_id);
        ethabi::encode_uint(&mut buf, connection.counterparty_client_id);
        ethabi::encode_uint(&mut buf, connection.counterparty_connection_id);

        buf
    }

    // Constructor
    public fun new(
        state: u64,
        client_id: u32,
        counterparty_client_id: u32,
        counterparty_connection_id: u32
    ): ConnectionEnd {
        ConnectionEnd {
            state,
            client_id,
            counterparty_client_id,
            counterparty_connection_id
        }
    }

    // Default function
    public fun default(): ConnectionEnd {
        new(0, 0, 0, 0)
    }

    #[test]
    fun test_encode_decode_connection() {
        let buf =
            x"0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000006400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014";
        let connection = ConnectionEnd {
            state: 2,
            client_id: 100,
            counterparty_client_id: 0,
            counterparty_connection_id: 20
        };

        let encoded = encode(&connection);

        std::debug::print(&encoded);

        assert!(encoded == buf, 1);
    }
}