use std::collections::BTreeMap;

use cosmwasm_std::{
    from_json,
    testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, ContractResult, DecCoin, Decimal, DistributionMsg, Event, Order,
    OwnedDeps, QuerierResult, Response, Uint128, WasmQuery,
};
use cw_account::{
    state::{Admins, Zkgm},
    types::{Admin, LocalAdmin},
};
use depolama::StorageExt;
use lst::{msg::ConfigResponse, types::ProtocolFeeConfig};

use crate::{execute, msg::ExecuteMsg, withdraw_all_rewards, ContractError};

const ADMIN: &str = "admin";
const LST_HUB: &str = "lst-hub";

fn setup_local() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let admin = Addr::unchecked(ADMIN);

    let res = cw_account::init(
        deps.as_mut(),
        cw_account::msg::InitMsg::Local {
            admin: admin.clone(),
        },
    );

    assert_eq!(res, Response::new());

    // only local admin is written to storage
    assert_eq!(
        deps.storage
            .iter::<Admins>(Order::Ascending)
            .map(|r| r.unwrap().0)
            .collect::<Vec<_>>(),
        &[Admin::Local(LocalAdmin {
            address: admin.to_string(),
        })]
    );

    // zkgm does not exist after a local admin init
    assert!(deps.storage.maybe_read_item::<Zkgm>().unwrap().is_none());

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::SetLstHubAddress(Addr::unchecked(LST_HUB)),
    )
    .unwrap();

    deps
}

#[test]
fn validator_shares_overflow() {
    let mut deps = setup_local();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN), &[]),
            ExecuteMsg::SetValidators(
                [
                    (Addr::unchecked("val-1"), Uint128::new(u128::MAX)),
                    (Addr::unchecked("val-2"), Uint128::new(1)),
                ]
                .into_iter()
                .collect()
            ),
        )
        .unwrap_err(),
        ContractError::TooManyShares,
    );
}

#[test]
fn admin_ops_require_admin() {
    let mut deps = setup_local();

    let non_admin = Addr::unchecked("non-admin");

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&non_admin, &[]),
            ExecuteMsg::SetValidators(BTreeMap::new()),
        )
        .unwrap_err(),
        ContractError::CwAccount(cw_account::ContractError::OnlyAdmin {
            sender: Admin::Local(LocalAdmin {
                address: non_admin.to_string(),
            }),
        }),
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&non_admin, &[]),
            ExecuteMsg::SetLstHubAddress(Addr::unchecked("")),
        )
        .unwrap_err(),
        ContractError::CwAccount(cw_account::ContractError::OnlyAdmin {
            sender: Admin::Local(LocalAdmin {
                address: non_admin.to_string(),
            }),
        }),
    );
}

#[test]
fn lst_ops_require_lst() {
    let mut deps = setup_local();

    let lst_hub = Addr::unchecked(LST_HUB);
    let non_admin = Addr::unchecked("non-admin");

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN), &[]),
            ExecuteMsg::SetLstHubAddress(lst_hub.clone()),
        )
        .unwrap(),
        Response::new()
            .add_event(Event::new("set_lst_hub_address").add_attribute("address", &lst_hub)),
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&non_admin, &[]),
            ExecuteMsg::Staker(lst::msg::StakerExecuteMsg::Stake {}),
        )
        .unwrap_err(),
        ContractError::OnlyLstHub {
            sender: non_admin.clone(),
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&non_admin, &[]),
            ExecuteMsg::Staker(lst::msg::StakerExecuteMsg::Rebase {}),
        )
        .unwrap_err(),
        ContractError::OnlyLstHub {
            sender: non_admin.clone(),
        },
    );
}

#[test]
fn withdraw_all_rewards_floors_correctly() {
    let mut deps = setup_local();
    let env = mock_env();

    deps.querier.update_wasm(|w| match w {
        WasmQuery::Smart { contract_addr, msg } => match &**contract_addr {
            LST_HUB => match from_json::<lst::msg::QueryMsg>(msg).unwrap() {
                lst::msg::QueryMsg::Config {} => QuerierResult::Ok(ContractResult::Ok(
                    to_json_binary(&ConfigResponse {
                        native_token_denom: "au".to_owned(),
                        minimum_liquid_stake_amount: Default::default(),
                        protocol_fee_config: ProtocolFeeConfig {
                            fee_rate: Default::default(),
                            fee_recipient: "".to_owned(),
                        },
                        monitors: Default::default(),
                        lst_address: Addr::unchecked(""),
                        staker_address: Addr::unchecked(""),
                        batch_period_seconds: Default::default(),
                        unbonding_period_seconds: Default::default(),
                        stopped: Default::default(),
                    })
                    .unwrap(),
                )),
                _ => todo!(),
            },
            _ => todo!(),
        },
        _ => todo!(),
    });

    let (native_token_denom, total_pending_rewards, withdraw_msgs) =
        withdraw_all_rewards(deps.as_ref(), &env).unwrap();

    assert_eq!(native_token_denom, "au");
    assert_eq!(total_pending_rewards, Uint128::zero());
    assert_eq!(withdraw_msgs.collect::<Vec<_>>(), vec![]);

    deps.querier.distribution.set_rewards(
        "val-1",
        &env.contract.address,
        vec![DecCoin::new("1.5".parse::<Decimal>().unwrap(), "au")],
    );

    let (native_token_denom, total_pending_rewards, withdraw_msgs) =
        withdraw_all_rewards(deps.as_ref(), &env).unwrap();

    assert_eq!(native_token_denom, "au");
    assert_eq!(total_pending_rewards, Uint128::new(1));
    assert_eq!(
        withdraw_msgs.collect::<Vec<_>>(),
        vec![DistributionMsg::WithdrawDelegatorReward {
            validator: "val-1".to_owned()
        }]
    );

    deps.querier.distribution.set_rewards(
        "val-2",
        &env.contract.address,
        vec![DecCoin::new("0.5".parse::<Decimal>().unwrap(), "au")],
    );

    let (native_token_denom, total_pending_rewards, withdraw_msgs) =
        withdraw_all_rewards(deps.as_ref(), &env).unwrap();

    assert_eq!(native_token_denom, "au");
    assert_eq!(total_pending_rewards, Uint128::new(1));
    assert_eq!(
        withdraw_msgs.collect::<Vec<_>>(),
        vec![
            DistributionMsg::WithdrawDelegatorReward {
                validator: "val-1".to_owned()
            },
            DistributionMsg::WithdrawDelegatorReward {
                validator: "val-2".to_owned()
            }
        ]
    );
}
