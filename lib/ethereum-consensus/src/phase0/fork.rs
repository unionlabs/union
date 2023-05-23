use crate::primitives::{Epoch, Root, Version};
use ssz_rs::prelude::*;

#[derive(Default, Debug, SimpleSerialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fork {
    #[serde(with = "crate::serde::as_hex")]
    pub previous_version: Version,
    #[serde(with = "crate::serde::as_hex")]
    pub current_version: Version,
    #[serde(with = "crate::serde::as_string")]
    pub epoch: Epoch,
}

#[derive(Default, Debug, SimpleSerialize, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForkData {
    #[serde(with = "crate::serde::as_hex")]
    pub current_version: Version,
    pub genesis_validators_root: Root,
}
