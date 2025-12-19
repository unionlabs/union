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

use cosmwasm_event::Event;
use cosmwasm_std::Addr;

use crate::types::BatchId;

#[derive(Event)]
#[event("init")]
pub struct Init {
    pub admin: Addr,
    pub native_token_denom: String,
    pub minimum_liquid_stake_amount: u128,
    pub protocol_fee_rate: u128,
    pub protocol_fee_recipient: String,
    pub current_unbonding_period: u64,
    pub staker_address: Addr,
    pub lst_address: Addr,
}

#[derive(Event)]
#[event("bond")]
pub struct Bond {
    pub mint_to_address: Addr,
    pub sender: Addr,
    pub in_amount: u128,
    pub mint_amount: u128,
}

#[derive(Event)]
#[event("unbond")]
pub struct Unbond {
    pub staker: Addr,
    pub batch: BatchId,
    pub amount: u128,
    pub is_new_request: bool,
}

#[derive(Event)]
#[event("submit_batch")]
pub struct SubmitBatch {
    pub batch_id: BatchId,
    pub batch_total: u128,
    pub expected_unstaked: u128,
    pub current_unbonding_period: u64,
}

#[derive(Event)]
#[event("receive_rewards")]
pub struct ReceiveRewards {
    pub amount: u128,
    pub amount_after_protocol_fee: u128,
    pub protocol_fee: u128,
}

#[derive(Event)]
#[event("rebase")]
pub struct Rebase {
    pub caller: Addr,
}

#[derive(Event)]
#[event("receive_unstaked_tokens")]
pub struct ReceiveUnstakedTokens {
    pub batch: BatchId,
    pub amount: u128,
}

#[derive(Event)]
#[event("withdraw")]
pub struct Withdraw<'a> {
    pub staker: Addr,
    pub batch_id: BatchId,
    pub withdraw_to_address: &'a Addr,
    pub amount: u128,
}

#[derive(Event)]
#[event("transfer_ownership")]
pub struct TransferOwnership {
    pub new_owner: String,
    pub previous_owner: Addr,
}

#[derive(Event)]
#[event("revoke_ownership_transfer")]
pub struct RevokeOwnershipTransfer {}

#[derive(Event)]
#[event("accept_ownership")]
pub struct AcceptOwnership {
    pub new_owner: Addr,
}

#[derive(Event)]
#[event("circuit_breaker")]
pub struct CircuitBreaker {
    pub breaker: Addr,
}

#[derive(Event)]
#[event("resume_contract")]
pub struct ResumeContract {
    pub total_bonded_native_tokens: u128,
    pub total_issued_lst: u128,
    pub total_reward_amount: u128,
}

#[derive(Event)]
#[event("slash_batch")]
pub struct SlashBatch {
    pub batch_id: BatchId,
    pub amount: u128,
}
