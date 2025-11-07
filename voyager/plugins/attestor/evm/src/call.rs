use macros::model;
use unionlabs::primitives::H256;

#[model]
pub enum ModuleCall {
    VerifyState(VerifyState),
    SubmitAttestation(SubmitAttestation),
}

#[model]
pub struct SubmitAttestation {
    pub event: ibc_union_spec::event::FullEvent,
    pub tx_hash: H256,
    pub height: u64,
}
