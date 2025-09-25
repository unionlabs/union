use std::collections::BTreeMap;

use cosmwasm_std::{
    testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
    Addr, Event, Order, OwnedDeps, Response, Uint128,
};
use cw_account::{
    state::{Admins, Zkgm},
    types::{Admin, LocalAdmin},
};
use depolama::StorageExt;

use crate::{execute, msg::ExecuteMsg, ContractError};

const ADMIN: &str = "admin";

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

    let lst_hub = Addr::unchecked("lst-hub");
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
