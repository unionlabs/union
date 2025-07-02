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

module ibc::light_client {
    use std::option::Option;
    use std::string::{Self, String};

    use sui::table::{Self, Table};
    use sui::clock::Clock;
    use sui::object_bag::{Self, ObjectBag};

    use ibc::height::Height;
    use ibc::create_lens_client_event::CreateLensClientEvent;
    use ibc::cometbls_light_client;

    const E_CLIENT_TYPE_NOT_SUPPORTED: u64 = 1;

    public struct LightClientManager has store {
        clients: ObjectBag,
        client_id_to_type: Table<u32, String>,
    }

    public(package) fun new(ctx: &mut TxContext): LightClientManager {
        LightClientManager {
            clients: object_bag::new(ctx),
            client_id_to_type: table::new(ctx)
        }
    }

    public(package) fun exists(
        store: &LightClientManager,
        client_id: u32
    ): bool {
        store.client_id_to_type.contains(client_id)
    }

    public(package) fun create_client(
        store: &mut LightClientManager,
        client_type: String,
        client_id: u32,
        client_state_bytes: vector<u8>,
        consensus_state_bytes: vector<u8>,
        ctx: &mut TxContext,
    ): (vector<u8>, vector<u8>, String, Option<CreateLensClientEvent>) {
        let (csb, consb, c_cid, l_event) =  if (client_type.bytes() == b"cometbls") {
            let (client, csb, consb, c_cid, l_event) = cometbls_light_client::create_client(client_id, client_state_bytes, consensus_state_bytes, ctx);
            store.clients.add(client_id, client);
            (csb, consb, c_cid, l_event)
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        };

        store.client_id_to_type.add(client_id, client_type);

        (csb, consb, c_cid, l_event)
    }

    public(package) fun status(
        store: &LightClientManager,
        client_id: u32
    ): u64 {
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow<u32, cometbls_light_client::Client>(client_id).status()
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }

    public(package) fun misbehaviour(
        store: &mut LightClientManager,
        client_id: u32,
        misbehaviour: vector<u8>,
        relayer: address
    ) {
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow_mut<u32, cometbls_light_client::Client>(client_id).misbehaviour(misbehaviour, relayer);
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }

    public(package) fun get_timestamp_at_height(store: &LightClientManager, client_id: u32, height: u64): u64  {
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow<u32, cometbls_light_client::Client>(client_id).get_timestamp_at_height(height)
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }

    public(package) fun verify_non_membership(
        store: &LightClientManager,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        path: vector<u8>
    ): u64 {
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow<u32, cometbls_light_client::Client>(client_id).verify_non_membership(height, proof, path)
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }

    public(package) fun update_client(
        store: &mut LightClientManager,
        client_id: u32,
        clock: &Clock,
        client_msg: vector<u8>,
        relayer: address,
    ): (vector<u8>, vector<u8>, u64) {
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow_mut<u32, cometbls_light_client::Client>(client_id).update_client(clock, client_msg, relayer)
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }

    public(package) fun latest_height(
        store: &LightClientManager,
        client_id: u32
    ): u64 {
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow<u32, cometbls_light_client::Client>(client_id).latest_height()
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }

    public(package) fun verify_membership(
        store: &LightClientManager,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        key: vector<u8>,
        value: vector<u8>
    ): u64 {
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow<u32, cometbls_light_client::Client>(client_id).verify_membership(height, proof, key, value)
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }

    public(package) fun get_client_state(
        store: &LightClientManager,
        client_id: u32
    ): vector<u8> {        
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow<u32, cometbls_light_client::Client>(client_id).get_client_state()
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }

    public(package) fun get_consensus_state(
        store: &LightClientManager,
        client_id: u32,
        height: u64,
    ): vector<u8> {
        let client_type = store.client_id_to_type.borrow(client_id);
        if (client_type.bytes() == b"cometbls") {
            store.clients.borrow<u32, cometbls_light_client::Client>(client_id).get_consensus_state(height)
        } else {
            abort E_CLIENT_TYPE_NOT_SUPPORTED
        }
    }
}
