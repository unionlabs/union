module ibc::light_client {
    use ibc::cometbls_lc;
    use ibc::state_lens_ics23_mpt_lc;
    use ibc::state_lens_ics23_ics23_lc;
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
    ): (vector<u8>, vector<u8>) {
        if (string::bytes(&client_type) == &CLIENT_TYPE_COMETBLS) {
            let (client_state, consensus_state) =
                cometbls_lc::create_client(
                    ibc_signer,
                    client_id,
                    client_state_bytes,
                    consensus_state_bytes
                );
            return (client_state, consensus_state)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_MPT) {
            let (client_state, consensus_state) =
                state_lens_ics23_mpt_lc::create_client(
                    ibc_signer,
                    client_id,
                    client_state_bytes,
                    consensus_state_bytes
                );
            return (client_state, consensus_state)
        } else if (string::bytes(&client_type) == &CLIENT_TYPE_STATE_LENS_ICS23_ICS23) {
            let (client_state, consensus_state) =
                state_lens_ics23_ics23_lc::create_client(
                    ibc_signer,
                    client_id,
                    client_state_bytes,
                    consensus_state_bytes
                );
            return (client_state, consensus_state)
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
