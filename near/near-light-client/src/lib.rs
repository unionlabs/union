mod contract;
mod nibble_slice;
mod state_proof;
pub mod types;

pub use contract::*;
use near_primitives_core::{hash::CryptoHash, types::AccountId};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use unionlabs::near::types::{BlockHeaderInnerLiteView, ValidatorStakeView};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientState {
    latest_height: u64,
    ibc_account_id: AccountId,
    initial_block_producers: Option<Vec<ValidatorStakeView>>,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ConsensusState {
    pub state: BlockHeaderInnerLiteView,
    pub chunk_prev_state_root: CryptoHash,
}
