use std::marker::PhantomData;

use contract::{events::attribute::CLIENT_ID, instantiate};
use cosmwasm_std::{testing::mock_dependencies, to_json_binary};
use ibc_solidity::Connection;
use union_ibc_msg::msg::{
    ExecuteMsg, InitMsg, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
    MsgConnectionOpenTry, MsgRegisterClient,
};
use unionlabs::ibc::core::client::height::Height;

use super::*;

const CLIENT_TYPE: &str = "union";
const CLIENT_ADDRESS: &str = "unionclient";
const SENDER: &str = "unionsender";
const RELAYER: &str = "unionrelayer";

fn connection_open_init(
    deps: DepsMut,
    client_id: impl Into<u32>,
    counterparty_client_id: impl Into<u32>,
    sender_address_seed: impl Into<String>,
    relayer_address_seed: impl Into<String>,
) -> Result<Response, ContractError> {
    let msg = MsgConnectionOpenInit {
        client_id: client_id.into(),
        counterparty_client_id: counterparty_client_id.into(),
        relayer: mock_addr(relayer_address_seed).into_string(),
    };
    execute(
        deps,
        mock_env(),
        message_info(&mock_addr(sender_address_seed), &[]),
        ExecuteMsg::ConnectionOpenInit(msg),
    )
}

fn connection_open_try(
    deps: DepsMut,
    counterparty_client_id: impl Into<u32>,
    counterparty_connection_id: impl Into<u32>,
    client_id: impl Into<u32>,
    proof_height: impl Into<u64>,
    relayer_address_seed: impl Into<String>,
) -> Result<Response, ContractError> {
    let msg = MsgConnectionOpenTry {
        counterparty_client_id: counterparty_client_id.into(),
        counterparty_connection_id: counterparty_connection_id.into(),
        client_id: client_id.into(),
        proof_init: vec![1, 2, 3].into(),
        proof_height: proof_height.into(),
        relayer: mock_addr(relayer_address_seed).into_string(),
    };

    execute(
        deps,
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenTry(msg),
    )
}

#[test]
fn connection_open_init_ok() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");
    create_client(deps.as_mut(), CLIENT_TYPE, SENDER, RELAYER).expect("create client ok");

    let msg = MsgConnectionOpenInit {
        client_id: 1,
        counterparty_client_id: 2,
        relayer: mock_addr(RELAYER).into_string(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenInit(msg),
    )
    .is_ok())
}

#[test]
fn connection_open_init_commitment_saved() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");
    create_client(deps.as_mut(), CLIENT_TYPE, SENDER, RELAYER).expect("create client ok");
    connection_open_init(deps.as_mut(), 1_u32, 2_u32, SENDER, RELAYER)
        .expect("open connection init is ok");

    assert_eq!(
        crate::state::CONNECTIONS.load(&deps.storage, 1).unwrap(),
        Connection {
            state: ConnectionState::Init,
            client_id: 1,
            counterparty_client_id: 2,
            counterparty_connection_id: 0
        }
    );
}

#[test]
fn connection_open_try_ok() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");
    create_client(deps.as_mut(), CLIENT_TYPE, SENDER, RELAYER).expect("create client ok");

    let msg = MsgConnectionOpenTry {
        counterparty_client_id: 2,
        counterparty_connection_id: 1,
        client_id: 1,
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenTry(msg),
    )
    .is_ok());
}

#[test]
fn connection_open_try_client_not_found() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");

    let msg = MsgConnectionOpenTry {
        counterparty_client_id: 2,
        counterparty_connection_id: 1,
        client_id: 1,
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenTry(msg),
    )
    .is_err_and(|err| {
        match err {
            ContractError::Std(err) => matches!(err, StdError::NotFound { .. }),
            _ => false,
        }
    }));
}

// #[test]
// fn connection_open_try_invalid_proof() {
//     todo!()
// }

#[test]
fn connection_open_try_commitment_saved() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");
    create_client(deps.as_mut(), CLIENT_TYPE, SENDER, RELAYER).expect("create client ok");

    let msg = MsgConnectionOpenTry {
        counterparty_client_id: 2,
        counterparty_connection_id: 1,
        client_id: 1,
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenTry(msg),
    )
    .expect("connection open try is ok");

    assert_eq!(
        crate::state::CONNECTIONS.load(&deps.storage, 1).unwrap(),
        Connection {
            state: ConnectionState::TryOpen,
            client_id: 1,
            counterparty_client_id: 2,
            counterparty_connection_id: 1
        }
    );
}

#[test]
fn connection_open_ack_ok() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");
    create_client(deps.as_mut(), CLIENT_TYPE, SENDER, RELAYER).expect("create client ok");
    connection_open_init(deps.as_mut(), 1_u32, 2_u32, SENDER, RELAYER)
        .expect("connection open init is ok");

    let msg = MsgConnectionOpenAck {
        connection_id: 1,
        counterparty_connection_id: 1,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(dbg!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenAck(msg),
    ))
    .is_ok())
}

// #[test]
// fn connection_open_ack_invalid_proof() {
// todo!()
// }

#[test]
fn connection_open_ack_commitment_saved() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");
    create_client(deps.as_mut(), CLIENT_TYPE, SENDER, RELAYER).expect("create client ok");
    connection_open_init(deps.as_mut(), 1_u32, 2_u32, SENDER, RELAYER)
        .expect("connection open init is ok");

    let msg = MsgConnectionOpenAck {
        connection_id: 1,
        counterparty_connection_id: 1,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenAck(msg),
    )
    .expect("connection open ack is ok");

    assert_eq!(
        crate::state::CONNECTIONS.load(&deps.storage, 1).unwrap(),
        Connection {
            state: ConnectionState::Open,
            client_id: 1,
            counterparty_client_id: 2,
            counterparty_connection_id: 1
        }
    );
}

#[test]
fn connection_open_confirm_ok() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");
    create_client(deps.as_mut(), CLIENT_TYPE, SENDER, RELAYER).expect("create client ok");
    connection_open_try(deps.as_mut(), 2_u32, 1_u32, 1_u32, 1_u64, RELAYER)
        .expect("connection open try is ok");

    let msg = MsgConnectionOpenConfirm {
        connection_id: 1,
        proof_ack: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(dbg!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenConfirm(msg),
    ))
    .is_ok());
}

// #[test]
// fn connection_open_confirm_invalid_proof() {
//     todo!()
// }

#[test]
fn connection_open_try_confirm_commitment_saved() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut(), CLIENT_TYPE, CLIENT_ADDRESS, SENDER)
        .expect("register client ok");
    create_client(deps.as_mut(), CLIENT_TYPE, SENDER, RELAYER).expect("create client ok");
    connection_open_try(deps.as_mut(), 2_u32, 1_u32, 1_u32, 1_u64, RELAYER)
        .expect("connection open try is ok");

    let msg = MsgConnectionOpenConfirm {
        connection_id: 1,
        proof_ack: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenConfirm(msg),
    )
    .expect("connection_open_confirm is ok");

    assert_eq!(
        crate::state::CONNECTIONS.load(&deps.storage, 1).unwrap(),
        Connection {
            state: ConnectionState::Open,
            client_id: 1,
            counterparty_client_id: 2,
            counterparty_connection_id: 1
        }
    );
}
