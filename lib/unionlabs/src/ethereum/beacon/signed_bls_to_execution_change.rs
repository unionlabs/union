use macros::model;
use ssz::Ssz;

use crate::{ethereum::beacon::bls_to_execution_change::BlsToExecutionChange, hash::H768};

#[model]
#[derive(Ssz)]
pub struct SignedBlsToExecutionChange {
    message: BlsToExecutionChange,
    signature: H768,
}
