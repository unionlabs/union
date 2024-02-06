use std::fmt::Debug;

use bitvec::{prelude::Msb0, view::AsBits};
use serde::{Deserialize, Serialize};

use crate::{tendermint::types::simple_validator::SimpleValidator, Proto, TypeUrl};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct ValidatorSetCommit {
    pub validators: Vec<SimpleValidator>,
    // REVIEW: Is this arbitrary bytes or strongly typed? (i.e. H512)
    #[serde(with = "::serde_utils::hex_string_list")]
    pub signatures: Vec<Vec<u8>>,
    pub bitmap: Vec<u8>,
}

impl Debug for ValidatorSetCommit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidatorSetCommit")
            .field("validators", &self.validators)
            .field(
                "signatures",
                &self
                    .signatures
                    .iter()
                    .map(serde_utils::to_hex)
                    .collect::<Vec<_>>(),
            )
            .field(
                "bitmap",
                &self
                    .bitmap
                    .as_bits::<Msb0>()
                    .iter()
                    .by_refs()
                    // REVIEW: Is string literal or char more efficient?
                    .map(|bit| if *bit { '1' } else { '0' })
                    .collect::<String>(),
            )
            .finish()
    }
}

impl Proto for ValidatorSetCommit {
    type Proto = protos::union::galois::api::v2::ValidatorSetCommit;
}

impl TypeUrl for protos::union::galois::api::v2::ValidatorSetCommit {
    const TYPE_URL: &'static str = "/union.galois.api.v2.ValidatorSetCommit";
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
