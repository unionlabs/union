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
use near_verifier::{state_proof::RawStateProof, NearVerifierCtx};
use unionlabs::{
    ibc::{
        core::{client::height::Height, commitment::merkle_path::MerklePath},
        lightclients::near::{
            client_state::ClientState, consensus_state::ConsensusState, header::Header,
            validator_stake::ValidatorStakeView,
        },
    },
    id::ClientId,
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
        Height {
            revision_number: 0,
            revision_height: self.client_state.latest_height,
        }
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
        let proof: RawStateProof = serde_json::from_slice(&proof).unwrap();
        let consensus_state = self
            .consensus_states
            .get(&(height.revision_height + 1))
            .unwrap();

        let key = key_from_path(&path.key_path[1]);

        near_verifier::verify_state(
            proof,
            &consensus_state.chunk_prev_state_root,
            &self.client_state.ibc_account_id,
            &key,
            Some(&borsh::to_vec(&value).unwrap()),
        )
        .unwrap();

        true
    }

    // TODO(aeryz): client_msg can be Misbehaviour or Header
    pub fn verify_client_message(&self, client_msg: Vec<u8>) -> bool {
        let header: Header = borsh::from_slice(&client_msg).unwrap();

        let consensus_state = self.consensus_states.get(&header.trusted_height).unwrap();

        near_verifier::verify_header(
            self,
            consensus_state.state.clone(),
            header.new_state.clone(),
        )
        .unwrap();

        near_verifier::verify_path(
            header.new_state.inner_lite.prev_state_root,
            &header.prev_state_root_proof,
            header.prev_state_root,
        )
        .unwrap();

        true
    }

    #[allow(unused)]
    pub fn check_for_misbehaviour(&self, client_msg: Vec<u8>) -> bool {
        false
    }

    pub fn update_client(&mut self, client_msg: Vec<u8>) -> (Vec<u8>, Vec<(Height, Vec<u8>)>) {
        let header: Header = borsh::from_slice(&client_msg).unwrap();
        let new_consensus_state = ConsensusState {
            state: header.new_state.inner_lite.clone(),
            chunk_prev_state_root: header.prev_state_root,
            timestamp: header.new_state.inner_lite.timestamp_nanosec,
        };
        self.consensus_states.insert(
            header.new_state.inner_lite.height,
            new_consensus_state.clone(),
        );
        self.client_state.latest_height = header.new_state.inner_lite.height;
        if let Some(next_bps) = &header.new_state.next_bps {
            self.epoch_block_producers_map
                .insert(header.new_state.inner_lite.next_epoch_id, next_bps.clone());
        }

        (
            borsh::to_vec(&self.client_state).unwrap(),
            vec![(
                Height {
                    revision_number: 0,
                    revision_height: header.new_state.inner_lite.height,
                },
                borsh::to_vec(&new_consensus_state).unwrap(),
            )],
        )
    }

    #[allow(unused)]
    pub fn update_client_on_misbehaviour(&mut self, client_msg: Vec<u8>) {}
}

impl NearVerifierCtx for Contract {
    fn get_epoch_block_producers(
        &self,
        epoch_id: CryptoHash,
    ) -> Option<Vec<unionlabs::ibc::lightclients::near::validator_stake::ValidatorStakeView>> {
        self.epoch_block_producers_map
            .get(&epoch_id)
            .map(|bps| bps.clone())
    }

    fn ed25519_verify(
        &self,
        public_key: &[u8],
        signature: &[u8],
        message: &[u8],
    ) -> Result<(), near_verifier::error::Error> {
        if env::ed25519_verify(
            signature.try_into().unwrap(),
            message,
            public_key.try_into().unwrap(),
        ) {
            Ok(())
        } else {
            Err(near_verifier::error::Error::VerificationFailure(
                public_key.into(),
                signature.into(),
                message.into(),
            ))
        }
    }
}

fn key_from_path(path: &str) -> Vec<u8> {
    let mut commitments: Vec<u8> = Vec::new();
    commitments.extend(b"commitments");
    commitments.extend(borsh::to_vec(path).unwrap());
    commitments
}
