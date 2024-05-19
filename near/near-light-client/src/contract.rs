use ibc_vm_rs::{IbcQuery, IbcResponse, Status};
use near_primitives_core::hash::CryptoHash;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, near_bindgen,
    store::LookupMap,
    PanicOnDefault,
};
#[allow(unused)]
use near_sdk_contract_tools::owner::OwnerExternal;
#[allow(clippy::wildcard_imports)]
use near_sdk_contract_tools::Owner;
use unionlabs::{
    ibc::core::{client::height::Height, commitment::merkle_path::MerklePath},
    id::ClientId,
};

use crate::{
    types::{ApprovalInner, LightClientBlockView, Signature, ValidatorStakeView},
    ClientState, ConsensusState, StateProof,
};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    consensus_states: LookupMap<u64, ConsensusState>,
    client_state: ClientState,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn initialize(
        #[allow(unused)] client_id: ClientId,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    ) -> Self {
        let client_state: ClientState = borsh::from_slice(&client_state).unwrap();
        let consensus_state: ConsensusState = borsh::from_slice(&consensus_state).unwrap();
        let mut consensus_states: LookupMap<u64, ConsensusState> = LookupMap::new(b"c");
        consensus_states.insert(client_state.latest_height, consensus_state);
        Self {
            client_state,
            consensus_states,
        }
    }

    pub fn query(&self, query: Vec<IbcQuery>) -> Vec<IbcResponse> {
        query
            .into_iter()
            .map(|q| match q {
                IbcQuery::Status => IbcResponse::Status {
                    status: self.status(),
                },
                IbcQuery::LatestHeight => IbcResponse::LatestHeight {
                    height: self.latest_height(),
                },
                IbcQuery::VerifyMembership {
                    height,
                    delay_time_period,
                    delay_block_period,
                    proof,
                    path,
                    value,
                } => IbcResponse::VerifyMembership {
                    valid: self.verify_membership(
                        height,
                        delay_time_period,
                        delay_block_period,
                        proof,
                        path,
                        value,
                    ),
                },
                IbcQuery::VerifyClientMessage(msg) => IbcResponse::VerifyClientMessage {
                    valid: self.verify_client_message(msg),
                },
                IbcQuery::CheckForMisbehaviour(msg) => IbcResponse::CheckForMisbehaviour {
                    misbehaviour_found: self.check_for_misbehaviour(msg),
                },
                IbcQuery::TimestampAtHeight(_) => IbcResponse::TimestampAtHeight { timestamp: 100 },
            })
            .collect()
    }

    pub fn status(&self) -> Status {
        Status::Active
    }

    pub fn latest_height(&self) -> Height {
        Height {
            revision_number: 0,
            revision_height: self.client_state.latest_height,
        }
    }

    pub fn verify_membership(
        &self,
        height: Height,
        // TODO(aeryz): delay times might not be relevant for other chains we could make it optional
        delay_time_period: u64,
        delay_block_period: u64,
        proof: Vec<u8>,
        path: MerklePath,
        value: Vec<u8>,
    ) -> bool {
        let data: Vec<Vec<u8>> = serde_json::from_slice(&proof).unwrap();
        let state_proof = StateProof::parse(data);
        let consensus_state = self.consensus_states.get(&height.revision_height).unwrap();

        let key = key_from_path(&path.key_path[1]);

        state_proof.verify(
            &consensus_state.state_root,
            &self.client_state.ibc_account_id,
            &key,
            Some(&borsh::to_vec(&value).unwrap()),
        )
    }

    pub fn verify_client_message(&self, client_msg: Vec<u8>) -> bool {
        true
    }

    pub fn check_for_misbehaviour(&self, client_msg: Vec<u8>) -> bool {
        false
    }

    pub fn update_client(&mut self, client_msg: Vec<u8>) -> (Vec<u8>, Vec<(Height, Vec<u8>)>) {
        let consensus_state: (u64, ConsensusState) = borsh::from_slice(&client_msg).unwrap();
        self.consensus_states
            .insert(consensus_state.0, consensus_state.1.clone());
        self.client_state.latest_height = consensus_state.0;

        (
            borsh::to_vec(&self.client_state).unwrap(),
            vec![(
                Height {
                    revision_number: 0,
                    revision_height: consensus_state.0,
                },
                borsh::to_vec(&consensus_state.1).unwrap(),
            )],
        )
    }

    pub fn update_client_on_misbehaviour(&mut self, client_msg: Vec<u8>) {}
}

