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

use cosmwasm_std::{
    Addr, Uint128, coins,
    testing::{message_info, mock_env},
};
use cw_utils::PaymentError;

use crate::{
    error::ContractError,
    msg::ExecuteMsg,
    tests::test_helper::{ADMIN, NATIVE_TOKEN, UNION1, ensure_execute_error, setup},
    types::BatchId,
};

mod bond_tests;
mod circuit_breaker_tests;
mod helper_tests;
mod instantiate_tests;
mod ownership_tests;
mod query_tests;
mod rebase_tests;
mod reward_tests;
mod submit_batch_tests;
mod test_helper;
mod unbond_tests;
mod withdraw_tests;

#[test]
fn nonpayable() {
    let deps = setup();
    let env = mock_env();
    let info = message_info(&Addr::unchecked(ADMIN), &coins(1, NATIVE_TOKEN));

    let nonpayable_msgs = [
        ExecuteMsg::Unbond {
            amount: Uint128::new(100),
        },
        ExecuteMsg::Withdraw {
            withdraw_to_address: Addr::unchecked(UNION1),
            batch_id: BatchId::ONE,
        },
        ExecuteMsg::SubmitBatch {},
        ExecuteMsg::TransferOwnership {
            new_owner: UNION1.to_owned(),
        },
        ExecuteMsg::AcceptOwnership {},
        ExecuteMsg::RevokeOwnershipTransfer {},
        ExecuteMsg::UpdateConfig {
            protocol_fee_config: None,
            monitors: None,
            batch_period_seconds: None,
            unbonding_period_seconds: None,
        },
        ExecuteMsg::Rebase {},
        ExecuteMsg::CircuitBreaker {},
        ExecuteMsg::ResumeContract {
            total_bonded_native_tokens: Uint128::new(100),
            total_issued_lst: Uint128::new(100),
            total_reward_amount: Uint128::new(100),
        },
        ExecuteMsg::SlashBatches {
            new_amounts: vec![],
        },
    ];

    for msg in nonpayable_msgs {
        ensure_execute_error(
            deps.as_ref(),
            &env,
            &info,
            msg,
            ContractError::Payment(PaymentError::NonPayable {}),
        );
    }
}
