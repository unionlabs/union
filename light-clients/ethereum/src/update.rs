use ethereum_consensus::{sync_protocol::EXECUTION_PAYLOAD_DEPTH, types::H256};

// TODO(aeryz): ConsensusUpdateInfo can define `new` function in `ethereum_light_client_verifier`

pub type LightClientUpdate<const SYNC_COMMITTEE_SIZE: usize> =
    ethereum_consensus::bellatrix::LightClientUpdate<SYNC_COMMITTEE_SIZE>;
pub type ConsensusUpdateInfo<const SYNC_COMMITTEE_SIZE: usize> =
    ethereum_light_client_verifier::updates::bellatrix::ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE>;
pub type ExecutionUpdateInfo =
    ethereum_light_client_verifier::updates::bellatrix::ExecutionUpdateInfo;

pub fn new_consensus_update<const SYNC_COMMITTEE_SIZE: usize>(
    light_client_update: LightClientUpdate<SYNC_COMMITTEE_SIZE>,
    finalized_execution_root: H256,
    finalized_execution_branch: Vec<H256>,
) -> ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE> {
    let mut branch: [H256; EXECUTION_PAYLOAD_DEPTH] = Default::default();
    branch.clone_from_slice(&finalized_execution_branch);
    ConsensusUpdateInfo {
        light_client_update,
        finalized_execution_root,
        finalized_execution_branch: branch,
    }
}
