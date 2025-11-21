use cosmwasm_std::{
    Api, CanonicalAddr, Checksum, CodeInfoResponse, ContractInfoResponse, ContractResult,
    OwnedDeps, SystemError, SystemResult, WasmQuery,
    testing::{MockApi, MockQuerier, MockStorage, message_info, mock_dependencies, mock_env},
};
use frissitheto::UpgradeMsg;

use super::*;

fn setup() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env) {
    let mut deps = mock_dependencies();
    let env = mock_env();

    deps.querier.update_wasm(|wq| match wq {
        WasmQuery::CodeInfo { code_id } => SystemResult::Ok(ContractResult::Ok(
            to_json_binary(&CodeInfoResponse::new(
                *code_id,
                Addr::unchecked(""),
                BYTECODE_BASE_CHECKSUM,
            ))
            .unwrap(),
        )),
        _ => todo!("{wq:?}"),
    });

    assert_eq!(
        migrate(
            deps.as_mut(),
            env.clone(),
            UpgradeMsg::Init(InitMsg {
                bytecode_base_code_id: 1,
                cw_account_code_id: 2,
            }),
        )
        .unwrap(),
        Response::new(),
    );

    // code ids are stored correctly
    assert_eq!(deps.storage.read_item::<BytecodeBaseCodeId>().unwrap(), 1);
    assert_eq!(deps.storage.read_item::<CwAccountCodeId>().unwrap(), 2);

    (deps, env)
}

#[test]
fn invalid_bytecode_base_checksum() {
    let mut deps = mock_dependencies();
    let env = mock_env();

    deps.querier.update_wasm(|wq| match wq {
        WasmQuery::CodeInfo { code_id } => SystemResult::Ok(ContractResult::Ok(
            to_json_binary(&CodeInfoResponse::new(
                *code_id,
                Addr::unchecked(""),
                Checksum::from([0; 32]),
            ))
            .unwrap(),
        )),
        _ => todo!("{wq:?}"),
    });

    assert_eq!(
        migrate(
            deps.as_mut(),
            env.clone(),
            UpgradeMsg::Init(InitMsg {
                bytecode_base_code_id: 1,
                cw_account_code_id: 2,
            }),
        )
        .unwrap_err(),
        ContractError::InvalidBytecodeBaseChecksum([0; 32].into()),
    );
}

#[test]
fn call_proxy_does_not_exist() {
    let (mut deps, env) = setup();

    deps.querier.update_wasm(|wq| match wq {
        WasmQuery::ContractInfo { contract_addr } => {
            SystemResult::Err(SystemError::NoSuchContract {
                addr: contract_addr.clone(),
            })
        }
        _ => todo!("{wq:?}"),
    });

    let addr_canonical = CanonicalAddr::from(hex!(
        "befcea71d2c552fd60ebfd7772b087de5148061f12f635353241c495576c933f"
    ));

    let addr = deps.api.addr_humanize(&addr_canonical).unwrap();

    let proxy_addr = predict_call_proxy_account(deps.as_ref(), &env, &addr).unwrap();

    let label = "cosmwasm1jpev2csrppg792t22rn8z8uew8h3sjcpglcd0qv9g8gj8ky922tscp8avs/proxy/cosmwasm1hm7w5uwjc4f06c8tl4mh9vy8meg5spslztmr2dfjg8zf24mvjvls2yv8l5".to_owned();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&addr, &[]),
            ExecuteMsg::CallProxy([].to_vec()),
        )
        .unwrap(),
        Response::new()
            .add_messages([
                WasmMsg::Instantiate2 {
                    admin: Some(env.contract.address.to_string()),
                    code_id: 1,
                    label,
                    msg: b"{}".into(),
                    funds: vec!(),
                    salt: hex!("5c9752718c7368e640b76c2adcd7dddb06da0fcc7c5b02722a71b203a875829f")
                        .into(),
                },
                WasmMsg::Migrate {
                    contract_addr: proxy_addr.to_string(),
                    new_code_id: 2,
                    msg: to_json_binary(&UpgradeMsg::<_, ()>::Init(
                        cw_account::msg::InitMsg::Local {
                            admin: env.contract.address,
                        }
                    ))
                    .unwrap()
                },
                WasmMsg::UpdateAdmin {
                    contract_addr: proxy_addr.to_string(),
                    admin: proxy_addr.to_string(),
                },
                wasm_execute(
                    proxy_addr,
                    &cw_account::msg::ExecuteMsg::Dispatch([].into()),
                    vec![]
                )
                .unwrap()
            ])
            .add_event(Event::new("proxy_created").add_attribute("creator", addr)),
    );
}

#[test]
fn call_proxy_already_exists() {
    let (mut deps, env) = setup();

    deps.querier.update_wasm(|wq| match wq {
        WasmQuery::ContractInfo { .. } => SystemResult::Ok(ContractResult::Ok(
            to_json_binary(&ContractInfoResponse::new(
                123,
                Addr::unchecked(""),
                None,
                false,
                None,
            ))
            .unwrap(),
        )),
        _ => todo!("{wq:?}"),
    });

    let addr_canonical = CanonicalAddr::from(hex!(
        "befcea71d2c552fd60ebfd7772b087de5148061f12f635353241c495576c933f"
    ));

    let addr = deps.api.addr_humanize(&addr_canonical).unwrap();

    let proxy_addr = predict_call_proxy_account(deps.as_ref(), &env, &addr).unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            env,
            message_info(&addr, &[]),
            ExecuteMsg::CallProxy([].to_vec()),
        )
        .unwrap(),
        Response::new().add_message(
            wasm_execute(
                proxy_addr,
                &cw_account::msg::ExecuteMsg::Dispatch([].into()),
                vec![]
            )
            .unwrap()
        ),
    );
}

#[test]
fn init_and_instantiate_same() {
    let mut init_deps = mock_dependencies();
    let mut instantiate_deps = mock_dependencies();

    let handler = |wq: &WasmQuery| match wq {
        WasmQuery::CodeInfo { code_id } => SystemResult::Ok(ContractResult::Ok(
            to_json_binary(&CodeInfoResponse::new(
                *code_id,
                Addr::unchecked(""),
                BYTECODE_BASE_CHECKSUM,
            ))
            .unwrap(),
        )),
        _ => todo!("{wq:?}"),
    };

    init_deps.querier.update_wasm(handler);
    instantiate_deps.querier.update_wasm(handler);

    let env = mock_env();
    let info = message_info(&Addr::unchecked(""), &[]);

    let msg = InitMsg {
        bytecode_base_code_id: 1,
        cw_account_code_id: 2,
    };

    let init_res = migrate(
        init_deps.as_mut(),
        env.clone(),
        UpgradeMsg::Init(msg.clone()),
    )
    .unwrap();
    let instantiate_res = instantiate(instantiate_deps.as_mut(), env, info, msg.clone()).unwrap();

    assert_eq!(init_res, instantiate_res);
    assert_eq!(
        format!("{:?}", init_deps.storage),
        format!("{:?}", instantiate_deps.storage)
    );
}

#[test]
fn predict_proxy_account_address_ok() {
    let deps = mock_dependencies();

    assert_eq!(
        predict_call_proxy_account(
            deps.as_ref(),
            &mock_env(),
            &Addr::unchecked("cosmwasm193dft67r2n25pqj2v63kqzrrslv0grlac5dymga29jxzy9zw4p2qm2lcu6"),
        )
        .unwrap(),
        Addr::unchecked("cosmwasm1e39e2d2tmg89d0jqkpzgfz7zgryetum2s36egvwt2z8z4kgwx0lsu5v750"),
    );
}
