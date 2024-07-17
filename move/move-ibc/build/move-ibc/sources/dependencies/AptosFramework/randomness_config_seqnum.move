/// Randomness stall recovery utils.
///
/// When randomness generation is stuck due to a bug, the chain is also stuck. Below is the recovery procedure.
/// 1. Ensure more than 2/3 stakes are stuck at the same version.
/// 1. Every validator restarts with `randomness_override_seq_num` set to `X+1` in the node config file,
///    where `X` is the current `RandomnessConfigSeqNum` on chain.
/// 1. The chain should then be unblocked.
/// 1. Once the bug is fixed and the binary + framework have been patched,
///    a governance proposal is needed to set `RandomnessConfigSeqNum` to be `X+2`.
module aptos_framework::randomness_config_seqnum {
    use aptos_framework::config_buffer;
    use aptos_framework::system_addresses;

    friend aptos_framework::reconfiguration_with_dkg;

    /// If this seqnum is smaller than a validator local override, the on-chain `RandomnessConfig` will be ignored.
    /// Useful in a chain recovery from randomness stall.
    struct RandomnessConfigSeqNum has drop, key, store {
        seq_num: u64,
    }

    /// Update `RandomnessConfigSeqNum`.
    /// Used when re-enable randomness after an emergency randomness disable via local override.
    public fun set_for_next_epoch(framework: &signer, seq_num: u64) {
        system_addresses::assert_aptos_framework(framework);
        config_buffer::upsert(RandomnessConfigSeqNum { seq_num });
    }

    /// Initialize the configuration. Used in genesis or governance.
    public fun initialize(framework: &signer) {
        system_addresses::assert_aptos_framework(framework);
        if (!exists<RandomnessConfigSeqNum>(@aptos_framework)) {
            move_to(framework, RandomnessConfigSeqNum { seq_num: 0 })
        }
    }

    /// Only used in reconfigurations to apply the pending `RandomnessConfig`, if there is any.
    public(friend) fun on_new_epoch(framework: &signer) acquires RandomnessConfigSeqNum {
        system_addresses::assert_aptos_framework(framework);
        if (config_buffer::does_exist<RandomnessConfigSeqNum>()) {
            let new_config = config_buffer::extract<RandomnessConfigSeqNum>();
            if (exists<RandomnessConfigSeqNum>(@aptos_framework)) {
                *borrow_global_mut<RandomnessConfigSeqNum>(@aptos_framework) = new_config;
            } else {
                move_to(framework, new_config);
            }
        }
    }
}
