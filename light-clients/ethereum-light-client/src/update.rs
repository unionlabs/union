use crate::{
    client_state::ClientState, consensus_state::ConsensusState, errors::Error,
    eth_types::LightClientUpdate,
};
use ethereum_verifier::{
    compute_sync_committee_period_at_slot, compute_timestamp_at_slot, primitives::Hash32,
    LightClientContext,
};
use ibc::core::ics23_commitment::commitment::CommitmentRoot;

// TODO(aeryz): This is an ethereum spec implementation. Although implementing this in here is more performant, we might also
// consider moving this to `ethereum-verifier`, and calling it here, so that all spec related updates/verifications will be at
// the same library.
pub fn apply_light_client_update<C: LightClientContext>(
    client_state: &mut ClientState,
    consensus_state: &mut ConsensusState,
    consensus_update: LightClientUpdate,
    storage_root: Hash32,
) -> Result<(), Error> {
    let store_period = compute_sync_committee_period_at_slot(consensus_state.slot);
    let update_finalized_period =
        compute_sync_committee_period_at_slot(consensus_update.finalized_header.beacon.slot);

    match consensus_state.next_sync_committee {
        None if update_finalized_period != store_period => {
            return Err(Error::StorePeriodMustBeEqualToFinalizedPeriod)
        }
        None => {
            consensus_state.next_sync_committee = consensus_update
                .next_sync_committee
                .map(|c| c.aggregate_public_key);
        }
        Some(ref next_sync_committee) if update_finalized_period == store_period + 1 => {
            consensus_state.current_sync_committee = next_sync_committee.clone();
            consensus_state.next_sync_committee = consensus_update
                .next_sync_committee
                .map(|c| c.aggregate_public_key);
        }
        _ => {}
    }

    if consensus_update.attested_header.beacon.slot > consensus_state.slot {
        consensus_state.slot = consensus_update.attested_header.beacon.slot;
        // NOTE(aeryz): we don't use `optimistic_header`
    }

    // We implemented the spec until, this point. We apply our updates now.
    consensus_state.storage_root = CommitmentRoot::from_bytes(storage_root.as_ref());

    consensus_state.timestamp = compute_timestamp_at_slot(
        client_state.genesis_time,
        consensus_update.finalized_header.beacon.slot,
    );

    if client_state.latest_slot < consensus_update.finalized_header.beacon.slot {
        client_state.latest_slot = consensus_update.finalized_header.beacon.slot;
    }

    Ok(())
}
