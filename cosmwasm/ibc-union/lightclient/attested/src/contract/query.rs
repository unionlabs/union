use std::{collections::BTreeSet, num::NonZero};

use cosmwasm_std::{Deps, Order};
use depolama::{Bytes, StorageExt};
use ibc_union_light_client::spec::Timestamp;
use serde::{Deserialize, Serialize};
use unionlabs::primitives::H256;

use crate::{
    errors::Error,
    state::{Attestations, Attestors, HeightTimestamps, Quorum},
    types::{AttestationKey, AttestationValue},
};

pub fn quorum(deps: Deps, chain_id: String) -> Result<NonZero<u8>, Error> {
    deps.storage
        .maybe_read::<Quorum>(&chain_id)?
        .ok_or_else(|| Error::QuorumNotSet { chain_id })
}

pub fn attestors(deps: Deps, chain_id: String) -> Result<BTreeSet<H256>, Error> {
    deps.storage
        .iter_range::<Attestors>(
            Order::Ascending,
            (chain_id.clone(), H256::MIN)..=(chain_id, H256::MAX),
        )
        .map(|r| r.map(|((_, attestor), ())| attestor))
        .collect::<Result<_, _>>()
        .map_err(Into::into)
}

pub fn attested_value(
    deps: Deps,
    chain_id: String,
    height: u64,
    key: Bytes,
) -> Result<Option<AttestationValue>, Error> {
    deps.storage
        .maybe_read::<Attestations>(&AttestationKey {
            chain_id,
            height,
            key,
        })
        .map_err(Into::into)
}

pub fn timestamp_at_height(
    deps: Deps,
    chain_id: String,
    height: u64,
) -> Result<Option<Timestamp>, Error> {
    deps.storage
        .maybe_read::<HeightTimestamps>(&(chain_id, height))
        .map_err(Into::into)
}

pub fn latest_height(deps: Deps, chain_id: String) -> Result<Option<LatestHeight>, Error> {
    deps.storage
        .iter_range::<HeightTimestamps>(
            Order::Descending,
            (chain_id.clone(), 0)..=(chain_id.clone(), u64::MAX),
        )
        .next()
        .map(|r| r.map(|((_, height), timestamp)| LatestHeight { height, timestamp }))
        .transpose()
        .map_err(Into::into)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct LatestHeight {
    pub height: u64,
    pub timestamp: Timestamp,
}
