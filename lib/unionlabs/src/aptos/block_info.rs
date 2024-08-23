// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::array::TryFromSliceError;

use serde::{Deserialize, Serialize};

use super::{
    epoch_state::{EpochState, TryFromEpochStateError},
    hash_value::HashValue,
};

/// The round of a block is a consensus-internal counter, which starts with 0 and increases
/// monotonically.
pub type Round = u64;

pub type Version = u64;

/// A continuously increasing sequence number for committed blocks.
pub type BlockHeight = u64;

// Constants for the initial genesis block.
pub const GENESIS_EPOCH: u64 = 0;
pub const GENESIS_ROUND: Round = 0;
pub const GENESIS_VERSION: Version = 0;
pub const GENESIS_TIMESTAMP_USECS: u64 = 0;

/// This structure contains all the information needed for tracking a block
/// without having access to the block or its execution output state. It
/// assumes that the block is the last block executed within the ledger.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BlockInfo {
    /// The epoch to which the block belongs.
    pub epoch: u64,
    /// The consensus protocol is executed in rounds, which monotonically increase per epoch.
    pub round: Round,
    /// The identifier (hash) of the block.
    pub id: HashValue,
    /// The accumulator root hash after executing this block.
    pub executed_state_id: HashValue,
    /// The version of the latest transaction after executing this block.
    pub version: Version,
    /// The timestamp this block was proposed by a proposer.
    pub timestamp_usecs: u64,
    /// An optional field containing the next epoch info
    pub next_epoch_state: Option<EpochState>,
}

impl From<BlockInfo> for protos::union::ibc::lightclients::movement::v1::BlockInfo {
    fn from(value: BlockInfo) -> Self {
        Self {
            epoch: value.epoch,
            round: value.round,
            id: value.id.0.to_vec(),
            executed_state_id: value.executed_state_id.0.to_vec(),
            version: value.version,
            timestamp_usecs: value.timestamp_usecs,
            next_epoch_state: value.next_epoch_state.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TryFromBlockInfoError {
    #[error("invalid id")]
    Id(#[source] TryFromSliceError),
    #[error("invalid executed state id")]
    ExecutedStateId(#[source] TryFromSliceError),
    #[error("invalid next epoch state: {0}")]
    NextEpochState(#[from] TryFromEpochStateError),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::BlockInfo> for BlockInfo {
    type Error = TryFromBlockInfoError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::BlockInfo,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: value.epoch,
            round: value.round,
            id: HashValue::new(
                value
                    .id
                    .as_slice()
                    .try_into()
                    .map_err(TryFromBlockInfoError::Id)?,
            ),
            executed_state_id: HashValue::new(
                value
                    .executed_state_id
                    .as_slice()
                    .try_into()
                    .map_err(TryFromBlockInfoError::ExecutedStateId)?,
            ),
            version: value.version,
            timestamp_usecs: value.timestamp_usecs,
            next_epoch_state: value.next_epoch_state.map(TryInto::try_into).transpose()?,
        })
    }
}
