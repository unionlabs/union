use std::num::NonZeroU32;

use cosmwasm_std::{
    Addr, Deps, DepsMut, Empty, Env, Event, MessageInfo, Response, StdError, StdResult, WasmMsg,
    instantiate2_address, to_json_binary, wasm_execute,
};
use depolama::StorageExt;
use frissitheto::UpgradeMsg;
use hex_literal::hex;
use sha2::Digest;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InitMsg, MigrateMsg},
    state::{BytecodeBaseCodeId, CwAccountCodeId},
};

pub mod error;
pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;

const BYTECODE_BASE_CHECKSUM: &[u8] =
    &hex!("ec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1");

pub fn init(deps: DepsMut, msg: InitMsg) -> StdResult<Response> {
    deps.storage
        .write_item::<CwAccountCodeId>(&msg.cw_account_code_id);

    if deps
        .querier
        .query_wasm_code_info(msg.bytecode_base_code_id)?
        .checksum
        .as_slice()
        != BYTECODE_BASE_CHECKSUM
    {
        return Err(StdError::generic_err(
            "invalid bytecode base code id, checksum is not as expected",
        ));
    }

    deps.storage
        .write_item::<BytecodeBaseCodeId>(&msg.cw_account_code_id);

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InitMsg,
) -> Result<Response, ContractError> {
    frissitheto::init_state_version(&mut deps, const { NonZeroU32::new(1).unwrap() })?;

    Ok(init(deps, msg)?)
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CallProxy(msgs) => {
            let mut res = Response::new();

            let predicted_address = predict_call_proxy_account(deps.as_ref(), &env, &info.sender)?;

            if deps
                .querier
                .query_wasm_contract_info(&predicted_address)
                .is_err()
            {
                // proxy does not exist, add submsgs to create it

                res = res.add_messages([
                    WasmMsg::Instantiate2 {
                        admin: Some(env.contract.address.to_string()),
                        code_id: deps.storage.read_item::<BytecodeBaseCodeId>()?,
                        label: proxy_account_label(&env, &info.sender),
                        msg: to_json_binary(&Empty {})?,
                        funds: vec![],
                        salt: proxy_account_salt(deps.as_ref(), &info.sender)?.into(),
                    },
                    WasmMsg::Migrate {
                        contract_addr: predicted_address.to_string(),
                        new_code_id: deps.storage.read_item::<CwAccountCodeId>()?,
                        msg: to_json_binary(&UpgradeMsg::<_, ()>::Init(
                            cw_account::msg::InitMsg::Local {
                                // the proxy factory is the admin of all of the created proxies, and as such is the only actor that can call them
                                // this is safe since we check the proxy account address with info.sender
                                admin: env.contract.address.clone(),
                            },
                        ))?,
                    },
                    WasmMsg::UpdateAdmin {
                        contract_addr: predicted_address.to_string(),
                        admin: predicted_address.to_string(),
                    },
                ]);
            };

            Ok(res
                // forward the messages directly to the proxy account, along with any funds sent with this call
                .add_message(wasm_execute(
                    predicted_address,
                    &cw_account::msg::ExecuteMsg::Dispatch(msgs),
                    info.funds,
                )?)
                .add_event(Event::new("proxy_created").add_attribute("creator", info.sender)))
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, ContractError> {
    msg.run(
        deps,
        |deps, init_msg| {
            let res = init(deps, init_msg)?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

pub fn predict_call_proxy_account(
    deps: Deps,
    env: &Env,
    sender: &Addr,
) -> Result<Addr, ContractError> {
    let token_addr = instantiate2_address(
        BYTECODE_BASE_CHECKSUM,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        &proxy_account_salt(deps, sender)?,
    )
    .expect("both checksum and salt are valid lengths; qed;");

    Ok(deps.api.addr_humanize(&token_addr)?)
}

fn proxy_account_salt(deps: Deps<'_>, sender: &Addr) -> Result<[u8; 32], ContractError> {
    Ok(sha2::Sha256::new()
        .chain_update(deps.api.addr_canonicalize(sender.as_str())?.as_slice())
        .finalize()
        .into())
}

pub fn proxy_account_label(env: &Env, sender: &Addr) -> String {
    format!("{}/proxy/{sender}", env.contract.address)
}
