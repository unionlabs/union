use std::{fmt::Debug, num::NonZero, sync::LazyLock};

use access_manager::EXECUTE_REPLY_ID;
use access_manager_types::{
    CanCall, Nonce, RoleId, Selector,
    manager::{
        self,
        error::AccessManagerError,
        event::{OperationExecuted, OperationScheduled, RoleGranted},
        msg::{InitMsg, QueryMsg},
    },
};
use cosmwasm_std::{
    Addr, Binary, Deps, Env, Event, OwnedDeps, Reply, Response, SubMsg, SubMsgResponse,
    SubMsgResult, WasmMsg, from_json,
    testing::{MockApi, MockQuerier, MockStorage, message_info, mock_dependencies, mock_env},
    to_json_binary, to_json_string,
};
use frissitheto::UpgradeMsg;
use hex_literal::hex;
use serde::de::DeserializeOwned;
use unionlabs_primitives::H256;
use upgradable::msg::{Upgradable, Value};

use crate::{error::ContractError, execute, migrate, msg::ExecuteMsg, query, reply};

pub static ADMIN: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("admin"));

pub static ACCOUNT_1: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("account-1"));

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

#[test]
fn upgrade_only_authorized() {
    let (mut deps, env) = setup();

    let msg = ExecuteMsg::Upgradable(Upgradable::Upgrade {
        new_code_id: NonZero::new(1).unwrap(),
        msg: Value::Null,
    });

    // not authorized yet
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            msg.clone(),
        )
        .unwrap_err(),
        ContractError::AccessManager(access_manager::error::ContractError::AccessManager(
            AccessManagerError::AccessManagerUnauthorizedAccount {
                msg_sender: ACCOUNT_1.clone(),
                required_role_id: RoleId::ADMIN_ROLE,
            }
        ))
    );

    // set up roles so that account-1 can call upgrade on access manager
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        manager::msg::ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        }
        .into(),
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        manager::msg::ExecuteMsg::SetTargetFunctionRole {
            role_id: RoleId::new(1),
            target: env.contract.address.clone(),
            selectors: vec![Selector::extract_from_serialize(&msg).to_owned()],
        }
        .into(),
    )
    .unwrap();

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::extract_from_serialize(&msg).to_owned(),
            target: env.contract.address.clone(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall::Immediate {},
    );

    // authorized now
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            msg.clone(),
        )
        .unwrap(),
        Response::new()
            .add_message(WasmMsg::Migrate {
                contract_addr: env.contract.address.to_string(),
                new_code_id: 1,
                msg: br#"{"migrate":null}"#.into()
            })
            .add_event(
                Event::new("upgrade")
                    .add_attribute("new_code_id", "1")
                    .add_attribute("msg", "null")
            ),
    );
}

#[test]
fn upgrade_only_authorized_schedule() {
    let (mut deps, mut env) = setup();

    let operation_id = <H256>::new(hex!(
        "d680bcc48c06b386d582e40d9594f518ea334ab3077d08e655259bf30080dd2e"
    ));

    let msg = ExecuteMsg::Upgradable(Upgradable::Upgrade {
        new_code_id: NonZero::new(1).unwrap(),
        msg: Value::Null,
    });

    // set up roles so that account-1 can call upgrade on access manager with a delay
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        manager::msg::ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 10,
        }
        .into(),
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        manager::msg::ExecuteMsg::SetTargetFunctionRole {
            role_id: RoleId::new(1),
            target: env.contract.address.clone(),
            selectors: vec![Selector::extract_from_serialize(&msg).to_owned()],
        }
        .into(),
    )
    .unwrap();

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::extract_from_serialize(&msg).to_owned(),
            target: env.contract.address.clone(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall::WithDelay {
            delay: const { NonZero::new(10).unwrap() },
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            manager::msg::ExecuteMsg::Schedule {
                target: env.contract.address.clone(),
                data: to_json_string(&msg).unwrap(),
                when: NonZero::new(env.block.time.seconds() + 10).unwrap()
            }
            .into(),
        )
        .unwrap(),
        Response::new()
            .add_event(OperationScheduled {
                operation_id,
                nonce: Nonce::new(1),
                schedule: NonZero::new(env.block.time.seconds() + 10).unwrap(),
                caller: &ACCOUNT_1,
                target: &env.contract.address,
                data: &to_json_string(&msg).unwrap(),
            })
            .set_data(
                br#"["0xd680bcc48c06b386d582e40d9594f518ea334ab3077d08e655259bf30080dd2e",1]"#,
            ),
    );

    // operation has been scheduled, nonce is now 1 and timepoint has been set
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(1),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &Some(NonZero::new(env.block.time.seconds() + 10).unwrap()),
    );

    // not ready yet
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            manager::msg::ExecuteMsg::Execute {
                target: env.contract.address.clone(),
                data: to_json_string(&msg).unwrap(),
            }
            .into(),
        )
        .unwrap_err(),
        ContractError::AccessManager(access_manager::error::ContractError::AccessManager(
            AccessManagerError::AccessManagerNotReady(operation_id)
        )),
    );

    env.block.time = env.block.time.plus_seconds(10);

    // ready now, so should successfully execute
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            manager::msg::ExecuteMsg::Execute {
                target: env.contract.address.clone(),
                data: to_json_string(&msg).unwrap(),
            }
            .into(),
        )
        .unwrap(),
        Response::new()
            .add_submessage(SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_json_binary(&msg).unwrap(),
                    funds: vec![]
                },
                1
            ))
            .add_event(OperationExecuted {
                operation_id,
                nonce: Nonce::new(1),
            })
            .set_data(b"1"),
    );

    // NOTE: during execution
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::extract_from_serialize(&msg).to_owned(),
            target: env.contract.address.clone(),
            caller: env.contract.address.clone(),
        },
        &CanCall::Immediate {},
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&env.contract.address, &[]),
            msg.clone(),
        )
        .unwrap(),
        Response::new()
            .add_message(WasmMsg::Migrate {
                contract_addr: env.contract.address.to_string(),
                new_code_id: 1,
                msg: br#"{"migrate":null}"#.into()
            })
            .add_event(
                Event::new("upgrade")
                    .add_attribute("new_code_id", "1")
                    .add_attribute("msg", "null")
            ),
    );

    // still executing
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::extract_from_serialize(&msg).to_owned(),
            target: env.contract.address.clone(),
            caller: env.contract.address.clone(),
        },
        &CanCall::Immediate {},
    );

    reply(
        deps.as_mut(),
        env.clone(),
        Reply {
            id: EXECUTE_REPLY_ID,
            payload: Binary::default(),
            gas_used: 0,
            result: SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                #[expect(deprecated, reason = "need to construct this type somehow")]
                data: None,
                msg_responses: vec![],
            }),
        },
    )
    .unwrap();

    // execution finished
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::extract_from_serialize(&msg).to_owned(),
            target: env.contract.address.clone(),
            caller: env.contract.address.clone(),
        },
        &CanCall::Unauthorized {},
    );
}

