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

use cosmwasm_std::{Addr, StdError};
use cw_utils::PaymentError;
use frissitheto::UpgradeError;

use crate::types::{BatchId, Staker, MAX_FEE_RATE};

pub type ContractResult<T> = core::result::Result<T, ContractError>;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("migration error")]
    Migrate(#[from] UpgradeError),

    #[error(transparent)]
    Payment(#[from] PaymentError),

    #[error("unauthorized: {sender}")]
    Unauthorized { sender: Addr },

    #[error("no pending owner to claim")]
    NoPendingOwner,

    #[error("the caller is not the pending owner")]
    CallerIsNotPendingOwner,

    #[error(
        "ownership transfer not ready, claimable at {claimable_at_seconds} (now={now_seconds})"
    )]
    OwnershipTransferNotReady {
        now_seconds: u64,
        claimable_at_seconds: u64,
    },

    #[error(
        "attempted to bond less than minimum stake amount \
        (min={minimum_stake_amount}, sent={sent_amount})"
    )]
    MinimumLiquidStakeAmount {
        minimum_stake_amount: u128,
        sent_amount: u128,
    },

    #[error("computed mint amount is zero")]
    ComputedMintAmountIsZero,

    #[error("batch is not ready to be submitted/received (now={now}, ready_at={ready_at})")]
    BatchNotReady { now: u64, ready_at: u64 },

    #[error("batch {batch_id} has already been submitted")]
    BatchAlreadySubmitted { batch_id: BatchId },

    #[error("no liquid unstake requests in the current batch (batch_id={batch_id})")]
    NoUnstakeRequestsInCurrentBatch { batch_id: BatchId },

    #[error("batch {batch_id} not found")]
    BatchNotFound { batch_id: BatchId },

    #[error("batch {batch_id} is still pending")]
    BatchStillPending { batch_id: BatchId },

    #[error("batch {batch_id} has already been received")]
    BatchAlreadyReceived { batch_id: BatchId },

    #[error(
        "received wrong batch amount, batch_id {batch_id} \
        expected {expected}, got {received}"
    )]
    ReceivedWrongBatchAmount {
        batch_id: BatchId,
        expected: u128,
        received: u128,
    },

    #[error("batch {batch_id} is not yet received")]
    BatchNotYetReceived { batch_id: BatchId },

    #[error("staker {staker} not found in batch {batch_id} (hash={})", staker.hash())]
    NoRequestInBatch { batch_id: BatchId, staker: Staker },

    #[error(
        "unbond slippage exceeded (total_issued_lst={total_issued_lst}, \
        amount_to_unstake={amount_to_unstake})"
    )]
    InvalidUnstakeAmount {
        total_issued_lst: u128,
        amount_to_unstake: u128,
    },

    #[error("contract was intentionally stopped")]
    Stopped,

    #[error("contract is not stopped")]
    NotStopped,

    #[error(
        "received rewards ({received_rewards}) are \
        less than the protocol fee ({protocol_fee})"
    )]
    RewardsReceivedLessThanProtocolFee {
        received_rewards: u128,
        protocol_fee: u128,
    },

    #[error("computed fees are zero for the received rewards ({received_rewards})")]
    ComputedFeesAreZero { received_rewards: u128 },

    #[error("No liquid stake to distribute rewards to")]
    NoLiquidStake,

    #[error("slippage not met (min={min_mint_amount}, actual={actual})")]
    SlippageNotMet { min_mint_amount: u128, actual: u128 },

    #[error("protocol fee rate can't be higher then {MAX_FEE_RATE}")]
    InvalidProtocolFeeRate,
    #[error(
        "the batch period ({batch_period_seconds}) is larger than the \
        queried unbonding period ({unbonding_period_seconds})"
    )]
    BatchPeriodLargerThanUnbondingPeriod {
        batch_period_seconds: u64,
        unbonding_period_seconds: u64,
    },

    #[error(
        "attempted to unbond more native tokens {unbond_amount} than \
        total bonded native tokens {total_bonded_native_tokens}"
    )]
    AttemptedToUnbondMoreThanBonded {
        unbond_amount: u128,
        total_bonded_native_tokens: u128,
    },

    #[error("batch {batch_id} not yet submitted")]
    BatchNotYetSubmitted { batch_id: BatchId },
}
