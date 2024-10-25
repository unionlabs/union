use macros::model;
use ssz::Ssz;

use crate::{bls::BlsSignature, ethereum::beacon::bls_to_execution_change::BlsToExecutionChange};

#[model]
#[derive(Ssz)]
pub struct SignedBlsToExecutionChange {
    message: BlsToExecutionChange,
    signature: BlsSignature,
}
