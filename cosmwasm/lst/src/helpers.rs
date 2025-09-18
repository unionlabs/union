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

use cosmwasm_std::{Addr, DecCoin, Decimal256, Deps, StdResult, Uint128};

use crate::types::AccountingState;

pub fn assets_to_shares(total_assets: u128, total_shares: u128, assets: u128) -> u128 {
    // possible truncation issues when quantities are small
    // initial very large total_bonded_native_tokens would cause round to 0 and block minting
    // mint at a 1:1 ratio if there is no bonded native token currently
    if total_assets == 0 {
        assets
    } else {
        Uint128::new(assets)
            .multiply_ratio(total_shares, total_assets)
            .u128()
    }
}

pub fn shares_to_assets(total_assets: u128, total_shares: u128, shares: u128) -> u128 {
    // TODO: Check if this branch can even be hit
    if shares == 0 {
        0
    } else {
        Uint128::new(shares)
            .multiply_ratio(total_assets, total_shares)
            .into()
    }
}

pub fn total_assets(
    deps: Deps,
    state: &AccountingState,
    native_token_denom: &str,
    staker_address: &Addr,
) -> StdResult<u128> {
    Ok(state
        .total_bonded_native_tokens
        .checked_add(total_pending_rewards(
            deps,
            native_token_denom,
            staker_address,
        )?)
        .expect("overflow"))
}

fn total_pending_rewards(
    deps: Deps,
    native_token_denom: &str,
    staker_address: &Addr,
) -> StdResult<u128> {
    Ok(deps
        .querier
        .query_delegation_total_rewards(staker_address)?
        .total
        .into_iter()
        .filter_map(|DecCoin { denom, amount }| (denom == native_token_denom).then_some(amount))
        .try_fold(Decimal256::zero(), |a, b| a.checked_add(b))
        .map(|total| Uint128::try_from(total.to_uint_floor()))
        .expect("too many tokens")
        .expect("too many tokens")
        .u128())
}

// NOTE: Unused for now, once the unbonding period is queried from the chain directly instead of
// being in the config this will need to be used again. /// Query the unbonding period from the
// chain, and verify that the batch period is smaller than the queried unbonding period.
// pub fn query_and_validate_unbonding_period(
//     deps: Deps,
//     batch_period: u64,
// ) -> Result<u64, ContractError> {
//     #[derive(Clone, PartialEq, Message)]
//     pub struct QueryParamsRequest {}

//     #[derive(Clone, PartialEq, Message)]
//     pub struct QueryParamsResponse {
//         #[prost(message, optional, tag = "1")]
//         pub params: Option<Params>,
//     }

//     #[derive(Clone, PartialEq, Message)]
//     pub struct Params {
//         #[prost(message, optional, tag = "1")]
//         pub unbonding_time: Option<Duration>,
//     }
//     #[derive(Clone, PartialEq, Message)]
//     pub struct Duration {
//         #[prost(int64, tag = "1")]
//         pub seconds: i64,
//     }

//     let res = deps.querier.query_grpc(
//         "/cosmos.staking.v1beta1.Query/Params".to_owned(),
//         QueryParamsRequest {}.encode_to_vec().into(),
//     )?;

//     let unbonding_period = QueryParamsResponse::decode(&*res)
//         .map_err(|e| StdError::generic_err(format!("error decoding query params response: {e}")))
//         .and_then(|res| {
//             res.params.ok_or_else(|| {
//                 StdError::generic_err("invalid query params response, missing params")
//             })
//         })
//         .and_then(|res| {
//             res.unbonding_time.ok_or_else(|| {
//                 StdError::generic_err(
//                     "invalid query params response, missing params.unbonding_time",
//                 )
//             })
//         })
//         .and_then(|res| {
//             res.seconds.try_into().map_err(|_| {
//                 StdError::generic_err(
//                     "invalid query params response, params.unbonding_time.seconds is negative",
//                 )
//             })
//         })?;

//     // Ensure the batch period is lower then unbonding period.
//     if batch_period > unbonding_period {
//         Err(ContractError::BatchPeriodLargerThanUnbondingPeriod {
//             batch_period,
//             unbonding_period,
//         })
//     } else {
//         Ok(unbonding_period)
//     }
// }
