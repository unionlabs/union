use contract::instantiate;
use cosmwasm_std::{testing::mock_dependencies, to_json_binary};
use union_ibc_msg::msg::{InitMsg, MsgSendPacket};

use super::*;

#[test]
fn send_packet_ok() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .expect("instantiate is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel: 1,
        timeout_height: 10,
        timeout_timestamp: 1000000,
        data: vec![0, 1, 2].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg)
    )
    .is_ok())
}

#[test]
fn send_packet_missing_timeout() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .expect("instantiate is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel: 1,
        timeout_height: 0,
        timeout_timestamp: 0,
        data: vec![0, 1, 2].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::TimeoutMustBeSet) }))
}

#[test]
fn send_packet_channel_does_not_exist() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .expect("instantiate is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel: 3,
        timeout_height: 10,
        timeout_timestamp: 1000000,
        data: vec![0, 1, 2].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .is_err_and(|err| {
        match err {
            ContractError::Std(err) => matches!(err, StdError::NotFound { .. }),
            _ => false,
        }
    }))
}

#[test]
fn send_packet_module_is_not_channel_owner() {
    let mut deps = mock_dependencies();
    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        InitMsg {},
    )
    .expect("instantiate is ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&1),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));

    // Create client
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");
    // Create connection
    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    // Create channel
    channel_open_init(deps.as_mut()).expect("channel open init is ok");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel: 1,
        timeout_height: 10,
        timeout_timestamp: 1000000,
        data: vec![0, 1, 2].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr("not module"), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::Unauthorized { .. }) }))
}
