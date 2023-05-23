#![cfg(feature = "serde")]

use ethereum_consensus::altair::mainnet::SyncCommittee;
use ethereum_consensus::bellatrix::mainnet::ExecutionPayloadHeader;
use ethereum_consensus::phase0::mainnet::{
    Attestation, AttesterSlashing, BeaconState, Deposit, PendingAttestation, ProposerSlashing,
    SignedBeaconBlock, SignedVoluntaryExit,
};

fn main() {
    let mut block = SignedBeaconBlock::default();
    block
        .message
        .body
        .proposer_slashings
        .push(ProposerSlashing::default());
    block
        .message
        .body
        .attester_slashings
        .push(AttesterSlashing::default());
    block.message.body.attestations.push(Attestation::default());
    block.message.body.deposits.push(Deposit::default());
    block
        .message
        .body
        .voluntary_exits
        .push(SignedVoluntaryExit::default());

    let block_json = serde_json::to_string(&block).unwrap();
    println!("{block_json}");
    let _: SignedBeaconBlock = serde_json::from_str(&block_json).unwrap();

    let mut state = BeaconState::default();
    state
        .current_epoch_attestations
        .push(PendingAttestation::default());

    let state_json = serde_json::to_string(&state).unwrap();
    println!("{state_json}");
    let _: BeaconState = serde_json::from_str(&state_json).unwrap();

    let sync_committee = SyncCommittee::default();
    let sync_committee_json = serde_json::to_string(&sync_committee).unwrap();
    println!("{}", &sync_committee_json);
    let _: SyncCommittee = serde_json::from_str(&sync_committee_json).unwrap();

    let header = ExecutionPayloadHeader::default();
    let header_json = serde_json::to_string(&header).unwrap();
    println!("{}", &header_json);
    let _: ExecutionPayloadHeader = serde_json::from_str(&header_json).unwrap();
}
