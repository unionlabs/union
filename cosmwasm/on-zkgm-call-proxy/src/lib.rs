use std::num::NonZeroU32;

use cosmwasm_std::{
    from_json, to_json_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, WasmMsg,
};
use depolama::{Bytes, Prefix, StorageExt, Store, ValueCodec};
use frissitheto::{UpgradeError, UpgradeMsg};
use serde::{Deserialize, Serialize};
use ucs03_zkgm::msg::OnZkgm;

enum Zkgm {}
impl Store for Zkgm {
    const PREFIX: Prefix = Prefix::new(b"zkgm");

    type Key = ();

    type Value = Addr;
}

impl ValueCodec<Addr> for Zkgm {
    fn encode_value(value: &Addr) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct InitMsg {
    pub zkgm: Addr,
}

fn init(deps: DepsMut, msg: InitMsg) -> Result<Response, Error> {
    deps.storage.write_item::<Zkgm>(&msg.zkgm);
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _: Env,
    _: MessageInfo,
    msg: InitMsg,
) -> Result<Response, Error> {
    frissitheto::init_state_version(&mut deps, const { NonZeroU32::new(1).unwrap() })
        .expect("infallible, instantiate can only be called once; qed;");
    init(deps, msg)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    OnZkgm(OnZkgm),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Msg {
    pub contract: Addr,
    pub msg: Binary,
    /// Funds that should be forwarded to the destination contract.
    ///
    /// Note that the forwarded call will fail if this contract does not have enough funds.
    ///
    /// # /!\ Security Note /!\
    ///
    /// Any funds sent (or allowances given) to this contract can be used by anyone. It is expected that this contract is first funded, and then called, in an atomic operation. This can be either a zkgm batch, funds sent directly to this contract in a `Msg` before this one in the same transaction, or any other way that ensures atomicity.
    pub funds: Vec<Coin>,
    pub call_action: CallAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CallAction {
    /// Thread the message through directly to the contract.
    Direct,
    /// Call the [`OnProxyOnZkgmCall`] entrypoint of the contract. This enables threading the OnZkgm information through, as well as the underlying message to be executed on the called contract, while still providing an indirection between the caller and the called contract.
    CallOnProxyCall,
}

/// Required interface for a contract supporting [`CallAction::CallOnProxyCall`] behaviour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum OnZkgmCallProxyMsg {
    OnProxyOnZkgmCall(OnProxyOnZkgmCall),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct OnProxyOnZkgmCall {
    // REVIEW: Should we thread through on_zkgm_message.message? That field is decoded to get msg
    pub on_zkgm_msg: OnZkgm,
    pub msg: Binary,
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    _: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, Error> {
    match msg {
        ExecuteMsg::OnZkgm(on_zkgm_msg) => {
            let Msg {
                contract,
                msg,
                funds,
                call_action,
            } = from_json::<Msg>(&*on_zkgm_msg.message)?;

            match call_action {
                CallAction::Direct => Ok(Response::new().add_message(WasmMsg::Execute {
                    contract_addr: contract.into(),
                    msg,
                    funds,
                })),
                CallAction::CallOnProxyCall => {
                    // we only need to check this here because the sensitive fields are ignored in the direct entrypoint
                    ensure_zkgm(deps.as_ref(), &info)?;

                    Ok(Response::new().add_message(WasmMsg::Execute {
                        contract_addr: contract.into(),
                        msg: to_json_binary(&OnZkgmCallProxyMsg::OnProxyOnZkgmCall(
                            OnProxyOnZkgmCall { on_zkgm_msg, msg },
                        ))?,
                        funds,
                    }))
                }
            }
        }
    }
}

fn ensure_zkgm(deps: Deps, info: &MessageInfo) -> Result<(), Error> {
    if deps.storage.read_item::<Zkgm>()? != info.sender {
        Err(Error::Unauthorized)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct MigrateMsg {}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(
    deps: DepsMut,
    _: Env,
    msg: UpgradeMsg<InitMsg, MigrateMsg>,
) -> Result<Response, Error> {
    msg.run(
        deps,
        |deps, msg| {
            let res = init(deps, msg)?;
            Ok((res, None))
        },
        |_, _, _| Ok((Response::default(), None)),
    )
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    StdError(#[from] StdError),

    #[error(transparent)]
    Migrate(#[from] UpgradeError),

    #[error("unauthorized")]
    Unauthorized,
}
