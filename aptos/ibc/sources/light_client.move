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

module ibc::light_client {
    use ibc::cometbls_lc;
    use ibc::state_lens_ics23_mpt_lc;
    use ibc::state_lens_ics23_ics23_lc;
    use ibc::consensus_state_update::{ConsensusStateUpdate, Self};
    use std::string::{Self, String};

    const E_UNKNOWN_CLIENT_TYPE: u64 = 1;
    const CLIENT_TYPE_STATE_LENS_ICS23_MPT: vector<u8> = b"state-lens/ics23/mpt";
    const CLIENT_TYPE_STATE_LENS_ICS23_ICS23: vector<u8> = b"state-lens/ics23/ics23";
    const CLIENT_TYPE_COMETBLS: vector<u8> = b"cometbls";

    public fun create_client(
        client_type: String,
        ibc_signer: &signer,
        client_id: u32,
        client_state_bytes: vector<u8>,
        consensus_state_bytes: vector<u8>
    ): (ConsensusStateUpdate, String) {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            let (client_state, counterpaty_chain_id) =
                cometbls_lc::create_client(
                    ibc_signer,
                    client_id,
                    client_state_bytes,
                    consensus_state_bytes
                );
            return (client_state, counterpaty_chain_id)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            let (client_state, counterpaty_chain_id) =
                state_lens_ics23_mpt_lc::create_client(
                    ibc_signer,
                    client_id,
                    client_state_bytes,
                    consensus_state_bytes
                );
            return (client_state, counterpaty_chain_id)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            let (client_state, counterpaty_chain_id) =
                state_lens_ics23_ics23_lc::create_client(
                    ibc_signer,
                    client_id,
                    client_state_bytes,
                    consensus_state_bytes
                );
            return (client_state, counterpaty_chain_id)
        };
        abort E_UNKNOWN_CLIENT_TYPE

    }

    public fun status(client_type: String, client_id: u32): u64 {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::status(client_id)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::status(client_id)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::status(client_id)
        };
        abort E_UNKNOWN_CLIENT_TYPE

    }

    public fun latest_height(client_type: String, client_id: u32): u64 {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::latest_height(client_id)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::latest_height(client_id)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::latest_height(client_id)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }

    public fun check_for_misbehaviour(
        client_type: String, client_id: u32, header: vector<u8>
    ): bool {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::check_for_misbehaviour(client_id, header)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::check_for_misbehaviour(client_id, header)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::check_for_misbehaviour(client_id, header)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }

    public fun update_client(
        client_type: String, client_id: u32, client_msg: vector<u8>
    ): (vector<u8>, vector<vector<u8>>, vector<u64>) {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::update_client(client_id, client_msg)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::update_client(client_id, client_msg)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::update_client(client_id, client_msg)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }

    public fun report_misbehaviour(
        client_type: String, client_id: u32, misbehaviour: vector<u8>
    ) {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            cometbls_lc::report_misbehaviour(client_id, misbehaviour)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            state_lens_ics23_mpt_lc::report_misbehaviour(client_id, misbehaviour)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            state_lens_ics23_ics23_lc::report_misbehaviour(client_id, misbehaviour)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }

    public fun get_timestamp_at_height(
        client_type: String, client_id: u32, height: u64
    ): u64 {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::get_timestamp_at_height(client_id, height)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::get_timestamp_at_height(client_id, height)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::get_timestamp_at_height(client_id, height)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }

    public fun get_client_state(client_type: String, client_id: u32): vector<u8> {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::get_client_state(client_id)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::get_client_state(client_id)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::get_client_state(client_id)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }

    public fun get_consensus_state(
        client_type: String, client_id: u32, height: u64
    ): vector<u8> {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::get_consensus_state(client_id, height)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::get_consensus_state(client_id, height)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::get_consensus_state(client_id, height)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }

    public fun verify_membership(
        client_type: String,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        key: vector<u8>,
        value: vector<u8>
    ): u64 {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::verify_membership(client_id, height, proof, key, value)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::verify_membership(client_id, height, proof, key, value)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::verify_membership(client_id, height, proof, key, value)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }

    public fun verify_non_membership(
        client_type: String,
        client_id: u32,
        height: u64,
        proof: vector<u8>,
        path: vector<u8>
    ): u64 {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            return cometbls_lc::verify_non_membership(client_id, height, proof, path)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            return state_lens_ics23_mpt_lc::verify_non_membership(client_id, height, proof, path)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            return state_lens_ics23_ics23_lc::verify_non_membership(client_id, height, proof, path)
        };
        abort E_UNKNOWN_CLIENT_TYPE
    }
}
