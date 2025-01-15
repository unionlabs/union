use contract::instantiate;
use cosmwasm_std::{testing::mock_dependencies, to_json_binary};
use ibc_solidity::Packet;
use ibc_union_spec::COMMITMENT_MAGIC;
use union_ibc_msg::msg::{
    InitMsg, MsgBatchAcks, MsgBatchSend, MsgIntentPacketRecv, MsgPacketAcknowledgement,
    MsgPacketRecv, MsgPacketTimeout, MsgSendPacket, MsgWriteAcknowledgement,
};

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

#[test]
fn recv_packet_ok() {
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

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 2,
            destination_channel: 1,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .is_ok())
}

#[test]
fn recv_packet_invalid_channel_state() {
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

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 2,
            destination_channel: 5,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .is_err_and(|err| {
        match err {
            ContractError::Std(err) => {
                matches!(err, StdError::NotFound { .. })
            }
            _ => false,
        }
    }))
}

#[test]
fn recv_packet_timeout_timestamp() {
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

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 2,
            destination_channel: 1,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 100,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::ReceivedTimedOutPacketTimestamp { .. }) }))
}

#[test]
fn recv_packet_timeout_height() {
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

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 2,
            destination_channel: 1,
            data: vec![1, 2, 3].into(),
            timeout_height: 1,
            timeout_timestamp: 2000000000000000000,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::ReceivedTimedOutPacketHeight { .. }) }))
}

#[test]
fn recv_intent_packet_ok() {
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

    let msg = MsgIntentPacketRecv {
        packets: vec![Packet {
            source_channel: 2,
            destination_channel: 1,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        market_maker_msgs: vec![vec![1, 2, 3].into()],
        market_maker: mock_addr("marketmaker").into_string(),
        empty_proof: vec![].into(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::IntentPacketRecv(msg)
    )
    .is_ok())
}

#[test]
fn recv_intent_packet_timeout_timestamp() {
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

    let msg = MsgIntentPacketRecv {
        packets: vec![Packet {
            source_channel: 2,
            destination_channel: 1,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 100,
        }],
        market_maker_msgs: vec![vec![1, 2, 3].into()],
        market_maker: mock_addr("marketmaker").into_string(),
        empty_proof: vec![].into(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::IntentPacketRecv(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::ReceivedTimedOutPacketTimestamp { .. }) }))
}

#[test]
fn recv_intent_packet_timeout_height() {
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

    let msg = MsgIntentPacketRecv {
        packets: vec![Packet {
            source_channel: 2,
            destination_channel: 1,
            data: vec![1, 2, 3].into(),
            timeout_height: 1,
            timeout_timestamp: 2000000000000000000,
        }],
        market_maker_msgs: vec![vec![1, 2, 3].into()],
        market_maker: mock_addr("marketmaker").into_string(),
        empty_proof: vec![].into(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::IntentPacketRecv(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::ReceivedTimedOutPacketHeight { .. }) }))
}

