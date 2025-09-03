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

use cosmwasm_std::{Addr, Deps, Order, StdResult};
use depolama::{StorageExt, Store};
use itertools::Itertools;
use unionlabs_primitives::H256;

use crate::{
    error::ContractError,
    helpers::get_rates,
    msg::{AccountingStateResponse, Batch, BatchesResponse, ConfigResponse, IdentifiedBatch},
    state::{
        AccountingStateStore, ConfigStore, CurrentPendingBatch, LstAddress, Monitors,
        ProtocolFeeConfigStore, ReceivedBatches, Stopped, SubmittedBatches,
        UnstakeRequestsByStakerHash,
    },
    types::{BatchId, Config, PendingBatch, UnstakeRequest, UnstakeRequestKey},
};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let Config {
        native_token_denom,
        minimum_liquid_stake_amount,
        batch_period_seconds,
        unbonding_period_seconds,
    } = deps.storage.read_item::<ConfigStore>()?;

    Ok(ConfigResponse {
        native_token_denom,
        minimum_liquid_stake_amount: minimum_liquid_stake_amount.into(),
        protocol_fee_config: deps.storage.read_item::<ProtocolFeeConfigStore>()?,
        lst_address: deps.storage.read_item::<LstAddress>()?,
        monitors: deps
            .storage
            .read_item::<Monitors>()?
            .into_iter()
            .map(Addr::unchecked)
            .collect(),
        batch_period_seconds,
        unbonding_period_seconds,
        stopped: deps.storage.read_item::<Stopped>()?,
    })
}

pub fn query_state(deps: Deps) -> StdResult<AccountingStateResponse> {
    let accounting_state = deps.storage.read_item::<AccountingStateStore>()?;

    let (redemption_rate, purchase_rate) = get_rates(&accounting_state);
    let res = AccountingStateResponse {
        total_bonded_native_tokens: accounting_state.total_bonded_native_tokens.into(),
        total_issued_lst: accounting_state.total_issued_lst.into(),
        total_reward_amount: accounting_state.total_reward_amount.into(),
        redemption_rate,
        purchase_rate,
    };
    Ok(res)
}

pub fn query_batch(deps: Deps, batch_id: BatchId) -> Result<Option<Batch>, ContractError> {
    if let Some(batch) = deps.storage.maybe_read::<ReceivedBatches>(&batch_id)? {
        Ok(Some(Batch::Received(batch)))
    } else if let Some(batch) = deps.storage.maybe_read::<SubmittedBatches>(&batch_id)? {
        Ok(Some(Batch::Submitted(batch)))
    } else if let Some(batch) = deps.storage.maybe_read_item::<CurrentPendingBatch>()? {
        if batch.batch_id == batch_id {
            Ok(Some(Batch::Pending(batch)))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub fn query_batches<S: Store<Key = BatchId>>(
    deps: Deps,
    start_after: Option<BatchId>,
    limit: Option<usize>,
) -> StdResult<BatchesResponse<S::Value>> {
    Ok(BatchesResponse {
        batches: deps
            .storage
            .iter_range::<S>(Order::Ascending, start_after.unwrap_or(BatchId::ONE)..)
            .take(limit.unwrap_or(usize::MAX))
            .map_ok(|(batch_id, batch)| IdentifiedBatch { batch_id, batch })
            .collect::<Result<_, _>>()?,
    })
}

pub fn query_batches_by_ids(
    deps: Deps,
    batch_ids: &[BatchId],
) -> Result<BatchesResponse<Batch>, ContractError> {
    Ok(BatchesResponse {
        batches: batch_ids
            .iter()
            .map(|&batch_id| {
                // TODO: This is a naive query, it can be made more efficient
                query_batch(deps, batch_id)
                    .and_then(|batch| batch.ok_or(ContractError::BatchNotFound { batch_id }))
                    .map(|batch| IdentifiedBatch { batch_id, batch })
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn query_pending_batch(deps: Deps) -> StdResult<PendingBatch> {
    deps.storage.read_item::<CurrentPendingBatch>()
}

pub fn query_unstake_requests_by_staker_hash(
    deps: Deps,
    staker_hash: H256,
) -> StdResult<Vec<UnstakeRequest>> {
    deps.storage
        .iter_range::<UnstakeRequestsByStakerHash>(
            Order::Ascending,
            UnstakeRequestKey {
                batch_id: BatchId::ONE,
                staker_hash,
            }..=UnstakeRequestKey {
                batch_id: BatchId::MAX,
                staker_hash,
            },
        )
        .map_ok(|(_, unstake_request)| unstake_request)
        .collect()
}

pub fn query_all_unstake_requests(
    deps: Deps,
    start_after: Option<UnstakeRequestKey>,
    limit: Option<usize>,
) -> StdResult<Vec<UnstakeRequest>> {
    deps.storage
        .iter_range::<UnstakeRequestsByStakerHash>(
            Order::Ascending,
            start_after.unwrap_or(UnstakeRequestKey {
                batch_id: BatchId::ONE,
                staker_hash: H256::new([0x00; 32]),
            })..,
        )
        .take(limit.unwrap_or(usize::MAX))
        .map_ok(|(_, unstake_request)| unstake_request)
        .collect()
}
