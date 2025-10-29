use access_managed::Restricted;
use access_manager_types::CanCall;
use contract::execute;
use cosmwasm_std::{
    Addr, Binary, DepsMut, QuerierResult, Response, StdResult, WasmQuery, from_json,
    testing::{MockApi, message_info, mock_env},
    to_json_binary,
};
use ibc_union_msg::{
    lightclient::QueryMsg as LightClientQueryMsg,
    msg::{
        ExecuteMsg, MsgChannelOpenAck, MsgChannelOpenInit, MsgConnectionOpenConfirm,
        MsgConnectionOpenInit, MsgConnectionOpenTry, MsgCreateClient, MsgRegisterClient,
    },
};
use ibc_union_spec::{ClientId, ConnectionId};

use super::*;

mod channel;
mod client;
mod connection;

const CLIENT_TYPE: &str = "union";
const CLIENT_ADDRESS: &str = "unionclient";
const SENDER: &str = "unionsender";
const MANAGER: &str = "manager";
const RELAYER: &str = "unionrelayer";
const VERSION: &str = "version";

/// Creates a mock address from a given string.
/// Addresses are prefixed with the default [`MockApi`] prefix.
fn mock_addr(address_seed: impl Into<String>) -> Addr {
    let mock_api = MockApi::default();
    mock_api.addr_make(&Into::<String>::into(address_seed))
}

fn wasm_query_handler<F: Fn(LightClientQueryMsg) -> StdResult<Binary> + 'static>(
    querier: F,
) -> impl Fn(&WasmQuery) -> QuerierResult + 'static {
    move |msg| match msg {
        WasmQuery::Smart { msg, contract_addr } => {
            if contract_addr == mock_addr(MANAGER).as_str() {
                let msg = from_json::<access_manager_types::manager::msg::QueryMsg>(msg).unwrap();

                match msg {
                    access_manager_types::manager::msg::QueryMsg::CanCall { .. } => {
                        QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                            to_json_binary(&CanCall {
                                allowed: true,
                                delay: 0,
                            })
                            .unwrap(),
                        ))
                    }
                    _ => unimplemented!(),
                }
            } else {
                // we assume all other queries will be into the ibc contract

                QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(
                    querier(from_json::<LightClientQueryMsg>(msg).unwrap()).unwrap(),
                ))
            }
        }
        _ => panic!("Only smart queries should be possible now. Adjust this based on your needs."),
    }
}

/// Creates a mock client.
/// Uses [`mock_addr`] to convert address seeds to addresses
/// Addresses are prefixed with the default [`MockApi`] prefix.
fn register_client(deps: DepsMut) -> Result<Response, ContractError> {
    let register_msg = Restricted::wrap(ExecuteMsg::RegisterClient(MsgRegisterClient {
        client_type: CLIENT_TYPE.to_owned(),
        client_address: mock_addr(CLIENT_ADDRESS).into_string(),
    }));

    let sender = mock_addr(SENDER);
    execute(deps, mock_env(), message_info(&sender, &[]), register_msg)
}

fn create_client(deps: DepsMut) -> Result<Response, ContractError> {
    let execute_msg = Restricted::wrap(ExecuteMsg::CreateClient(MsgCreateClient {
        client_type: CLIENT_TYPE.to_owned(),
        client_state_bytes: vec![1, 2, 3].into(),
        consensus_state_bytes: vec![1, 2, 3].into(),
        relayer: mock_addr(RELAYER).into_string(),
    }));

    let sender = mock_addr(SENDER);
    execute(deps, mock_env(), message_info(&sender, &[]), execute_msg)
}

fn connection_open_init(deps: DepsMut) -> Result<Response, ContractError> {
    let msg = MsgConnectionOpenInit {
        client_id: ClientId!(1),
        counterparty_client_id: ClientId!(2),
    };
    execute(
        deps,
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        Restricted::wrap(ExecuteMsg::ConnectionOpenInit(msg)),
    )
}

fn connection_open_try(deps: DepsMut) -> Result<Response, ContractError> {
    let msg = MsgConnectionOpenTry {
        counterparty_client_id: ClientId!(2),
        counterparty_connection_id: ConnectionId!(1),
        client_id: ClientId!(1),
        proof_init: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    execute(
        deps,
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        Restricted::wrap(ExecuteMsg::ConnectionOpenTry(msg)),
    )
}

fn connection_open_confirm(deps: DepsMut) -> Result<Response, ContractError> {
    let msg = MsgConnectionOpenConfirm {
        connection_id: ConnectionId!(1),
        proof_ack: vec![1, 2, 3].into(),
        proof_height: 1,
    };

    execute(
        deps,
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        Restricted::wrap(ExecuteMsg::ConnectionOpenConfirm(msg)),
    )
}

fn channel_open_init(deps: DepsMut) -> Result<Response, ContractError> {
    let msg = MsgChannelOpenInit {
        port_id: mock_addr(SENDER).to_string(),
        counterparty_port_id: vec![1].into(),
        connection_id: ConnectionId!(1),
        version: VERSION.to_owned(),
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps,
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        Restricted::wrap(ExecuteMsg::ChannelOpenInit(msg)),
    )
}

fn channel_open_ack(deps: DepsMut) -> Result<Response, ContractError> {
    let msg = MsgChannelOpenAck {
        channel_id: ChannelId!(1),
        counterparty_version: VERSION.to_owned(),
        counterparty_channel_id: ChannelId!(1),
        proof_try: vec![1].into(),
        proof_height: 1,
        relayer: mock_addr(RELAYER).to_string(),
    };
    execute(
        deps,
        mock_env(),
        message_info(&mock_addr(SENDER), &[]),
        Restricted::wrap(ExecuteMsg::ChannelOpenAck(msg)),
    )
}

#[test]
fn display() {
    assert_eq!(
        ContractErrorKind::ArithmeticOverflow,
        ContractErrorKind::parse(
            ContractError::ArithmeticOverflow
                .to_string()
                .split(' ')
                .next()
                .unwrap()
        )
        .unwrap()
    )
}
