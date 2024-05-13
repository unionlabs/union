use ibc_vm_rs::Status;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen, PanicOnDefault,
};
#[allow(unused)]
use near_sdk_contract_tools::owner::OwnerExternal;
#[allow(clippy::wildcard_imports)]
use near_sdk_contract_tools::Owner;
use unionlabs::{
    ibc::core::{client::height::Height, commitment::merkle_path::MerklePath},
    id::ClientId,
};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize, Owner)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn initialize(
        client_id: ClientId,
        client_state: Vec<u8>,
        consensus_state: Vec<u8>,
    ) -> Self {
        Self {}
    }

    pub fn status(&self) -> Status {
        Status::Active
    }

    pub fn latest_height(&self) -> Height {
        Height {
            revision_number: 0,
            revision_height: 100,
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
        true
    }
}
