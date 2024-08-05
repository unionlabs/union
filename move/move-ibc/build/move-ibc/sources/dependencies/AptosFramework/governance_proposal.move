/// Define the GovernanceProposal that will be used as part of on-chain governance by AptosGovernance.
///
/// This is separate from the AptosGovernance module to avoid circular dependency between AptosGovernance and Stake.
module aptos_framework::governance_proposal {
    friend aptos_framework::aptos_governance;

    struct GovernanceProposal has store, drop {}

    /// Create and return a GovernanceProposal resource. Can only be called by AptosGovernance
    public(friend) fun create_proposal(): GovernanceProposal {
        GovernanceProposal {}
    }

    /// Useful for AptosGovernance to create an empty proposal as proof.
    public(friend) fun create_empty_proposal(): GovernanceProposal {
        create_proposal()
    }

    #[test_only]
    public fun create_test_proposal(): GovernanceProposal {
        create_empty_proposal()
    }
}
