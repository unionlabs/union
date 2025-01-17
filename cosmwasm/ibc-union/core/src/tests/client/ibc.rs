use cosmwasm_std::{
    testing::{message_info, mock_dependencies, mock_env},
    to_json_binary, Addr, Event,
};
use ibc_union_msg::{
    lightclient::{
        QueryMsg as LightClientQueryMsg, VerifyClientMessageUpdate, VerifyCreationResponse,
    },
    msg::{ExecuteMsg, InitMsg, MsgUpdateClient},
};

use super::*;
use crate::{
    contract::{events, execute, instantiate},
    ContractError,
};

const CLIENT_TYPE: &str = "union";
const CLIENT_ADDRESS: &str = "unionclient";
const SENDER: &str = "unionsender";
const RELAYER: &str = "unionrelayer";

fn new_client_registered_event(client_type: &str, client_address: &Addr) -> Event {
    Event::new(events::client::REGISTER)
        .add_attribute(events::attribute::CLIENT_TYPE, client_type)
        .add_attribute(events::attribute::CLIENT_ADDRESS, client_address)
}

#[test]
fn register_client_ok() {
    let mut deps = mock_dependencies();
    let res = register_client(deps.as_mut()).unwrap();

    assert!(res
        .events
        .into_iter()
        .any(|e| e == new_client_registered_event(CLIENT_TYPE, &mock_addr(CLIENT_ADDRESS))));

    assert_eq!(
        crate::state::CLIENT_REGISTRY
            .load(&deps.storage, CLIENT_TYPE)
            .unwrap()
            .as_str(),
        mock_addr(CLIENT_ADDRESS).to_string()
    );
}

#[test]
fn register_client_fails_when_duplicate() {
    let mut deps = mock_dependencies();
    register_client(deps.as_mut()).unwrap();
    assert_eq!(
        register_client(deps.as_mut()),
        Err(ContractError::ClientTypeAlreadyExists)
    );
}

#[test]
fn create_client_ok() {
    let mut deps = mock_dependencies();
    let sender = mock_addr(SENDER);

    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&sender, &[]),
        InitMsg {},
    )
    .unwrap();
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                latest_height: 1,
                counterparty_chain_id: "testchain".to_owned(),
                events: None,
            }),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    assert!(create_client(deps.as_mut()).is_ok())
}

#[test]
fn create_client_commitments_saved() {
    let mut deps = mock_dependencies();
    let sender = mock_addr(SENDER);

    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&sender, &[]),
        InitMsg {},
    )
    .expect("instantiate ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                latest_height: 1,
                counterparty_chain_id: "testchain".to_owned(),
                events: None,
            }),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id: u32 = res
        .events
        .iter()
        .find(|event| event.ty.eq(events::client::CREATE))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq(events::attribute::CLIENT_ID))
        .expect("client type attribute exists")
        .value
        .parse()
        .expect("client type string is u32");

    assert_eq!(
        crate::state::CLIENT_TYPES
            .load(&deps.storage, client_id)
            .unwrap(),
        CLIENT_TYPE
    );
    assert_eq!(
        crate::state::CLIENT_IMPLS
            .load(&deps.storage, client_id)
            .unwrap()
            .as_str(),
        mock_addr(CLIENT_ADDRESS).to_string()
    );
    assert_eq!(
        crate::state::CLIENT_STATES
            .load(&deps.storage, client_id)
            .unwrap(),
        vec![1, 2, 3]
    );
    assert_eq!(
        crate::state::CLIENT_CONSENSUS_STATES
            .load(&deps.storage, (client_id, 1))
            .unwrap(),
        vec![1, 2, 3]
    );
}

#[test]
fn update_client_ok() {
    let mut deps = mock_dependencies();
    let sender = mock_addr(SENDER);

    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&sender, &[]),
        InitMsg {},
    )
    .expect("instantiate ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                latest_height: 1,
                counterparty_chain_id: "testchain".to_owned(),
                events: None,
            }),
            LightClientQueryMsg::VerifyClientMessage { .. } => {
                to_json_binary(&VerifyClientMessageUpdate {
                    height: 2,
                    consensus_state: vec![3, 2, 1].into(),
                    client_state: vec![3, 2, 1].into(),
                })
            }
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id: u32 = res
        .events
        .iter()
        .find(|event| event.ty.eq(events::client::CREATE))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq(events::attribute::CLIENT_ID))
        .expect("client type attribute exists")
        .value
        .parse()
        .expect("client type string is u32");

    let msg = ExecuteMsg::UpdateClient(MsgUpdateClient {
        client_id,
        client_message: vec![3, 2, 1].into(),
        relayer: mock_addr(RELAYER).into_string(),
    });
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        msg
    )
    .is_ok())
}

#[test]
fn update_client_ko() {
    let mut deps = mock_dependencies();
    let sender = mock_addr(SENDER);

    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&sender, &[]),
        InitMsg {},
    )
    .expect("instantiate ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                latest_height: 1,
                counterparty_chain_id: "testchain".to_owned(),
                events: None,
            }),
            LightClientQueryMsg::VerifyClientMessage { .. } => to_json_binary(&0),
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id: u32 = res
        .events
        .iter()
        .find(|event| event.ty.eq(events::client::CREATE))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq(events::attribute::CLIENT_ID))
        .expect("client type attribute exists")
        .value
        .parse()
        .expect("client type string is u32");

    let msg = ExecuteMsg::UpdateClient(MsgUpdateClient {
        client_id,
        client_message: vec![3, 2, 1].into(),
        relayer: mock_addr(RELAYER).into_string(),
    });
    assert!(execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        msg
    )
    .is_err())
}

#[test]
fn update_client_commitments_saved() {
    let mut deps = mock_dependencies();
    let sender = mock_addr(SENDER);

    instantiate(
        deps.as_mut(),
        mock_env(),
        message_info(&sender, &[]),
        InitMsg {},
    )
    .expect("instantiate ok");
    deps.querier
        .update_wasm(wasm_query_handler(|msg| match msg {
            LightClientQueryMsg::VerifyCreation { .. } => to_json_binary(&VerifyCreationResponse {
                latest_height: 1,
                counterparty_chain_id: "testchain".to_owned(),
                events: None,
            }),
            LightClientQueryMsg::VerifyClientMessage { .. } => {
                to_json_binary(&VerifyClientMessageUpdate {
                    height: 2,
                    consensus_state: vec![3, 2, 1].into(),
                    client_state: vec![3, 2, 1].into(),
                })
            }
            msg => panic!("should not be called: {:?}", msg),
        }));

    register_client(deps.as_mut()).expect("register client ok");
    let res = create_client(deps.as_mut()).expect("create client ok");
    let client_id: u32 = res
        .events
        .iter()
        .find(|event| event.ty.eq(events::client::CREATE))
        .expect("create client event exists")
        .attributes
        .iter()
        .find(|attribute| attribute.key.eq(events::attribute::CLIENT_ID))
        .expect("client type attribute exists")
        .value
        .parse()
        .expect("client type string is u32");

    let msg = ExecuteMsg::UpdateClient(MsgUpdateClient {
        client_id,
        client_message: vec![3, 2, 1].into(),
        relayer: mock_addr(RELAYER).into_string(),
    });
    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        msg,
    )
    .expect("update client ok");

    assert_eq!(
        crate::state::CLIENT_STATES
            .load(&deps.storage, client_id)
            .unwrap(),
        vec![3, 2, 1]
    );
    assert_eq!(
        crate::state::CLIENT_CONSENSUS_STATES
            .load(&deps.storage, (client_id, 2))
            .unwrap(),
        vec![3, 2, 1]
    );
}