#[test]
fn upgrade_only_authorized_delay_direct() {
    let (mut deps, mut env) = setup();

    let operation_id = <H256>::new(hex!(
        "d680bcc48c06b386d582e40d9594f518ea334ab3077d08e655259bf30080dd2e"
    ));

    let msg = ExecuteMsg::Upgradable(Upgradable::Upgrade {
        new_code_id: NonZero::new(1).unwrap(),
        msg: Value::Null,
    });

    // set up roles so that account-1 can call upgrade on access manager with a delay
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        manager::msg::ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 10,
        }
        .into(),
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        manager::msg::ExecuteMsg::SetTargetFunctionRole {
            role_id: RoleId::new(1),
            target: env.contract.address.clone(),
            selectors: vec![Selector::extract_from_serialize(&msg).to_owned()],
        }
        .into(),
    )
    .unwrap();

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::extract_from_serialize(&msg).to_owned(),
            target: env.contract.address.clone(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall::WithDelay {
            delay: const { NonZero::new(10).unwrap() },
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            manager::msg::ExecuteMsg::Schedule {
                target: env.contract.address.clone(),
                data: to_json_string(&msg).unwrap(),
                when: NonZero::new(env.block.time.seconds() + 10).unwrap()
            }
            .into(),
        )
        .unwrap(),
        Response::new()
            .add_event(OperationScheduled {
                operation_id,
                nonce: Nonce::new(1),
                schedule: NonZero::new(env.block.time.seconds() + 10).unwrap(),
                caller: &ACCOUNT_1,
                target: &env.contract.address,
                data: &to_json_string(&msg).unwrap(),
            })
            .set_data(
                br#"["0xd680bcc48c06b386d582e40d9594f518ea334ab3077d08e655259bf30080dd2e",1]"#,
            ),
    );

    // operation has been scheduled, nonce is now 1 and timepoint has been set
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(1),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &Some(NonZero::new(env.block.time.seconds() + 10).unwrap()),
    );

    // not ready yet
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            manager::msg::ExecuteMsg::Execute {
                target: env.contract.address.clone(),
                data: to_json_string(&msg).unwrap(),
            }
            .into(),
        )
        .unwrap_err(),
        ContractError::AccessManager(access_manager::error::ContractError::AccessManager(
            AccessManagerError::AccessManagerNotReady(operation_id)
        )),
    );

    env.block.time = env.block.time.plus_seconds(10);

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::extract_from_serialize(&msg).to_owned(),
            target: env.contract.address.clone(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall::WithDelay {
            delay: NonZero::new(10).unwrap(),
        },
    );

    // ready now, so should successfully execute
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            msg.clone(),
        )
        .unwrap(),
        Response::new()
            .add_message(WasmMsg::Migrate {
                contract_addr: env.contract.address.to_string(),
                new_code_id: 1,
                msg: br#"{"migrate":null}"#.into()
            })
            .add_event(
                Event::new("upgrade")
                    .add_attribute("new_code_id", "1")
                    .add_attribute("msg", "null")
            ),
    );

    // not authorized to call anymore
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::extract_from_serialize(&msg).to_owned(),
            target: env.contract.address.clone(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall::WithDelay {
            delay: NonZero::new(10).unwrap(),
        },
    );
}
