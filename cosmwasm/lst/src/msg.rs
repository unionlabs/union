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

use cosmwasm_std::{Addr, Decimal, Uint128};
use serde::{Deserialize, Serialize};

use crate::types::{
    BatchExpectedAmount, BatchId, PendingBatch, ProtocolFeeConfig, ReceivedBatch, SubmittedBatch,
    UnstakeRequestKey,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InitMsg {
    pub native_token_denom: String,

    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint128"))]
    pub minimum_liquid_stake_amount: u128,

    /// Address of the account that delegates the tokens
    /// toward the validators.
    pub staker_address: Addr,

    /// Protocol fee configuration.
    pub protocol_fee_config: ProtocolFeeConfig,

    /// Address of the LST contract.
    pub lst_address: Addr,

    /// Frequency (in seconds) at which the unbonding queue is executed.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint64"))]
    pub batch_period_seconds: u64,

    /// The unbonding period of the chain.
    #[serde(with = "::serde_utils::string")]
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Uint64"))]
    pub unbonding_period_seconds: u64,

    /// Set of addresses allowed to trigger a circuit break.
    pub monitors: Vec<Addr>,
    pub admin: Addr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[allow(clippy::large_enum_variant)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Initiates the bonding process for a user.
    Bond {
        /// The address to mint the LST to.
        mint_to_address: Addr,

        /// Minimum expected amount of LST tokens to be received
        /// for the operation to be considered valid.
        min_mint_amount: Uint128,
    },

    /// Initiates the unbonding process for a user.
    Unbond {
        /// The amount to unstake.
        amount: Uint128,
    },

    /// Withdraws unstaked tokens.
    Withdraw {
        /// The address to withdraw the funds to.
        withdraw_to_address: Addr,
        /// ID of the batch from which to withdraw.
        batch_id: BatchId,
    },

    /// Processes the pending batch.
    SubmitBatch {},

    TransferOwnership {
        /// Address of the new owner on the protocol chain.
        #[cfg_attr(feature = "schemars", schemars(with = "Addr"))]
        new_owner: String,
    },

    /// Accepts ownership transfer; callable by the new owner.
    AcceptOwnership {},

    /// Revokes ownership transfer; callable by the current owner.
    RevokeOwnershipTransfer {},

    /// Updates contract configuration; callable by the owner.
    UpdateConfig {
        /// Updated protocol fee configuration.
        protocol_fee_config: Option<ProtocolFeeConfig>,

        /// Updated list of circuit breaker monitors.
        monitors: Option<Vec<Addr>>,

        /// Updated unbonding batch execution frequency (in seconds).
        batch_period_seconds: Option<u64>,

        /// Updated unbonding period for this chain.
        unbonding_period_seconds: Option<u64>,
    },
    /// Receives rewards from the native chain.
    ReceiveRewards {},

    /// Rebase the LST by claiming all current pending rewards and restaking them.
    Rebase {},

    /// Receives unstaked tokens from the native chain.
    ReceiveUnstakedTokens {
        /// ID of the batch that originated the unstake request.
        batch_id: BatchId,
    },

    /// Stops the contract due to irregularities; callable by monitors and admin.
    CircuitBreaker {},

    /// Resumes the contract; callable by the admin.
    ResumeContract {
        /// Updated total native tokens delegated (used post-slashing).
        total_bonded_native_tokens: Uint128,

        /// Updated total issued liquid staked tokens.
        total_issued_lst: Uint128,

        /// Updated total protocol rewards.
        total_reward_amount: Uint128,
    },
    SlashBatches {
        new_amounts: Vec<BatchExpectedAmount>,
    },

    /// Call Staker to received unstaked tokens for specific batch
    ReceiveBatch { batch_id: BatchId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schemars",
    derive(schemars::JsonSchema, cosmwasm_schema::QueryResponses)
)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum QueryMsg {
    /// Queries the contract configuration.
    /// Returns the current `native_chain_config`, `protocol_chain_config`,
    /// `protocol_fee_config`, `liquid_stake_token_denom`, and other settings.
    #[cfg_attr(feature = "schemars", returns(ConfigResponse))]
    Config {},

    /// Queries the current accounting state of the contract.
    /// Returns totals such as delegated native tokens, LST supply, and rewards.
    #[cfg_attr(feature = "schemars", returns(AccountingStateResponse))]
    AccountingState {},

    /// Queries the information of a specific batch by its ID.
    #[cfg_attr(feature = "schemars", returns(Batch))]
    Batch {
        /// ID of the batch to query.
        batch_id: BatchId,
    },

    /// Queries a paginated list of all submitted batches.
    #[cfg_attr(feature = "schemars", returns(BatchesResponse<IdentifiedBatch<SubmittedBatch>>))]
    SubmittedBatches {
        /// If provided, starts listing batches after this batch ID.
        start_after: Option<BatchId>,

        /// Maximum number of batches to return.
        limit: Option<usize>,
    },

    /// Queries a paginated list of all received batches.
    #[cfg_attr(feature = "schemars", returns(BatchesResponse<IdentifiedBatch<ReceivedBatch>>))]
    ReceivedBatches {
        /// If provided, starts listing batches after this batch ID.
        start_after: Option<BatchId>,

        /// Maximum number of batches to return.
        limit: Option<usize>,
    },

    /// Queries the batches with the provided list of IDs.
    #[cfg_attr(feature = "schemars", returns(BatchesResponse<IdentifiedBatch<Batch>>))]
    BatchesByIds {
        /// List of batch IDs to fetch.
        batch_ids: Vec<BatchId>,
    },

    /// Queries the current batch that is pending processing (if any).
    #[cfg_attr(feature = "schemars", returns(Batch))]
    PendingBatch {},

    /// Queries the unstake requests made by a specific staker.
    #[cfg_attr(feature = "schemars", returns(Vec<crate::types::UnstakeRequest>))]
    UnstakeRequestsByStaker {
        /// Address of the user whose unstake requests are to be queried.
        staker: Addr,
    },

    /// Queries all unstake requests in the contract.
    #[cfg_attr(feature = "schemars", returns(Vec<crate::types::UnstakeRequest>))]
    AllUnstakeRequests {
        /// If provided, starts listing unstake requests after this key.
        start_after: Option<UnstakeRequestKey>,

        /// Maximum number of unstake requests to return.
        limit: Option<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct ConfigResponse {
    pub native_token_denom: String,
    pub minimum_liquid_stake_amount: Uint128,
    pub protocol_fee_config: ProtocolFeeConfig,
    pub monitors: Vec<Addr>,
    pub lst_address: Addr,
    pub staker_address: Addr,
    pub batch_period_seconds: u64,
    pub unbonding_period_seconds: u64,
    pub stopped: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct AccountingStateResponse {
    pub total_assets: Uint128,
    pub total_shares: Uint128,
    pub total_reward_amount: Uint128,
    pub redemption_rate: Decimal,
    pub purchase_rate: Decimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct OwnershipResponse {
    pub pending_owner: Addr,
    pub total_reward_amount: Uint128,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct BatchesResponse<B> {
    pub batches: Vec<IdentifiedBatch<B>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct IdentifiedBatch<B> {
    pub batch_id: BatchId,
    pub batch: B,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "status")]
pub enum Batch {
    Pending(PendingBatch),
    Submitted(SubmittedBatch),
    Received(ReceivedBatch),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "status")]
pub enum StakerExecuteMsg {
    /// Stake the tokens provided along with this call.
    ///
    /// This must only be callable by the LST hub itself.
    Stake {},
    /// Unstake `amount` of tokens.
    ///
    /// This must only be callable by the LST hub itself.
    Unstake { amount: Uint128 },
    /// Rebase the current rewards by restaking them through the LST hub.
    ///
    /// This must only be callable by the LST hub itself.
    Rebase {},
    /// Receive unstaked tokens to mark batch as received
    ///
    /// This must only be callable by the LST hub itself.
    ReceiveUnstakedTokens { batch_id: BatchId },
}
