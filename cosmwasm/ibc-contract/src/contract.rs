#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, StdResult};
use cosmwasm_std::{
    to_json_binary, Deps, DepsMut, Env, Event, MessageInfo, Reply, Response, SubMsg, SubMsgResult,
    WasmMsg,
};
use ibc_vm_rs::{IbcEvent, IbcMsg, IbcResponse, Status};

use crate::{
    msg::{ExecuteMsg, IbcInitMsg, InitMsg},
    state::{CLIENTS, CLIENT_INDEX, CODE_IDS, COMMITMENTS},
    ContractError,
};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

pub struct CwIbcHost<'a> {
    deps: DepsMut<'a>,
    env: Env,
}

impl<'a> ibc_vm_rs::IbcHost for CwIbcHost<'a> {
    fn next_client_identifier(&mut self, client_type: &String) -> String {
        let index = CLIENT_INDEX
            .update(self.deps.storage, |mut i| -> StdResult<_> {
                i += 1;
                Ok(i)
            })
            .unwrap();

        format!("{client_type}-{index}")
    }

    fn commit(&mut self, key: String, value: Vec<u8>) {
        COMMITMENTS.save(self.deps.storage, key, &value).unwrap();
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let response = match msg {
        ExecuteMsg::RegisterClient {
            code_id,
            client_type,
        } => {
            if CODE_IDS.has(deps.storage, client_type.clone()) {
                return Err(ContractError::AlreadyRegistered);
            }
            CODE_IDS.save(deps.storage, client_type, &code_id)?;
            Response::default()
        }
        ExecuteMsg::CreateClient {
            client_state,
            consensus_state,
            client_type,
        } => {
            if !CODE_IDS.has(deps.storage, client_type.clone()) {
                return Err(ContractError::ClientTypeNotRegistered);
            }

            let mut host = CwIbcHost { deps, env };
            let runnable =
                ibc_vm_rs::create_client(client_type, client_state.into(), consensus_state.into());

            fold(&mut host, runnable, IbcResponse::Empty)?
        }
    };
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id {
        // init submessage
        1 => {
            // TODO(aeryz): We can't guarantee the type here, `CreateClient` should be in a top-level enum and should be decoded
            // automatically
            let runnable: ibc_vm_rs::CreateClient = cosmwasm_std::from_json(msg.payload).unwrap();

            let ibc_vm_rs::CreateClient::Initialize { client_id, .. } = runnable.clone() else {
                panic!("no please");
            };

            let SubMsgResult::Ok(resp) = msg.result else {
                return Err(ContractError::EthAbiDecoding);
            };

            let contract_address = resp
                .events
                .into_iter()
                .find(|event| event.ty == "wasm")
                .and_then(|event| {
                    event
                        .attributes
                        .into_iter()
                        .find(|e| e.key == "_contract_address")
                        .map(|e| e.value)
                })
                .unwrap();

            let address = deps.api.addr_validate(&contract_address)?;

            CLIENTS.save(deps.storage, client_id, &address).unwrap();

            let mut host = CwIbcHost { deps, env };
            fold(&mut host, runnable, IbcResponse::Initialize)?;
        }
        _ => return Err(ContractError::UnknownReplyId),
    }

    Ok(Response::new())
}

// TODO(aeryz): i hate naming lol
pub fn fold<'a, T: ibc_vm_rs::Runnable<CwIbcHost<'a>>>(
    host: &mut CwIbcHost<'a>,
    mut runnable: T,
    mut response: IbcResponse,
) -> Result<Response, ContractError> {
    let mut ibc_msg;
    loop {
        let either = runnable.process(host, response).unwrap();

        (runnable, ibc_msg) = match either {
            ibc_vm_rs::Either::Left(left) => left,
            ibc_vm_rs::Either::Right(event) => {
                return Ok(Response::new().add_event(ibc_event_to_cw(event)));
            }
        };

        match ibc_msg {
            IbcMsg::Initialize {
                client_id,
                client_type,
                client_state,
                consensus_state,
            } => {
                let Some(code_id) = CODE_IDS
                    .may_load(host.deps.storage, client_type.clone())
                    .unwrap()
                else {
                    return Err(ContractError::ClientTypeNotRegistered);
                };
                let submsg = SubMsg::reply_on_success(
                    WasmMsg::Instantiate {
                        admin: Some(host.env.contract.address.to_string()),
                        code_id,
                        msg: to_json_binary(&IbcInitMsg {
                            client_id,
                            client_state: client_state.into(),
                            consensus_state: consensus_state.into(),
                        })?,
                        funds: Vec::new(),
                        label: "Client".into(),
                    },
                    1,
                )
                .with_payload(cosmwasm_std::to_json_vec(&runnable).unwrap());
                return Ok(Response::new().add_submessage(submsg));
            }
            IbcMsg::Status { client_id } => {
                let status = query_status(host.deps.as_ref(), client_id)?;
                response = IbcResponse::Status { status };
            }
            IbcMsg::LatestHeight { client_id } => {
                let height = query_latest_height(host.deps.as_ref(), client_id)?;
                response = IbcResponse::LatestHeight { height };
            }
        }
    }
}

pub fn ibc_event_to_cw(_ibc_event: IbcEvent) -> Event {
    todo!()
}

pub fn query_status(_deps: Deps, _client_id: String) -> StdResult<Status> {
    todo!()
}

pub fn query_latest_height(_deps: Deps, _client_id: String) -> StdResult<u64> {
    todo!()
}
