use std::{collections::BTreeSet, num::NonZero};

use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::{H256, H512};

use crate::types::Attestation;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InitMsg {
    pub ibc_host: Addr,
    pub attestors: BTreeSet<H256>,
    pub quorum: NonZero<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    Attest {
        attestation: Attestation,
        attestor: H256,
        signature: H512,
    },
}
