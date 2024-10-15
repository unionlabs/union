use bitvec::prelude::Msb0;
use macros::model;

use crate::tendermint::types::simple_validator::SimpleValidator;

#[model(proto(raw(protos::union::galois::api::v3::ValidatorSetCommit), from))]
pub struct ValidatorSetCommit {
    pub validators: Vec<SimpleValidator>,
    // REVIEW: Is this arbitrary bytes or strongly typed? (i.e. H512)
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string_list"))]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub signatures: Vec<Vec<u8>>,
    #[debug(wrap = ::serde_utils::fmt::DebugBits::<_, _, Msb0>::new)]
    pub bitmap: Vec<u8>,
}

#[cfg(feature = "proto")]
impl From<ValidatorSetCommit> for protos::union::galois::api::v3::ValidatorSetCommit {
    fn from(value: ValidatorSetCommit) -> Self {
        Self {
            validators: value.validators.into_iter().map(Into::into).collect(),
            signatures: value.signatures,
            bitmap: value.bitmap,
        }
    }
}
