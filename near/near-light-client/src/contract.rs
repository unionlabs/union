use ibc_vm_rs::Status;
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

use crate::{ClientState, ConsensusState, StateProof};

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
        client_id: ClientId,
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
        client_id: ClientId,
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
