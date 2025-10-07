use std::{fmt::Debug, sync::LazyLock};

use access_manager_types::{
    manager::{
        event::RoleGranted,
        msg::{InitMsg, QueryMsg},
    },
    time::Delay,
    Access, RoleId,
};
use cosmwasm_std::{
    from_json,
    testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
    Addr, Deps, Env, OwnedDeps, Response,
};
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use serde::de::DeserializeOwned;

use crate::{migrate, query, state::RoleMembers};

pub static ADMIN: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("admin"));

pub static ACCOUNT_1: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("account-1"));
pub static ACCOUNT_2: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("account-2"));

pub static TARGET_1: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("target-1"));
pub static TARGET_2: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("target-2"));

pub fn setup() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env) {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let res = migrate(
        deps.as_mut(),
        env.clone(),
        UpgradeMsg::Init(InitMsg {
            initial_admin: ADMIN.clone(),
        }),
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new().add_event(RoleGranted {
            role_id: RoleId::ADMIN_ROLE,
            account: &ADMIN,
            delay: 0,
            since: env.block.time.seconds(),
            new_member: true,
        }),
    );

    assert_eq!(
        deps.storage
            .read::<RoleMembers>(&(RoleId::ADMIN_ROLE, ADMIN.clone()))
            .unwrap(),
        Access {
            since: env.block.time.seconds(),
            delay: Delay::new(0),
        },
    );

    (deps, env)
}

#[track_caller]
pub(crate) fn assert_query_result<T: Debug + PartialEq + DeserializeOwned>(
    deps: Deps,
    env: &Env,
    msg: QueryMsg,
    expected: &T,
) {
    let res = query(deps, env.clone(), msg).unwrap();
    assert_eq!(&from_json::<T>(res).unwrap(), expected);
}
