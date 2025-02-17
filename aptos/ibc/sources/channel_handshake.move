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

module ibc::channel_handshake {
    use std::string::{String};
    use ibc::ibc;
    use ibc::dispatcher;
    use ibc::engine;
    use ibc::helpers;

    public entry fun channel_open_init<T: key + store + drop>(
        port_id: address,
        counterparty_port_id: vector<u8>,
        connection_id: u32,
        version: String
    ) {
        let (channel_id, connection_id) =
            ibc::channel_open_init<T>(
                port_id,
                counterparty_port_id,
                connection_id,
                version
            );

        engine::dispatch<T>(
            helpers::pack_channel_open_init_params(connection_id, channel_id, version)
        );

        dispatcher::delete_storage<T>();
    }

    public entry fun channel_open_try<T: key + store + drop>(
        port_id: address,
        connection_id: u32,
        counterparty_channel_id: u32,
        counterparty_port_id: vector<u8>,
        version: String,
        counterparty_version: String,
        proof_init: vector<u8>,
        proof_height: u64
    ) {
        let channel_id =
            ibc::channel_open_try<T>(
                port_id,
                connection_id,
                counterparty_channel_id,
                counterparty_port_id,
                version,
                counterparty_version,
                proof_init,
                proof_height
            );

        engine::dispatch<T>(
            helpers::pack_channel_open_try_params(
                connection_id,
                channel_id,
                counterparty_channel_id,
                version,
                counterparty_version
            )
        );

        dispatcher::delete_storage<T>();
    }

    public entry fun channel_open_ack<T: key + store + drop>(
        port_id: address,
        channel_id: u32,
        counterparty_version: String,
        counterparty_channel_id: u32,
        proof_try: vector<u8>,
        proof_height: u64
    ) {
        ibc::channel_open_ack<T>(
            port_id,
            channel_id,
            counterparty_version,
            counterparty_channel_id,
            proof_try,
            proof_height
        );

        engine::dispatch<T>(
            helpers::pack_channel_open_ack_params(
                channel_id,
                counterparty_channel_id,
                counterparty_version
            )
        );

        dispatcher::delete_storage<T>();
    }

    public entry fun channel_open_confirm<T: key + store + drop>(
        port_id: address,
        channel_id: u32,
        proof_ack: vector<u8>,
        proof_height: u64
    ) {
        ibc::channel_open_confirm<T>(port_id, channel_id, proof_ack, proof_height);

        engine::dispatch<T>(helpers::pack_channel_open_confirm_params(channel_id));

        dispatcher::delete_storage<T>();
    }
}