#[test]
fn acknowledge_packet_ok() {
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel: 1,
        timeout_height: 100000,
        timeout_timestamp: 2000000000000000000,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketAcknowledgement {
        packets: vec![Packet {
            source_channel: 1,
            destination_channel: 0,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        acknowledgements: vec![vec![1, 2, 3].into()],
        proof: vec![1].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketAck(msg)
    )
    .is_ok())
}

#[test]
fn acknowledge_packet_tampered() {
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel: 1,
        timeout_height: 100000,
        timeout_timestamp: 2000000000000000000,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketAcknowledgement {
        packets: vec![Packet {
            source_channel: 1,
            destination_channel: 0,
            data: vec![4, 1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        acknowledgements: vec![vec![1, 2, 3].into()],
        proof: vec![1].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketAck(msg)
    )
    .is_err_and(|err| matches!(err, ContractError::PacketCommitmentNotFound)))
}

#[test]
fn acknowledge_packet_not_sent() {
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketAcknowledgement {
        packets: vec![Packet {
            source_channel: 1,
            destination_channel: 0,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        acknowledgements: vec![vec![1, 2, 3].into()],
        proof: vec![1].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketAck(msg)
    )
    .is_err_and(|err| matches!(err, ContractError::PacketCommitmentNotFound)))
}

#[test]
fn timeout_packet_timestamp_ok() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
        timeout_timestamp: 100,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketTimeout {
        packet: Packet {
            source_channel: 1,
            destination_channel: 0,
            data: vec![1, 2, 3].into(),
            timeout_height: 10,
            timeout_timestamp: 100,
        },
        proof: vec![1].into(),
        proof_height: 11,
        relayer: mock_addr(RELAYER).into_string(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketTimeout(msg)
    )
    .is_ok())
}

#[test]
fn timeout_packet_timestamp_timestamp_not_reached() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
        timeout_timestamp: 200000,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketTimeout {
        packet: Packet {
            source_channel: 1,
            destination_channel: 0,
            data: vec![1, 2, 3].into(),
            timeout_height: 10,
            timeout_timestamp: 200000,
        },
        proof: vec![1].into(),
        proof_height: 11,
        relayer: mock_addr(RELAYER).into_string(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketTimeout(msg)
    )
    .is_err_and(|err| { matches!(err, ContractError::TimeoutTimestampNotReached) }))
}

#[test]
fn timeout_packet_height_ok() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
        timeout_timestamp: 0,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketTimeout {
        packet: Packet {
            source_channel: 1,
            destination_channel: 0,
            data: vec![1, 2, 3].into(),
            timeout_height: 10,
            timeout_timestamp: 0,
        },
        proof: vec![1].into(),
        proof_height: 11,
        relayer: mock_addr(RELAYER).into_string(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketTimeout(msg)
    )
    .is_ok())
}

#[test]
fn timeout_packet_height_not_reached() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
        timeout_timestamp: 0,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet ok");

    let msg = MsgPacketTimeout {
        packet: Packet {
            source_channel: 1,
            destination_channel: 0,
            data: vec![1, 2, 3].into(),
            timeout_height: 10,
            timeout_timestamp: 0,
        },
        proof: vec![1].into(),
        proof_height: 9,
        relayer: mock_addr(RELAYER).into_string(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketTimeout(msg)
    )
    .is_err_and(|err| { matches!(err, ContractError::TimeoutHeightNotReached) }))
}

#[test]
fn write_acknowledgement_ok() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 1,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");

    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 1,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_ok())
}

#[test]
fn write_acknowledgement_module_is_not_channel_owner() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr("malicious").to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr("malicious"), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr("malicious"), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 1,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");

    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 1,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::Unauthorized { .. }) }))
}

#[test]
fn write_acknowledgement_packet_not_received() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 1,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::PacketNotReceived) }))
}

#[test]
fn write_acknowledgement_already_exists() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 1,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");

    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 1,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_ok());
    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 1,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 2000000000000000000,
        },
        acknowledgement: vec![1].into(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .is_err_and(|err| { matches!(err, ContractError::AlreadyAcknowledged) }))
}

#[test]
fn batch_send_ok() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgSendPacket {
        source_channel: 2,
        timeout_height: 10,
        timeout_timestamp: 0,
        data: vec![1, 2, 3].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet is ok");
    let msg = MsgSendPacket {
        source_channel: 2,
        timeout_height: 10,
        timeout_timestamp: 0,
        data: vec![4, 5, 6].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketSend(msg),
    )
    .expect("send packet is ok");

    let msg = MsgBatchSend {
        source_channel: 2,
        packets: vec![
            Packet {
                source_channel: 2,
                destination_channel: 0,
                data: vec![4, 5, 6].into(),
                timeout_height: 10,
                timeout_timestamp: 0,
            },
            Packet {
                source_channel: 2,
                destination_channel: 0,
                data: vec![1, 2, 3].into(),
                timeout_height: 10,
                timeout_timestamp: 0,
            },
        ],
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchSend(msg)
    )
    .is_ok())
}

#[test]
fn batch_send_packet_not_sent() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgBatchSend {
        source_channel: 2,
        packets: vec![
            Packet {
                source_channel: 2,
                destination_channel: 0,
                data: vec![4, 5, 6].into(),
                timeout_height: 10,
                timeout_timestamp: 0,
            },
            Packet {
                source_channel: 2,
                destination_channel: 0,
                data: vec![1, 2, 3].into(),
                timeout_height: 10,
                timeout_timestamp: 0,
            },
        ],
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchSend(msg)
    )
    .is_err_and(|err| { matches!(err, ContractError::PacketCommitmentNotFound) }))
}

