use cosmwasm_std::{
    from_json, testing::MockApi, Addr, Binary, QuerierResult, StdResult, WasmQuery,
};
use union_ibc_msg::lightclient::QueryMsg as LightClientQueryMsg;

mod cometbls;
mod ibc;

/// Creates a mock address from a given string. Prefixed with the default
/// `MockApi` prefix.
fn mock_addr(sender: &str) -> Addr {
    let mock_api = MockApi::default();
    mock_api.addr_make(sender)
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