fn key_from_path(path: &str) -> Vec<u8> {
    let mut commitments: Vec<u8> = Vec::new();
    commitments.extend(b"commitments");
    commitments.extend(borsh::to_vec(path).unwrap());
    commitments
}

// def reconstruct_light_client_block_view_fields(block_view):
//     current_block_hash = sha256(concat(
//         sha256(concat(
//             sha256(borsh(block_view.inner_lite)),
//             block_view.inner_rest_hash,
//         )),
//         block_view.prev_block_hash
//     ))

//     next_block_hash = sha256(concat(
//         block_view.next_block_inner_hash,
//         current_block_hash
//     ))

//     approval_message = concat(
//         borsh(ApprovalInner::Endorsement(next_block_hash)),
//         little_endian(block_view.inner_lite.height + 2)
//     )

//     return (current_block_hash, next_block_hash, approval_message)

fn reconstruct_light_client_block_view_fields(
    block_view: LightClientBlockView,
) -> (CryptoHash, CryptoHash, Vec<u8>) {
    let concat = |first: &[u8], second: &[u8]| [first, second].concat();

    let current_block_hash = CryptoHash(
        env::sha256(&concat(
            &env::sha256(&concat(
                &env::sha256(&borsh::to_vec(&block_view.inner_lite).unwrap()),
                block_view.inner_rest_hash.as_bytes(),
            )),
            block_view.prev_block_hash.as_bytes(),
        ))
        .try_into()
        .unwrap(),
    );

    let next_block_hash = CryptoHash(
        env::sha256(&concat(
            block_view.next_block_inner_hash.as_bytes(),
            current_block_hash.as_bytes(),
        ))
        .try_into()
        .unwrap(),
    );

    let approval_message = concat(
        &borsh::to_vec(&ApprovalInner::Endorsement(next_block_hash.clone())).unwrap(),
        &(block_view.inner_lite.height + 2).to_le_bytes(),
    );

    (current_block_hash, next_block_hash, approval_message)
}

fn validate_head(
    head: LightClientBlockView,
    block_view: LightClientBlockView,
    epoch_block_producers_map: &mut LookupMap<CryptoHash, Vec<ValidatorStakeView>>,
) {
    let (_current_block_hash, _next_block_hash, approval_message) =
        reconstruct_light_client_block_view_fields(block_view.clone());

    if block_view.inner_lite.height <= head.inner_lite.height {
        panic!("false");
    }

    if ![&head.inner_lite.epoch_id, &head.inner_lite.next_epoch_id]
        .contains(&&block_view.inner_lite.epoch_id)
    {
        panic!("false");
    }

    if block_view.inner_lite.epoch_id == head.inner_lite.next_epoch_id
        && block_view.next_bps.is_none()
    {
        panic!("false");
    }

    let mut total_stake = 0;
    let mut approved_stake = 0;

    let epoch_block_producers = epoch_block_producers_map
        .get(&block_view.inner_lite.epoch_id)
        .expect("noo");
    for (maybe_signature, block_producer) in block_view
        .approvals_after_next
        .iter()
        .zip(epoch_block_producers.iter())
    {
        let ValidatorStakeView::V1(block_producer) = block_producer.clone();
        total_stake += block_producer.stake;

        if maybe_signature.is_none() {
            continue;
        }

        match maybe_signature {
            Some(signature) => {
                approved_stake += block_producer.stake;

                if !verify_signature(&block_producer.public_key, signature, &approval_message) {
                    panic!("no bro no");
                }
            }
            None => continue,
        }
    }

    let threshold = total_stake.checked_mul(2).unwrap().checked_div(3).unwrap();
    if approved_stake <= threshold {
        panic!("not cool bro");
    }

    if let Some(next_bps) = &block_view.next_bps {
        if env::sha256(&borsh::to_vec(next_bps).unwrap()) != block_view.inner_lite.next_bp_hash.0 {
            panic!("no bro no");
        }
        epoch_block_producers_map.insert(block_view.inner_lite.next_epoch_id, next_bps.clone());
    }
}

fn verify_signature(_public_key: &Vec<u8>, _signature: &Signature, _message: &Vec<u8>) -> bool {
    true
}
