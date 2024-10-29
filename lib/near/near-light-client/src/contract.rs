use ibc_vm_rs::{IbcQuery, IbcResponse, Status};
use near_primitives_core::hash::CryptoHash;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, near_bindgen,
    store::LookupMap,
    PanicOnDefault,
};
#[allow(unused)]
use near_sdk_contract_tools::{owner::OwnerExternal, Owner};
use unionlabs::{
    ibc::core::{client::height::Height, commitment::merkle_path::MerklePath},
    id::ClientId,
    near::types::{
        ApprovalInner, BlockHeaderInnerLite, BlockHeaderInnerLiteView, HeaderUpdate,
        LightClientBlockView, PublicKey, Signature, ValidatorStakeView,
    },
};

use crate::{
    merkle::{self, combine_hash, hash_borsh},
    state_proof::RawStateProof,
    ClientState, ConsensusState,
};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    consensus_states: LookupMap<u64, ConsensusState>,
    client_state: ClientState,
    epoch_block_producers_map: LookupMap<CryptoHash, Vec<ValidatorStakeView>>,
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
        let mut block_producers = LookupMap::new(b"epoch_block_producers".as_slice());
        block_producers.insert(
            consensus_state.state.epoch_id,
            client_state.initial_block_producers.clone().unwrap(),
        );
        let mut consensus_states: LookupMap<u64, ConsensusState> = LookupMap::new(b"c");
        consensus_states.insert(client_state.latest_height, consensus_state);
        Self {
            client_state,
            consensus_states,
            epoch_block_producers_map: block_producers,
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
        Height::new(self.client_state.latest_height)
    }

    #[allow(unused)]
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
        let raw_state_proof: RawStateProof = serde_json::from_slice(&proof).unwrap();
        let state_proof = raw_state_proof.parse();
        let consensus_state = self.consensus_states.get(&(height.height() + 1)).unwrap();

        let key = key_from_path(&path.key_path[1]);

        if !state_proof.verify(
            &consensus_state.chunk_prev_state_root,
            &self.client_state.ibc_account_id,
            &key,
            Some(&borsh::to_vec(&value).unwrap()),
        ) {
            panic!("commitment verification failed");
        }

        true
    }

    // TODO(aeryz): client_msg can be Misbehaviour or Header
    pub fn verify_client_message(&self, client_msg: Vec<u8>) -> bool {
        let header_update: HeaderUpdate = borsh::from_slice(&client_msg).unwrap();

        let consensus_state = self
            .consensus_states
            .get(&header_update.trusted_height)
            .unwrap();

        validate_head(
            consensus_state.state.clone(),
            header_update.new_state.clone(),
            &self.epoch_block_producers_map,
        );

        merkle::verify_path(
            header_update.new_state.inner_lite.prev_state_root,
            &header_update.prev_state_root_proof,
            header_update.prev_state_root,
        )
    }

    #[allow(unused)]
    pub fn check_for_misbehaviour(&self, client_msg: Vec<u8>) -> bool {
        false
    }

    pub fn update_client(&mut self, client_msg: Vec<u8>) -> (Vec<u8>, Vec<(Height, Vec<u8>)>) {
        let header_update: HeaderUpdate = borsh::from_slice(&client_msg).unwrap();
        let new_consensus_state = ConsensusState {
            state: header_update.new_state.inner_lite.clone(),
            chunk_prev_state_root: header_update.prev_state_root,
        };
        self.consensus_states.insert(
            header_update.new_state.inner_lite.height,
            new_consensus_state.clone(),
        );
        self.client_state.latest_height = header_update.new_state.inner_lite.height;
        if let Some(next_bps) = &header_update.new_state.next_bps {
            self.epoch_block_producers_map.insert(
                header_update.new_state.inner_lite.next_epoch_id,
                next_bps.clone(),
            );
        }

        (
            borsh::to_vec(&self.client_state).unwrap(),
            vec![(
                Height::new(header_update.new_state.inner_lite.height),
                borsh::to_vec(&new_consensus_state).unwrap(),
            )],
        )
    }

    #[allow(unused)]
    pub fn update_client_on_misbehaviour(&mut self, client_msg: Vec<u8>) {}
}

fn key_from_path(path: &str) -> Vec<u8> {
    let mut commitments: Vec<u8> = Vec::new();
    commitments.extend(b"commitments");
    commitments.extend(borsh::to_vec(path).unwrap());
    commitments
}

fn reconstruct_light_client_block_view_fields(
    block_view: LightClientBlockView,
) -> (CryptoHash, CryptoHash, Vec<u8>) {
    let current_block_hash = combine_hash(
        &combine_hash(
            &CryptoHash(
                env::sha256(
                    &borsh::to_vec(&Into::<BlockHeaderInnerLite>::into(
                        block_view.inner_lite.clone(),
                    ))
                    .unwrap(),
                )
                .try_into()
                .unwrap(),
            ),
            &block_view.inner_rest_hash,
        ),
        &block_view.prev_block_hash,
    );

    let next_block_hash = combine_hash(&block_view.next_block_inner_hash, &current_block_hash);

    let mut approval_message = borsh::to_vec(&ApprovalInner::Endorsement(next_block_hash)).unwrap();
    approval_message.extend_from_slice(&(block_view.inner_lite.height + 2).to_le_bytes());

    (current_block_hash, next_block_hash, approval_message)
}

fn validate_head(
    head: BlockHeaderInnerLiteView,
    block_view: LightClientBlockView,
    epoch_block_producers_map: &LookupMap<CryptoHash, Vec<ValidatorStakeView>>,
) {
    let (_current_block_hash, _next_block_hash, approval_message) =
        reconstruct_light_client_block_view_fields(block_view.clone());

    assert!(block_view.inner_lite.height > head.height, "false");

    assert!(
        [&head.epoch_id, &head.next_epoch_id].contains(&&block_view.inner_lite.epoch_id),
        "false"
    );

    assert!(
        !(block_view.inner_lite.epoch_id == head.next_epoch_id && block_view.next_bps.is_none()),
        "false"
    );

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

                let PublicKey::Ed25519(pubkey) = block_producer.public_key else {
                    panic!("pubkey type is not supported");
                };

                assert!(
                    verify_signature(&pubkey[..], signature, &approval_message),
                    "invalid signature"
                );
            }
            None => continue,
        }
    }

    let threshold = total_stake.checked_mul(2).unwrap().checked_div(3).unwrap();
    assert!(approved_stake > threshold, "approved_stake <= threshold");

    if let Some(next_bps) = &block_view.next_bps {
        assert!(
            hash_borsh(next_bps) == block_view.inner_lite.next_bp_hash,
            "next bps hash mismatch"
        );
    }
}

fn verify_signature(public_key: &[u8], signature: &Signature, message: &[u8]) -> bool {
    let &Signature::Ed25519(sig) = &signature else {
        panic!("signature must be ed25519");
    };
    env::ed25519_verify(
        sig.as_slice().try_into().unwrap(),
        message,
        public_key.try_into().unwrap(),
    )
}
