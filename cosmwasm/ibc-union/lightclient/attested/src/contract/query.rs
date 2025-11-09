use std::{collections::BTreeSet, num::NonZero};

use cosmwasm_std::{Deps, Order};
use depolama::{Bytes, StorageExt};
use ibc_union_light_client::spec::Timestamp;
use unionlabs::primitives::H256;

use crate::{
    errors::Error,
    state::{Attestations, Attestors, HeightTimestamps, Quorum},
    types::{AttestationKey, AttestationValue},
};

pub fn quorum(deps: Deps) -> Result<NonZero<u8>, Error> {
    deps.storage
        .maybe_read_item::<Quorum>()?
        .ok_or(Error::QuorumNotSet)
}

pub fn attestors(deps: Deps) -> Result<BTreeSet<H256>, Error> {
    deps.storage
        .iter::<Attestors>(Order::Ascending)
        .map(|r| r.map(|(attestor, ())| attestor))
        .collect::<Result<_, _>>()
        .map_err(Into::into)
}

pub fn attested_value(
    deps: Deps,
    height: u64,
    key: Bytes,
) -> Result<Option<AttestationValue>, Error> {
    deps.storage
        .maybe_read::<Attestations>(&AttestationKey { height, key })
        .map_err(Into::into)
}

pub fn timestamp_at_height(deps: Deps, height: u64) -> Result<Option<Timestamp>, Error> {
    deps.storage
        .maybe_read::<HeightTimestamps>(&height)
        .map_err(Into::into)
}
