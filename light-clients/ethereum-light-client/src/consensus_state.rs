use cosmwasm_std::Deps;
use unionlabs::{
    bls::BlsPublicKey,
    cosmwasm::wasm::union::custom_query::{query_aggregate_public_keys, UnionCustomQuery},
    ensure,
    ethereum::config::ChainSpec,
    ibc::lightclients::ethereum::{
        consensus_state::ConsensusState, trusted_sync_committee::ActiveSyncCommittee,
    },
};

use crate::errors::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct TrustedConsensusState<C: ChainSpec> {
    pub state: ConsensusState,
    /// Full sync committee data which corresponds to the aggregate key that we
    /// store at the client.
    ///
    /// This sync committee can either be the current sync committee or the next sync
    /// committee. That's because the verifier uses next or current sync committee's
    /// public keys to verify the signature against. It is based on
    pub sync_committee: ActiveSyncCommittee<C>,
}

impl<C: ChainSpec> TrustedConsensusState<C> {
    pub fn new(
        deps: Deps<UnionCustomQuery>,
        consensus_state: ConsensusState,
        sync_committee: ActiveSyncCommittee<C>,
    ) -> Result<Self, Error> {
        let (active_sync_committee, given_committee) = match &sync_committee {
            ActiveSyncCommittee::Current(committee) => {
                (consensus_state.current_sync_committee, committee)
            }
            ActiveSyncCommittee::Next(committee) => (
                consensus_state
                    .next_sync_committee
                    .ok_or(Error::NoNextSyncCommittee)?,
                committee,
            ),
        };

        let aggregate_public_key =
            query_aggregate_public_keys(deps, given_committee.pubkeys.clone().into())?;

        // We are making sure that the given trusted sync committee actually matches
        // the sync committee that we stored
        ensure(
            active_sync_committee == given_committee.aggregate_pubkey
                && given_committee.aggregate_pubkey == aggregate_public_key,
            Error::TrustedSyncCommitteeMismatch {
                stored_aggregate: active_sync_committee,
                given_aggregate: given_committee.aggregate_pubkey,
            },
        )?;

        Ok(TrustedConsensusState {
            state: consensus_state,
            sync_committee,
        })
    }

    pub fn current_sync_committee_aggregate_key(&self) -> BlsPublicKey {
        self.state.current_sync_committee
    }
}
