use cosmwasm_std::{testing::mock_dependencies, to_json_binary};
use depolama::StorageExt;
use ibc_union_msg::{
    lightclient::VerifyCreationResponse,
    msg::{
        ExecuteMsg, InitMsg, MsgConnectionOpenAck, MsgConnectionOpenConfirm, MsgConnectionOpenInit,
        MsgConnectionOpenTry,
    },
};
use ibc_union_spec::Connection;

use super::*;
use crate::{contract::init, state::Connections};

#[test]
fn connection_open_init_ok() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    let msg = MsgConnectionOpenInit {
        client_id: ClientId!(1),
        counterparty_client_id: ClientId!(2),
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
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    connection_open_init(deps.as_mut()).expect("open connection init is ok");

    assert_eq!(
        deps.storage.read::<Connections>(&ConnectionId!(1)).unwrap(),
        Connection {
            state: ConnectionState::Init,
            client_id: ClientId!(1),
            counterparty_client_id: ClientId!(2),
            counterparty_connection_id: None
        }
    );
}

#[test]
fn connection_open_try_ok() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    let msg = MsgConnectionOpenTry {
        counterparty_client_id: ClientId!(2),
        counterparty_connection_id: ConnectionId!(1),
        client_id: ClientId!(1),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
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
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");

    let msg = MsgConnectionOpenTry {
        counterparty_client_id: ClientId!(2),
        counterparty_connection_id: ConnectionId!(1),
        client_id: ClientId!(1),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&mock_addr(SENDER), &[]),
            ExecuteMsg::ConnectionOpenTry(msg),
        ),
        Err(ContractError::Std(StdError::generic_err(
            "key 0x00000001 not present"
        )))
    );
}

// #[test]
// fn connection_open_try_invalid_proof() {
//     todo!()
// }

#[test]
fn connection_open_try_commitment_saved() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    let msg = MsgConnectionOpenTry {
        counterparty_client_id: ClientId!(2),
        counterparty_connection_id: ConnectionId!(1),
        client_id: ClientId!(1),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenTry(msg),
    )
    .expect("connection open try is ok");

    assert_eq!(
        deps.storage.read::<Connections>(&ConnectionId!(1)).unwrap(),
        Connection {
            state: ConnectionState::TryOpen,
            client_id: ClientId!(1),
            counterparty_client_id: ClientId!(2),
            counterparty_connection_id: Some(ConnectionId!(1))
        }
    );
}

#[test]
fn connection_open_ack_ok() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    connection_open_init(deps.as_mut()).expect("connection open init is ok");

    let msg = MsgConnectionOpenAck {
        connection_id: ConnectionId!(1),
        counterparty_connection_id: ConnectionId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenAck(msg),
    )
    .is_ok())
}

// #[test]
// fn connection_open_ack_invalid_proof() {
// todo!()
// }

#[test]
fn connection_open_ack_commitment_saved() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    connection_open_init(deps.as_mut()).expect("connection open init is ok");

    let msg = MsgConnectionOpenAck {
        connection_id: ConnectionId!(1),
        counterparty_connection_id: ConnectionId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenAck(msg),
    )
    .expect("connection open ack is ok");

    assert_eq!(
        deps.storage.read::<Connections>(&ConnectionId!(1)).unwrap(),
        Connection {
            state: ConnectionState::Open,
            client_id: ClientId!(1),
            counterparty_client_id: ClientId!(2),
            counterparty_connection_id: Some(ConnectionId!(1))
        }
    );
}

#[test]
fn connection_open_confirm_ok() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    connection_open_try(deps.as_mut()).expect("connection open try is ok");

    let msg = MsgConnectionOpenConfirm {
        connection_id: ConnectionId!(1),
        proof_ack: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ConnectionOpenConfirm(msg),
    )
    .is_ok());
}

// #[test]
// fn connection_open_confirm_invalid_proof() {
//     todo!()
// }

#[test]
fn connection_open_try_confirm_commitment_saved() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                events: vec![],
                storage_writes: Default::default(),
                client_state_bytes: None,
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetLatestHeight { .. } => to_json_binary(&1),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    connection_open_try(deps.as_mut()).expect("connection open try is ok");

    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    assert_eq!(
        deps.storage.read::<Connections>(&ConnectionId!(1)).unwrap(),
        Connection {
            state: ConnectionState::Open,
            client_id: ClientId!(1),
            counterparty_client_id: ClientId!(2),
            counterparty_connection_id: Some(ConnectionId!(1))
        }
    );
}
