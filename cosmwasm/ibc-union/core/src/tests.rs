use contract::execute;
use cosmwasm_std::{
    from_json,
    testing::{message_info, mock_env, MockApi},
    Addr, Binary, DepsMut, QuerierResult, Response, StdResult, WasmQuery,
};
use union_ibc_msg::{
    lightclient::QueryMsg as LightClientQueryMsg,
    msg::{ExecuteMsg, MsgCreateClient, MsgRegisterClient},
};

use super::*;

mod client;
mod connection;

/// Creates a mock address from a given string.
/// Addresses are prefixed with the default [`MockApi`] prefix.
fn mock_addr(sender: impl Into<String>) -> Addr {
    let mock_api = MockApi::default();
    mock_api.addr_make(&Into::<String>::into(sender))
}

fn wasm_query_handler<F: Fn(LightClientQueryMsg) -> StdResult<Binary> + 'static>(
    querier: F,
) -> impl Fn(&WasmQuery) -> QuerierResult + 'static {
    move |msg| match msg {
        WasmQuery::Smart { msg, .. } => {
            let msg: LightClientQueryMsg = from_json(msg).unwrap();
            let res = querier(msg).unwrap();
            QuerierResult::Ok(cosmwasm_std::ContractResult::Ok(res))
        }
        _ => panic!("Only smart queries should be possible now. Adjust this based on your needs."),
    }
}

/// Creates a mock client.
/// Uses [`mock_addr`] to convert address seeds to addresses
/// Addresses are prefixed with the default [`MockApi`] prefix.
fn register_client(
    deps: DepsMut,
    client_type: impl Into<String>,
    client_address_seed: impl Into<String>,
    sender_address_seed: impl Into<String>,
) -> Result<Response, ContractError> {
    let register_msg = ExecuteMsg::RegisterClient(MsgRegisterClient {
        client_type: client_type.into(),
        client_address: mock_addr(client_address_seed).into_string(),
    });

    let sender = mock_addr(sender_address_seed);
    execute(deps, mock_env(), message_info(&sender, &[]), register_msg)
}

fn create_client(
    deps: DepsMut,
    client_type: impl Into<String> + Clone,
    sender_address_seed: impl Into<String> + Clone,
    relayer_address_seed: impl Into<String> + Clone,
) -> Result<Response, ContractError> {
    let execute_msg = ExecuteMsg::CreateClient(MsgCreateClient {
        client_type: client_type.into(),
        client_state_bytes: vec![1, 2, 3].into(),
        consensus_state_bytes: vec![1, 2, 3].into(),
        relayer: mock_addr(relayer_address_seed).into_string(),
    });

    let sender = mock_addr(sender_address_seed);
    execute(deps, mock_env(), message_info(&sender, &[]), execute_msg)
}

#[test]
fn display() {
    assert_eq!(
        ContractErrorKind::ArithmeticOverflow,
        ContractErrorKind::parse_from_error_message(&ContractError::ArithmeticOverflow.to_string())
            .unwrap()
    )
}
