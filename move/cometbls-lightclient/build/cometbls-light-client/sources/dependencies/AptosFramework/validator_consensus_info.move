/// Common type: `ValidatorConsensusInfo`.
module aptos_framework::validator_consensus_info {
    /// Information about a validator that participates consensus.
    struct ValidatorConsensusInfo has copy, drop, store {
        addr: address,
        pk_bytes: vector<u8>,
        voting_power: u64,
    }

    /// Create a default `ValidatorConsensusInfo` object. Value may be invalid. Only for place holding prupose.
    public fun default(): ValidatorConsensusInfo {
        ValidatorConsensusInfo {
            addr: @vm,
            pk_bytes: vector[],
            voting_power: 0,
        }
    }

    /// Create a `ValidatorConsensusInfo` object.
    public fun new(addr: address, pk_bytes: vector<u8>, voting_power: u64): ValidatorConsensusInfo {
        ValidatorConsensusInfo {
            addr,
            pk_bytes,
            voting_power,
        }
    }

    /// Get `ValidatorConsensusInfo.addr`.
    public fun get_addr(vci: &ValidatorConsensusInfo): address {
        vci.addr
    }

    /// Get `ValidatorConsensusInfo.pk_bytes`.
    public fun get_pk_bytes(vci: &ValidatorConsensusInfo): vector<u8> {
        vci.pk_bytes
    }

    /// Get `ValidatorConsensusInfo.voting_power`.
    public fun get_voting_power(vci: &ValidatorConsensusInfo): u64 {
        vci.voting_power
    }
}
