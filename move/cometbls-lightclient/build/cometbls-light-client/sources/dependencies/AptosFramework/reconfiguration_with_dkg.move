/// Reconfiguration with DKG helper functions.
module aptos_framework::reconfiguration_with_dkg {
    use std::features;
    use std::option;
    use aptos_framework::consensus_config;
    use aptos_framework::dkg;
    use aptos_framework::execution_config;
    use aptos_framework::gas_schedule;
    use aptos_framework::jwk_consensus_config;
    use aptos_framework::jwks;
    use aptos_framework::keyless_account;
    use aptos_framework::randomness_api_v0_config;
    use aptos_framework::randomness_config;
    use aptos_framework::randomness_config_seqnum;
    use aptos_framework::reconfiguration;
    use aptos_framework::reconfiguration_state;
    use aptos_framework::stake;
    use aptos_framework::system_addresses;
    friend aptos_framework::block;
    friend aptos_framework::aptos_governance;

    /// Trigger a reconfiguration with DKG.
    /// Do nothing if one is already in progress.
    public(friend) fun try_start() {
        let incomplete_dkg_session = dkg::incomplete_session();
        if (option::is_some(&incomplete_dkg_session)) {
            let session = option::borrow(&incomplete_dkg_session);
            if (dkg::session_dealer_epoch(session) == reconfiguration::current_epoch()) {
                return
            }
        };
        reconfiguration_state::on_reconfig_start();
        let cur_epoch = reconfiguration::current_epoch();
        dkg::start(
            cur_epoch,
            randomness_config::current(),
            stake::cur_validator_consensus_infos(),
            stake::next_validator_consensus_infos(),
        );
    }

    /// Clear incomplete DKG session, if it exists.
    /// Apply buffered on-chain configs (except for ValidatorSet, which is done inside `reconfiguration::reconfigure()`).
    /// Re-enable validator set changes.
    /// Run the default reconfiguration to enter the new epoch.
    public(friend) fun finish(framework: &signer) {
        system_addresses::assert_aptos_framework(framework);
        dkg::try_clear_incomplete_session(framework);
        consensus_config::on_new_epoch(framework);
        execution_config::on_new_epoch(framework);
        gas_schedule::on_new_epoch(framework);
        std::version::on_new_epoch(framework);
        features::on_new_epoch(framework);
        jwk_consensus_config::on_new_epoch(framework);
        jwks::on_new_epoch(framework);
        keyless_account::on_new_epoch(framework);
        randomness_config_seqnum::on_new_epoch(framework);
        randomness_config::on_new_epoch(framework);
        randomness_api_v0_config::on_new_epoch(framework);
        reconfiguration::reconfigure();
    }

    /// Complete the current reconfiguration with DKG.
    /// Abort if no DKG is in progress.
    fun finish_with_dkg_result(account: &signer, dkg_result: vector<u8>) {
        dkg::finish(dkg_result);
        finish(account);
    }
}
