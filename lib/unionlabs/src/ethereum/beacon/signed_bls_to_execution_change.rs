use macros::model;

use crate::{ethereum::beacon::bls_to_execution_change::BlsToExecutionChange, hash::H768};

#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct SignedBlsToExecutionChange {
    message: BlsToExecutionChange,
    signature: H768,
}
