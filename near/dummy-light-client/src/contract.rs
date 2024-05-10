use ibc_vm_rs::Status;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen, PanicOnDefault,
};
#[allow(unused)]
use near_sdk_contract_tools::owner::OwnerExternal;
#[allow(clippy::wildcard_imports)]
use near_sdk_contract_tools::Owner;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize, Owner)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn initialize(client_id: String, client_state: Vec<u8>, consensus_state: Vec<u8>) -> Self {
        Self {}
    }

    pub fn status(&self) -> Status {
        Status::Active
    }

    pub fn latest_height(&self) -> u64 {
        10
    }
}
