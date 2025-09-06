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
    testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
    Addr, OwnedDeps,
};

use crate::{contract::init, msg::InitMsg, types::ProtocolFeeConfig};

pub const NATIVE_TOKEN: &str = "au";

pub const ADMIN: &str = "union1admin";

pub const UNION1: &str = "union1union1";
pub const UNION2: &str = "union1union2";
pub const UNION3: &str = "union1union3";

pub const UNION_STAKER: &str = "union1unionstaker";

pub const FEE_RECIPIENT: &str = "union1feerecipient";

pub const UNION_MONITOR_1: &str = "union1unionmonitor1";
pub const UNION_MONITOR_2: &str = "union1unionmonitor2";

pub const LST_ADDRESS: &str = "union1lsttokenaddress";

pub fn mock_init_msg() -> InitMsg {
    InitMsg {
        staker_address: Addr::unchecked(UNION_STAKER),
        minimum_liquid_stake_amount: 100,
        lst_address: Addr::unchecked(LST_ADDRESS),
        monitors: vec![
            Addr::unchecked(UNION_MONITOR_1),
            Addr::unchecked(UNION_MONITOR_2),
        ],
        batch_period_seconds: 86400,
        unbonding_period_seconds: 1_000_000,
        protocol_fee_config: ProtocolFeeConfig {
            fee_rate: 10_000,
            fee_recipient: FEE_RECIPIENT.to_string(),
        },
        admin: Addr::unchecked(ADMIN),
        native_token_denom: "au".to_owned(),
    }
}

pub fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    // let mut deps = mock_deps_with_unbonding_time(100_000);
    let mut deps = mock_dependencies();
    let msg = mock_init_msg();

    init(deps.as_mut(), mock_env(), msg).unwrap();

    deps
}

// NOTE: Unused for now, once the unbonding period is queried from the chain directly instead of
// being in the config this will need to be used again. #[derive(Clone, PartialEq, Message)]
// pub struct Duration {
//     #[prost(int64, tag = "1")]
//     pub seconds: i64,
// }

// #[derive(Clone, PartialEq, Message)]
// pub struct Params {
//     #[prost(message, optional, tag = "1")]
//     pub unbonding_time: Option<Duration>,
// }

// #[derive(Clone, PartialEq, Message)]
// pub struct QueryParamsResponse {
//     #[prost(message, optional, tag = "1")]
//     pub params: Option<Params>,
// }

// #[derive(Clone, Default)]
// pub struct MockQuerier {
//     unbonding_time: i64,
// }

// impl cosmwasm_std::Querier for MockQuerier {
//     fn raw_query(&self, bin_request: &[u8]) -> cosmwasm_std::QuerierResult {
//         // Deserialize the request
//         let query: cosmwasm_std::QueryRequest = cosmwasm_std::from_json(bin_request).unwrap();

//         match query {
//             // this query is meant to be used for getting the unbonding time
//             cosmwasm_std::QueryRequest::Grpc(_) => {
//                 cosmwasm_std::SystemResult::Ok(cosmwasm_std::ContractResult::Ok(
//                     prost::Message::encode_to_vec(
//                         &crate::tests::test_helper::QueryParamsResponse {
//                             params: Some(crate::tests::test_helper::Params {
//                                 unbonding_time: Some(crate::tests::test_helper::Duration {
//                                     seconds: self.unbonding_time,
//                                 }),
//                             }),
//                         },
//                     )
//                     .into(),
//                 ))
//             }
//             _ => panic!("unexpected query"),
//         }
//     }
// }

// pub fn mock_deps_with_unbonding_time(
//     unbonding_time: i64,
// ) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
//     cosmwasm_std::OwnedDeps {
//         storage: cosmwasm_std::testing::MockStorage::default(),
//         api: cosmwasm_std::testing::MockApi::default().with_prefix("union"),
//         querier: MockQuerier { unbonding_time },
//         custom_query_type: std::marker::PhantomData,
//     }
// }
