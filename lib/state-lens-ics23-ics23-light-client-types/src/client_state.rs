use unionlabs::{primitives::H256, tuple::AsTuple};

pub type ClientState = state_lens_light_client_types::ClientState<Extra>;

#[derive(Debug, Clone, PartialEq, AsTuple)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Extra {
    /// ibc contract that is running on l2
    pub contract_address: H256,
}
