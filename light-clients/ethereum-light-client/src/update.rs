use ethereum_verifier::{
    compute_sync_committee_period_at_slot, compute_timestamp_at_slot, primitives::Hash32,
    LightClientContext,
};
use ibc_types::ibc::lightclients::{
    ethereum::{self, light_client_update::LightClientUpdate},
    wasm,
};

use crate::errors::Error;

// TODO(aeryz): This is an ethereum spec implementation. Although implementing this in here is more performant, we might also
// consider moving this to `ethereum-verifier`, and calling it here, so that all spec related updates/verifications will be at
// the same library.
pub fn apply_light_client_update<Ctx: LightClientContext>(
    client_state: &mut wasm::client_state::ClientState<ethereum::client_state::ClientState>,
    consensus_state: &mut wasm::consensus_state::ConsensusState<
        ethereum::consensus_state::ConsensusState,
    >,
    consensus_update: LightClientUpdate<Ctx::ChainSpec>,
    storage_root: Hash32,
) -> Result<(), Error> {
    let store_period =
        compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(consensus_state.data.slot);
    let update_finalized_period = compute_sync_committee_period_at_slot::<Ctx::ChainSpec>(
        consensus_update.attested_header.beacon.slot,
    );

    match consensus_state.data.next_sync_committee {
        None if update_finalized_period != store_period => {
            return Err(Error::StorePeriodMustBeEqualToFinalizedPeriod)
        }
        None => {
            consensus_state.data.next_sync_committee = consensus_update
                .next_sync_committee
                .map(|c| c.aggregate_pubkey);
        }
        Some(ref next_sync_committee) if update_finalized_period == store_period + 1 => {
            consensus_state.data.current_sync_committee = next_sync_committee.clone();
            consensus_state.data.next_sync_committee = consensus_update
                .next_sync_committee
                .map(|c| c.aggregate_pubkey);
        }
        _ => {}
    }

    if consensus_update.attested_header.beacon.slot > consensus_state.data.slot {
        consensus_state.data.slot = consensus_update.attested_header.beacon.slot;
        // NOTE(aeryz): we don't use `optimistic_header`
    }

    // We implemented the spec until this point. We apply our updates now.
    consensus_state.data.storage_root = storage_root.into();

    consensus_state.timestamp = compute_timestamp_at_slot::<Ctx::ChainSpec>(
        client_state.data.genesis_time,
        consensus_update.finalized_header.beacon.slot,
    );

    if client_state.data.latest_slot < consensus_update.attested_header.beacon.slot {
        client_state.data.latest_slot = consensus_update.attested_header.beacon.slot;
    }

    Ok(())
}
