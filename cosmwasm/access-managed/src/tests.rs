use std::{
    fmt::Debug,
    num::NonZero,
    sync::{Arc, LazyLock, Mutex},
};

use access_manager_types::{
    CanCall,
    managed::{
        error::AccessManagedError,
        event::AuthorityUpdated,
        msg::{ExecuteMsg, InitMsg, QueryMsg},
    },
    manager,
};
use cosmwasm_std::{
    Addr, Binary, ContractInfoResponse, ContractResult, Deps, Env, OwnedDeps, Reply, Response,
    StdError, SubMsg, SubMsgResponse, SubMsgResult, SystemResult, from_json,
    testing::{MockApi, MockQuerier, MockStorage, message_info, mock_dependencies, mock_env},
    to_json_binary, to_json_string, wasm_execute,
};
use frissitheto::UpgradeMsg;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID, EnsureCanCallResult, Restricted,
    error::ContractError, execute, migrate, query, reply, state::Authority,
};

pub static MANAGER: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("manager"));

pub static ACCOUNT_1: LazyLock<Addr> = LazyLock::new(|| Addr::unchecked("account-1"));

pub fn setup() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env) {
    let mut deps = mock_dependencies();
    let env = mock_env();

    migrate(
        deps.as_mut(),
        env.clone(),
        UpgradeMsg::Init(InitMsg {
            initial_authority: MANAGER.clone(),
        }),
    )
    .unwrap();

    assert_query_result(deps.as_ref(), &env, QueryMsg::Authority {}, &*MANAGER);

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
enum TestExecuteMsg {
    A {},
}

#[test]
fn ensure_can_call() {
    let (mut deps, env) = setup();

    let can_call_result = Arc::new(Mutex::new(CanCall::Unauthorized {}));

    deps.querier.update_wasm({
        let can_call_result = can_call_result.clone();
        move |_| {
            SystemResult::Ok(ContractResult::Ok(
                to_json_binary(&*can_call_result.lock().unwrap()).unwrap(),
            ))
        }
    });

    assert_eq!(
        Restricted::wrap(TestExecuteMsg::A {})
            .ensure_can_call::<Authority>(deps.as_mut(), &env, &message_info(&ACCOUNT_1, &[]))
            .unwrap_err(),
        ContractError::AccessManaged(
            access_manager_types::managed::error::AccessManagedError::AccessManagedUnauthorized {
                caller: ACCOUNT_1.clone(),
            },
        ),
    );

    *can_call_result.lock().unwrap() = CanCall::Immediate {};

    assert_eq!(
        Restricted::wrap(TestExecuteMsg::A {})
            .ensure_can_call::<Authority>(deps.as_mut(), &env, &message_info(&ACCOUNT_1, &[]))
            .unwrap(),
        EnsureCanCallResult::Msg(TestExecuteMsg::A {}),
    );

    *can_call_result.lock().unwrap() = CanCall::WithDelay {
        delay: NonZero::new(1).unwrap(),
    };

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::IsConsumingScheduledOp {},
        &String::new(),
    );

    assert_eq!(
        Restricted::wrap(TestExecuteMsg::A {})
            .ensure_can_call::<Authority>(deps.as_mut(), &env, &message_info(&ACCOUNT_1, &[]))
            .unwrap(),
        EnsureCanCallResult::Scheduled(
            [
                SubMsg::reply_never(
                    wasm_execute(
                        &*MANAGER,
                        &manager::msg::ExecuteMsg::ConsumeScheduledOp {
                            caller: ACCOUNT_1.clone(),
                            data: to_json_string(&TestExecuteMsg::A {}).unwrap(),
                        },
                        vec![],
                    )
                    .unwrap()
                ),
                SubMsg::reply_on_success(
                    wasm_execute(&env.contract.address, &TestExecuteMsg::A {}, vec![]).unwrap(),
                    ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID,
                ),
            ]
            .to_vec()
        ),
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::IsConsumingScheduledOp {},
        &"is_consuming_scheduled_op".to_owned(),
    );

    assert_eq!(
        reply(
            deps.as_mut(),
            env.clone(),
            Reply {
                id: ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID,
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
        .unwrap(),
        Response::new(),
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::IsConsumingScheduledOp {},
        &String::new(),
    );
}

#[test]
fn set_authority() {
    let (mut deps, env) = setup();

    let new_authority = Addr::unchecked("new-authority");

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&Addr::unchecked("unauthorized"), &[]),
            ExecuteMsg::SetAuthority {
                new_authority: new_authority.clone()
            },
        )
        .unwrap_err(),
        ContractError::AccessManaged(AccessManagedError::AccessManagedUnauthorized {
            caller: Addr::unchecked("unauthorized")
        })
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&MANAGER, &[]),
            ExecuteMsg::SetAuthority {
                new_authority: new_authority.clone()
            },
        )
        .unwrap_err(),
        ContractError::AccessManaged(AccessManagedError::AccessManagedInvalidAuthority {
            authority: new_authority.clone()
        })
    );

    deps.querier.update_wasm(move |_| {
        SystemResult::Ok(ContractResult::Ok(
            to_json_binary(&ContractInfoResponse::new(
                0,
                Addr::unchecked(""),
                None,
                false,
                None,
            ))
            .unwrap(),
        ))
    });

    assert_query_result(deps.as_ref(), &env, QueryMsg::Authority {}, &*MANAGER);

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&MANAGER, &[]),
            ExecuteMsg::SetAuthority {
                new_authority: new_authority.clone()
            },
        )
        .unwrap(),
        Response::new().add_event(AuthorityUpdated {
            authority: &new_authority
        })
    );

    assert_query_result(deps.as_ref(), &env, QueryMsg::Authority {}, &new_authority);
}

#[test]
#[should_panic = "invariant: attempted to handle consume scheduled op reply while not consuming schedule"]
fn reply_consuming_scheduled_op_invariant() {
    let (mut deps, env) = setup();

    reply(
        deps.as_mut(),
        env,
        Reply {
            id: ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID,
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
}

#[test]
fn reply_unknown_id() {
    let (mut deps, env) = setup();

    assert_eq!(
        reply(
            deps.as_mut(),
            env,
            Reply {
                id: 123,
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
        .unwrap_err(),
        ContractError::Std(StdError::generic_err(
            "unknown reply: Reply { id: 123, payload: Binary(), gas_used: 0, result: Ok(SubMsgResponse { events: [], data: None, msg_responses: [] }) }"
        )),
    );
}
