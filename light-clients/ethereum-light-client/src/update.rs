use ethereum_verifier::{compute_sync_committee_period_at_slot, compute_timestamp_at_slot};
use ibc::core::ics23_commitment::commitment::CommitmentRoot;

use crate::{
    client_state::ClientState,
    consensus_state::{ConsensusState, TrustedConsensusState},
    errors::Error,
    eth_types::LightClientUpdate,
    types::AccountUpdateInfo,
};

pub fn apply_updates(
    client_state: &ClientState,
    trusted_consensus_state: &TrustedConsensusState,
    consensus_update: LightClientUpdate,
    // execution_update: ExecutionUpdateInfo, // TODO(aeryz)
    account_update: AccountUpdateInfo,
) -> Result<(ClientState, ConsensusState), Error> {
    let mut new_client_state = client_state.clone();

    let store_period = compute_sync_committee_period_at_slot(trusted_consensus_state.state.slot);
    let update_period =
        compute_sync_committee_period_at_slot(consensus_update.finalized_header.beacon.slot);
    let finalized_header_timestamp: u64 = compute_timestamp_at_slot(
        client_state.genesis_time,
        consensus_update.finalized_header.beacon.slot,
    );

    if client_state.latest_slot < consensus_update.finalized_header.beacon.slot {
        new_client_state.latest_slot = consensus_update.finalized_header.beacon.slot;
    }
    // if client_state.latest_execution_block_number < execution_update.block_number {
    //     new_client_state.latest_execution_block_number = execution_update.block_number;
    // }

    let new_consensus_state = if store_period == update_period {
        ConsensusState {
            slot: consensus_update.finalized_header.beacon.slot,
            storage_root: CommitmentRoot::from_bytes(account_update.account_storage_root.as_ref()),
            timestamp: finalized_header_timestamp,
            current_sync_committee: trusted_consensus_state.current_sync_committee_aggregate_key(),
            next_sync_committee: consensus_update
                .next_sync_committee
                .as_ref()
                .map(|c| c.aggregate_public_key.clone()),
        }
    } else if store_period + 1 == update_period {
        ConsensusState {
            slot: consensus_update.finalized_header.beacon.slot,
            storage_root: CommitmentRoot::from_bytes(account_update.account_storage_root.as_ref()),
            timestamp: finalized_header_timestamp,
            current_sync_committee: trusted_consensus_state
                .next_sync_committee
                .as_ref()
                .ok_or(Error::NoNextSyncCommittee)?
                .aggregate_public_key
                .clone(),
            next_sync_committee: consensus_update
                .next_sync_committee
                .as_ref()
                .map(|c| c.aggregate_public_key.clone()),
        }
    } else {
        // store_period + 1 < update_period
        return Err(Error::FuturePeriod);
    };

    Ok((new_client_state, new_consensus_state))
}
