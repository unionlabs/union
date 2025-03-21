use cosmwasm_std::{testing::mock_dependencies, to_json_binary};
use depolama::StorageExt;
use ibc_union_msg::{
    lightclient::VerifyCreationResponse,
    msg::{
        InitMsg, MsgChannelOpenAck, MsgChannelOpenConfirm, MsgChannelOpenInit, MsgChannelOpenTry,
    },
};
use ibc_union_spec::Channel;

use super::*;
use crate::{
    contract::init,
    state::{ChannelOwner, Channels},
};

const SENDER: &str = "unionsender";
const RELAYER: &str = "unionrelayer";
const VERSION: &str = "version";

#[test]
fn channel_open_init_ok() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .is_ok());
}

#[test]
fn channel_open_init_channel_claimed() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    channel_open_init(deps.as_mut()).expect("channel open init is ok");

    assert_eq!(
        deps.storage.read::<ChannelOwner>(&ChannelId!(1)).unwrap(),
        mock_addr(SENDER)
    );
}

#[test]
fn channel_open_init_commitment_saved() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");
    channel_open_init(deps.as_mut()).expect("channel open init is ok");

    assert_eq!(
        deps.storage.read::<Channels>(&ChannelId!(1)).unwrap(),
        Channel {
            state: ChannelState::Init,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: None,
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned()
        }
    );
}

#[test]
fn channel_open_try_ok() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenTry {
        port_id: mock_addr(SENDER).into_string(),
        channel: Channel {
            state: ChannelState::TryOpen,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: Some(ChannelId!(1)),
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned(),
        },
        counterparty_version: VERSION.to_owned(),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenTry(msg),
    )
    .is_ok())
}
#[test]
fn channel_open_try_invalid_state() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenTry {
        port_id: mock_addr(SENDER).into_string(),
        channel: Channel {
            state: ChannelState::Open,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: None,
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned(),
        },
        counterparty_version: VERSION.to_owned(),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenTry(msg),
    )
    .is_err_and(|err| {
        matches!(
            err,
            ContractError::ChannelInvalidState {
                got: ChannelState::Open,
                expected: ChannelState::TryOpen
            }
        )
    }))
}

#[test]
fn channel_open_try_channel_claimed() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenTry {
        port_id: mock_addr(SENDER).into_string(),
        channel: Channel {
            state: ChannelState::TryOpen,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: Some(ChannelId!(1)),
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned(),
        },
        counterparty_version: VERSION.to_owned(),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenTry(msg),
    )
    .expect("channel open try is ok");

    assert_eq!(
        deps.storage.read::<ChannelOwner>(&ChannelId!(1)).unwrap(),
        mock_addr(SENDER)
    );
}

#[test]
fn channel_open_try_commitment_saved() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenTry {
        port_id: mock_addr(SENDER).into_string(),
        channel: Channel {
            state: ChannelState::TryOpen,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: Some(ChannelId!(1)),
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned(),
        },
        counterparty_version: VERSION.to_owned(),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenTry(msg),
    )
    .expect("channel open try is ok");

    assert_eq!(
        deps.storage.read::<Channels>(&ChannelId!(1)).unwrap(),
        Channel {
            state: ChannelState::TryOpen,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: Some(ChannelId!(1)),
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned(),
        }
    );
}

#[test]
fn channel_open_ack_ok() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is ok");

    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(1),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenAck(msg)
    )
    .is_ok())
}

#[test]
fn channel_open_ack_not_found() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(1),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&mock_addr(SENDER), &[]),
            ExecuteMsg::ChannelOpenAck(msg)
        ),
        Err(ContractError::Std(StdError::generic_err(
            "key 0x00000001 not present"
        )))
    )
}

#[test]
fn channel_open_ack_commitment_saved() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenInit(msg),
    )
    .expect("channel open init is ok");

    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(1),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
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

    assert_eq!(
        deps.storage.read::<Channels>(&ChannelId!(1)).unwrap(),
        Channel {
            state: ChannelState::Open,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: Some(ChannelId!(1)),
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned()
        }
    );
}

#[test]
fn channel_open_confirm_ok() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenTry {
        port_id: mock_addr(SENDER).into_string(),
        channel: Channel {
            state: ChannelState::TryOpen,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: Some(ChannelId!(1)),
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned(),
        },
        counterparty_version: VERSION.to_owned(),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenTry(msg),
    )
    .expect("channel open try is ok");

    let msg = MsgChannelOpenConfirm {
        channel_id: ChannelId!(1),
        proof_ack: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenConfirm(msg),
    )
    .is_ok())
}

#[test]
fn channel_open_try_invalid_counterparty() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenTry {
        port_id: mock_addr(SENDER).into_string(),
        channel: Channel {
            state: ChannelState::TryOpen,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: None,
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned(),
        },
        counterparty_version: VERSION.to_owned(),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&mock_addr(SENDER), &[]),
            ExecuteMsg::ChannelOpenTry(msg),
        ),
        Err(ContractError::CounterpartyChannelIdInvalid)
    )
}

#[test]
fn channel_open_confirm_not_found() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenConfirm {
        channel_id: ChannelId!(1),
        proof_ack: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&mock_addr(SENDER), &[]),
            ExecuteMsg::ChannelOpenConfirm(msg),
        ),
        Err(ContractError::Std(StdError::generic_err(
            "key 0x00000001 not present"
        )))
    )
}

#[test]
fn channel_open_confirm_commitment_saved() {
    let mut deps = mock_dependencies();
    init(deps.as_mut(), InitMsg {}).unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                counterparty_chain_id: "testchain".to_owned(),
                client_state_bytes: None,
                events: vec![],
                storage_writes: Default::default(),
            }),
            LightClientQueryMsg::VerifyMembership { .. } => to_json_binary(&()),
            msg => panic!("should not be called: {:?}", msg),
        }));
    register_client(deps.as_mut()).expect("register client ok");
    create_client(deps.as_mut()).expect("create client ok");

    connection_open_try(deps.as_mut()).expect("connection open try is ok");
    connection_open_confirm(deps.as_mut()).expect("connection open confirm is ok");

    let msg = MsgChannelOpenTry {
        port_id: mock_addr(SENDER).into_string(),
        channel: Channel {
            state: ChannelState::TryOpen,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: Some(ChannelId!(1)),
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned(),
        },
        counterparty_version: VERSION.to_owned(),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).into_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenTry(msg),
    )
    .expect("channel open try is ok");

    let msg = MsgChannelOpenConfirm {
        channel_id: ChannelId!(1),
        proof_ack: vec![1, 2, 3].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        ExecuteMsg::ChannelOpenConfirm(msg),
    )
    .expect("channel open confirm is ok");

    assert_eq!(
        deps.storage.read::<Channels>(&ChannelId!(1)).unwrap(),
        Channel {
            state: ChannelState::Open,
            connection_id: ConnectionId!(1),
            counterparty_channel_id: Some(ChannelId!(1)),
            counterparty_port_id: vec![1].into(),
            version: VERSION.to_owned()
        }
    );
}
