// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cosmwasm/lst subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use core::fmt;
use std::num::NonZeroU64;

use cosmwasm_std::{Addr, StdError, StdResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unionlabs_primitives::{Bytes, H256};

/// The maximum allowed unbonding period is 42 days,
/// which is twice the typical staking period of a Cosmos SDK-based chain.
pub const MAX_UNBONDING_PERIOD: u64 = 3_628_800;

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    bincode::Encode,
    bincode::Decode,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(transparent)]
pub struct BatchId(
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint64"))]
    NonZeroU64,
);

impl fmt::Display for BatchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.get(), f)
    }
}

impl BatchId {
    pub const ONE: Self = Self::from_raw(1).unwrap();
    #[cfg(test)]
    pub const TWO: Self = Self::from_raw(2).unwrap();
    #[cfg(test)]
    pub const THREE: Self = Self::from_raw(3).unwrap();

    pub const MAX: Self = Self::from_raw(u64::MAX).unwrap();

    pub const fn new(id: NonZeroU64) -> Self {
        Self(id)
    }

    pub const fn from_raw(raw_id: u64) -> Option<Self> {
        match NonZeroU64::new(raw_id) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }

    pub const fn get(&self) -> NonZeroU64 {
        self.0
    }

    pub const fn increment(&self) -> Self {
        Self(self.get().checked_add(1).expect("holy batches, batman"))
    }

    pub fn to_be_bytes(&self) -> [u8; 8] {
        self.get().get().to_be_bytes()
    }

    pub fn from_be_bytes(raw: [u8; 8]) -> StdResult<Self> {
        BatchId::from_raw(u64::from_be_bytes(raw))
            .ok_or_else(|| StdError::generic_err("invalid key: batch id must be non-zero"))
    }

    pub fn try_from_be_bytes(raw: &Bytes) -> StdResult<Self> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected 8 bytes, found {}: {raw}",
                    raw.len(),
                ))
            })
            .and_then(BatchId::from_be_bytes)
    }
}

impl PendingBatch {
    pub fn new(batch_id: BatchId, submit_time: u64) -> Self {
        Self {
            batch_id,
            total_lst_to_burn: 0,
            unstake_requests_count: 0,
            submit_time,
        }
    }
}

/// Initial state of a batch. Only one batch is pending at a time (see
/// [`crate::state::CurrentPendingBatch`]).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct PendingBatch {
    /// The ID of this batch.
    pub batch_id: BatchId,

    /// Total amount of the LST to be burned in this batch
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub total_lst_to_burn: u128,

    /// The length of the unstake requests list.
    ///
    /// Multiple unbond requests in a batch are aggregated into one unstake request per user.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint64"))]
    pub unstake_requests_count: u64,

    /// The earliest timestamp at which the batch can be submitted.
    ///
    /// This will be `creation_time + batch_period`.
    ///
    /// Note that this is a *minimum* timestamp - empty batches will not be submitted.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint64"))]
    pub submit_time: u64,
}

/// The batch has been submitted, and all unbonding requests have been processed. The unbonded
/// tokens have not yet been sent back to this contract for withdrawing by the unbonded stakers.
///
/// Unbonding requests can only be processed after the unbonding period of the chain this contract
/// is running on.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct SubmittedBatch {
    /// Total amount of the LST to be burned in this batch
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub total_lst_to_burn: u128,

    /// The length of the unstake requests list.
    ///
    /// Multiple unbond requests in a batch are aggregated into one unstake request per user.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint64"))]
    pub unstake_requests_count: u64,
    /// Estimated time when the batch will be received.
    ///
    /// This will be `submission_time + unbonding_period`.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint64"))]
    pub receive_time: u64,

    /// The amount of native tokens that should be received after unstaking.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub expected_native_unstaked: u128,
}

/// The unbonding period has elapsed and the unbonded tokens have been sent back to this contract.
/// The unbonded stakers from this batch are now able to claim their unbonded tokens.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ReceivedBatch {
    /// Total amount of the LST to be burned in this batch
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub total_lst_to_burn: u128,

    /// The length of the unstake requests list.
    ///
    /// Multiple unbond requests in a batch are aggregated into one unstake request per user.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint64"))]
    pub unstake_requests_count: u64,

    /// The amount of native tokens received after unbonding.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub received_native_unstaked: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct BatchExpectedAmount {
    pub batch_id: BatchId,

    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub expected_native_amount: u128,
}

#[derive(Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct Config {
    /// The denom of the native token to liquid stake against.
    ///
    /// for `eU`, this will be `au` (the base denom of `U`).
    pub native_token_denom: String,

    /// Minimum amount of token that can be liquid staked.
    pub minimum_liquid_stake_amount: u128,

    /// Time in seconds between each batch.
    pub batch_period_seconds: u64,

    /// The unbonding period of the chain.
    // TODO: Remove this from the config and read it from the chain directly. This will require
    // GRPC queries to be enabled through cosmwasm first.
    pub unbonding_period_seconds: u64,
}

/// Config related to the fees collected by the contract to
/// operate the liquid staking protocol.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ProtocolFeeConfig {
    /// Fee percentage = fee_rate / 100_000
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    // TODO: BoundedU128<0, FEE_RATE_DENOMINATOR> once i have those types out of unionlabs
    pub fee_rate: u128,

    /// Address where the collected fees are sent.
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Addr"))]
    pub fee_recipient: String,
}

/// State of various balances this contract keeps track of.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct AccountingState {
    /// The total amount of native tokens that have been bonded.
    ///
    /// This plus the pending rewards is the "total assets".
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub total_bonded_native_tokens: u128,

    /// The total issuance of the LST, also known as the "total shares".
    ///
    /// Note that this is *not* the same as the total supply of the LST contract, but rather the
    /// total *cross-chain* supply of the LST. For example, when the LST is bridged, it will be
    /// burned on the source chain and minted on the destination chain.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub total_issued_lst: u128,

    // REVIEW: Unused? If this is only used for off-chain actions/ accounting then this is probably
    // better off in a separate storage
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub total_reward_amount: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
pub struct PendingOwner {
    pub address: String,
    pub owner_transfer_min_time_seconds: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UnstakeRequestKey {
    pub batch_id: BatchId,

    /// The hash of the staker of the associated [`UnstakeRequest`].
    ///
    /// This is `sha256(bytes(staker))`.
    pub staker_hash: H256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, bincode::Encode, bincode::Decode)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UnstakeRequest {
    pub batch_id: BatchId,
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Addr"))]
    pub staker: String,
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub amount: u128,
}

pub fn staker_hash(staker: &Addr) -> H256 {
    Sha256::digest(staker.as_str()).into()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn staker_hash_smoke_test() {
        assert_eq!(
            staker_hash(&Addr::unchecked("zkgm")),
            <H256>::new(hex!(
                "5c182b7072dacb88108401a7338542b8bba30d0d1a7c70963fdabfdfe2e4d4dd"
            ))
        );
    }
}