#[test]
fn batch_acks_ok() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 0,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 0,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");
    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 0,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 0,
        },
        acknowledgement: vec![1].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .expect("write ack is ok");
    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 0,
            destination_channel: 2,
            data: vec![3, 4, 5].into(),
            timeout_height: 100000,
            timeout_timestamp: 0,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");
    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 0,
            destination_channel: 2,
            data: vec![3, 4, 5].into(),
            timeout_height: 100000,
            timeout_timestamp: 0,
        },
        acknowledgement: vec![1].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .expect("write ack is ok");

    let msg = MsgBatchAcks {
        source_channel: 2,
        packets: vec![
            Packet {
                source_channel: 0,
                destination_channel: 2,
                data: vec![1, 2, 3].into(),
                timeout_height: 100000,
                timeout_timestamp: 0,
            },
            Packet {
                source_channel: 0,
                destination_channel: 2,
                data: vec![3, 4, 5].into(),
                timeout_height: 100000,
                timeout_timestamp: 0,
            },
        ],
        acks: vec![vec![1].into(), vec![1].into()],
    };
    assert!(dbg!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchAcks(msg)
    ))
    .is_ok())
}

#[test]
fn batch_acks_packet_not_received() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgBatchAcks {
        source_channel: 2,
        packets: vec![
            Packet {
                source_channel: 0,
                destination_channel: 2,
                data: vec![1, 2, 3].into(),
                timeout_height: 100000,
                timeout_timestamp: 0,
            },
            Packet {
                source_channel: 0,
                destination_channel: 2,
                data: vec![3, 4, 5].into(),
                timeout_height: 100000,
                timeout_timestamp: 0,
            },
        ],
        acks: vec![vec![1].into(), vec![1].into()],
    };
    assert!(dbg!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchAcks(msg)
    ))
    .is_err_and(|err| { matches!(err, ContractError::PacketCommitmentNotFound) }))
}

#[test]
fn batch_acks_tampered_packet() {
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
            LightClientQueryMsg::VerifyNonMembership { .. } => to_json_binary(&()),
            LightClientQueryMsg::GetTimestamp { .. } => to_json_binary(&100000),
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
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: 1,
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is okay");
    channel_open_ack(deps.as_mut()).expect("channel open ack is ok");
    let msg = MsgChannelOpenAck {
        channel_id: 2,
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: 0,
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg),
    )
    .expect("channel open ack is ok");

    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 0,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 0,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");
    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 0,
            destination_channel: 2,
            data: vec![1, 2, 3].into(),
            timeout_height: 100000,
            timeout_timestamp: 0,
        },
        acknowledgement: vec![1].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .expect("write ack is ok");
    let msg = MsgPacketRecv {
        packets: vec![Packet {
            source_channel: 0,
            destination_channel: 2,
            data: vec![3, 4, 5].into(),
            timeout_height: 100000,
            timeout_timestamp: 0,
        }],
        relayer_msgs: vec![vec![1].into()],
        relayer: mock_addr(RELAYER).to_string(),
        proof: vec![1, 2, 3].into(),
        proof_height: 1,
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::PacketRecv(msg),
    )
    .expect("recv packet ok");
    let msg = MsgWriteAcknowledgement {
        channel_id: 2,
        packet: Packet {
            source_channel: 0,
            destination_channel: 2,
            data: vec![3, 4, 5].into(),
            timeout_height: 100000,
            timeout_timestamp: 0,
        },
        acknowledgement: vec![1].into(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::WriteAcknowledgement(msg),
    )
    .expect("write ack is ok");

    let msg = MsgBatchAcks {
        source_channel: 2,
        packets: vec![
            Packet {
                source_channel: 0,
                destination_channel: 2,
                data: vec![10, 20, 30].into(),
                timeout_height: 100000,
                timeout_timestamp: 0,
            },
            Packet {
                source_channel: 0,
                destination_channel: 2,
                data: vec![30, 40, 50].into(),
                timeout_height: 100000,
                timeout_timestamp: 0,
            },
        ],
        acks: vec![vec![1].into(), vec![1].into()],
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::BatchAcks(msg)
    )
    .is_err_and(|err| { matches!(err, ContractError::PacketCommitmentNotFound) }))
}
