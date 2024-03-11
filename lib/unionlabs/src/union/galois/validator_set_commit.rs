use bitvec::prelude::Msb0;
use custom_debug_derive::Debug;
use macros::proto;
use serde::{Deserialize, Serialize};

use crate::tendermint::types::simple_validator::SimpleValidator;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::union::galois::api::v2::ValidatorSetCommit, from)]
pub struct ValidatorSetCommit {
    pub validators: Vec<SimpleValidator>,
    // REVIEW: Is this arbitrary bytes or strongly typed? (i.e. H512)
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(with = "::serde_utils::fmt::hex_list")]
    pub signatures: Vec<Vec<u8>>,
    #[debug(with = "::serde_utils::fmt::bits::<Msb0>")]
    pub bitmap: Vec<u8>,
}

impl From<ValidatorSetCommit> for protos::union::galois::api::v2::ValidatorSetCommit {
    fn from(value: ValidatorSetCommit) -> Self {
        Self {
            validators: value.validators.into_iter().map(Into::into).collect(),
            signatures: value.signatures,
            bitmap: value.bitmap,
        }
    }
}
