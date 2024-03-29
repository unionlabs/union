use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, IbcBasicResponse, IbcChannel,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcMsg,
    IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse,
    MessageInfo, Never, Reply, Response, StdError, StdResult,
};
use cw_storage_plus::Map;
use ethabi::{ParamType, Token};
use ics721::{
    execute::Ics721Execute, ibc::Ics721Ibc, query::Ics721Query, NonFungibleTokenPacketData,
};
use thiserror::Error;

pub const UCS_NFT_IBC_VERSION: &str = "ucs02-1";

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Ics721(#[from] ics721::ContractError),
    #[error(transparent)]
    Std(#[from] StdError),
    #[error("Unable to decode eth abi encoded nft")]
    EthAbiDecoding,
    #[error("Tried to open a channel between two different protocols: {src:?} <-> {dst:?}")]
    IbcVersionMismatch { src: Version, dst: Version },
}

#[cw_serde]
pub enum Version {
    ICS721,
    UCS02,
}

impl TryFrom<&str> for Version {
    type Error = ics721::ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value == ics721::ibc::IBC_VERSION {
            Ok(Version::ICS721)
        } else if value == UCS_NFT_IBC_VERSION {
            Ok(Version::UCS02)
        } else {
            Err(ics721::ContractError::InvalidVersion {
                actual: value.to_string(),
                expected: format!("{} or {}", ics721::ibc::IBC_VERSION, UCS_NFT_IBC_VERSION),
            })
        }
    }
}

pub const CHANNEL_VERSION: Map<&str, Version> = Map::new("channel_version");

#[repr(transparent)]
pub struct UCS02NonFungibleTokenPacketData(NonFungibleTokenPacketData);

impl From<NonFungibleTokenPacketData> for UCS02NonFungibleTokenPacketData {
    fn from(value: NonFungibleTokenPacketData) -> Self {
        Self(value)
    }
}

impl UCS02NonFungibleTokenPacketData {
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self, ContractError> {
        let values = ethabi::decode(
            &[
                ParamType::String,
                ParamType::String,
                ParamType::Bytes,
                ParamType::Array(Box::new(ParamType::String)),
                ParamType::Array(Box::new(ParamType::String)),
                ParamType::Array(Box::new(ParamType::Bytes)),
                ParamType::String,
                ParamType::String,
                ParamType::String,
            ],
            bz.as_ref(),
        )
        .map_err(|_| ContractError::EthAbiDecoding)?;
        match &values[..] {
            [Token::String(class_id), Token::String(class_uri), Token::Bytes(class_data), Token::Array(token_ids), Token::Array(token_uris), Token::Array(token_data), Token::String(sender), Token::String(receiver), Token::String(memo)] => {
                Ok(NonFungibleTokenPacketData {
                    class_id: ics721::ClassId::new(class_id),
                    class_uri: if class_uri.is_empty() {
                        None
                    } else {
                        Some(class_uri.clone())
                    },
                    class_data: if class_data.is_empty() {
                        None
                    } else {
                        Some(class_data.clone().into())
                    },
                    token_ids: token_ids
                        .iter()
                        .cloned()
                        .map(|token_id| match token_id {
                            Token::String(token_id) => Ok(ics721::TokenId::new(token_id)),
                            _ => Err(ContractError::EthAbiDecoding),
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                    token_uris: if token_uris.is_empty() {
                        None
                    } else {
                        Some(
                            token_uris
                                .iter()
                                .cloned()
                                .map(|token_uri| match token_uri {
                                    Token::String(token_uri) => Ok(token_uri),
                                    _ => Err(ContractError::EthAbiDecoding),
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        )
                    },
                    token_data: if token_data.is_empty() {
                        None
                    } else {
                        Some(
                            token_data
                                .iter()
                                .cloned()
                                .map(|token_data| match token_data {
                                    Token::Bytes(token_data) => Ok(token_data.into()),
                                    _ => Err(ContractError::EthAbiDecoding),
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        )
                    },
                    sender: sender.clone(),
                    receiver: receiver.clone(),
                    memo: if memo.is_empty() {
                        None
                    } else {
                        Some(memo.clone())
                    },
                }
                .into())
            }
            _ => Err(ContractError::EthAbiDecoding),
        }
    }

    pub fn encode(self) -> Vec<u8> {
        ethabi::encode(&[
            Token::String(self.0.class_id.into()),
            Token::String(self.0.class_uri.unwrap_or_default()),
            Token::Bytes(self.0.class_data.unwrap_or_default().into()),
            Token::Array(
                self.0
                    .token_ids
                    .into_iter()
                    .map(|token_id| Token::String(token_id.into()))
                    .collect(),
            ),
            Token::Array(
                self.0
                    .token_uris
                    .unwrap_or_default()
                    .into_iter()
                    .map(|token_uri| Token::String(token_uri.into()))
                    .collect(),
            ),
            Token::Array(
                self.0
                    .token_data
                    .unwrap_or_default()
                    .into_iter()
                    .map(|token_data| Token::Bytes(token_data.into()))
                    .collect(),
            ),
            Token::String(self.0.sender),
            Token::String(self.0.receiver),
            Token::String(self.0.memo.unwrap_or_default()),
        ])
    }
}

pub fn validate_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<Version, ContractError> {
    if channel.order != IbcOrder::Unordered {
        return Err(ics721::ContractError::OrderedChannel {}.into());
    }
    let version = Version::try_from(channel.version.as_ref())?;
    match counterparty_version.map(Version::try_from).transpose()? {
        Some(counterparty_version) if version != counterparty_version => {
            Err(ContractError::IbcVersionMismatch {
                src: version,
                dst: counterparty_version,
            })
        }
        _ => Ok(version),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ics721::msg::InstantiateMsg,
) -> StdResult<Response> {
    sg_ics721::state::SgIcs721Contract::default().instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ics721::msg::ExecuteMsg,
) -> Result<Response, ContractError> {
    sg_ics721::state::SgIcs721Contract::default()
        .execute(deps.branch(), env, info, msg)
        .map(|mut response| {
            response.messages = response
                .messages
                .into_iter()
                .map(|mut message| {
                    message.msg = match message.msg {
                        CosmosMsg::Ibc(IbcMsg::SendPacket {
                            channel_id,
                            data,
                            timeout,
                        }) => {
                            let version = CHANNEL_VERSION
                                .load(deps.storage, &channel_id)
                                .expect("impossible");
                            match version {
                                Version::ICS721 => CosmosMsg::Ibc(IbcMsg::SendPacket {
                                    channel_id,
                                    data,
                                    timeout,
                                }),
                                Version::UCS02 => {
                                    let nft_data = from_json::<NonFungibleTokenPacketData>(&data)
                                        .expect("impossible");
                                    CosmosMsg::Ibc(IbcMsg::SendPacket {
                                        channel_id,
                                        data: UCS02NonFungibleTokenPacketData::from(nft_data)
                                            .encode()
                                            .into(),
                                        timeout,
                                    })
                                }
                            }
                        }
                        msg => msg,
                    };
                    message
                })
                .collect();
            response
        })
        .map_err(Into::into)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: ics721::msg::QueryMsg) -> StdResult<Binary> {
    sg_ics721::state::SgIcs721Contract::default().query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: ics721::msg::MigrateMsg,
) -> Result<Response, ContractError> {
    sg_ics721::state::SgIcs721Contract::default()
        .migrate(deps, env, msg)
        .map_err(Into::into)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    sg_ics721::state::SgIcs721Contract::default()
        .reply(deps, env, reply)
        .map_err(Into::into)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(IbcChannelOpenResponse::None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let version = validate_order_and_version(msg.channel(), msg.counterparty_version())?;
    CHANNEL_VERSION.save(deps.storage, &msg.channel().endpoint.channel_id, &version)?;
    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_connect")
        .add_attribute("channel", &msg.channel().endpoint.channel_id)
        .add_attribute("port", &msg.channel().endpoint.port_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    sg_ics721::state::SgIcs721Contract::default()
        .ibc_channel_close(deps, env, msg)
        .map_err(Into::into)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    mut msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let receive_ibc_packet = || -> Result<IbcPacketReceiveMsg, ContractError> {
        let version = CHANNEL_VERSION
            .load(deps.storage, &msg.packet.dest.channel_id)
            .expect("impossible");
        match version {
            Version::ICS721 => Ok(msg),
            Version::UCS02 => {
                let nft_data = UCS02NonFungibleTokenPacketData::decode(&msg.packet.data)?;
                msg.packet.data = to_json_binary(&nft_data.0)?;
                Ok(msg)
            }
        }
    };
    match receive_ibc_packet() {
        Ok(msg) => sg_ics721::state::SgIcs721Contract::default().ibc_packet_receive(deps, env, msg),
        Err(error) => Ok(IbcReceiveResponse::new()
            .add_attribute("method", "ibc_packet_receive")
            .add_attribute("error", error.to_string())
            .set_ack(ics721::ibc_helpers::ack_fail(error.to_string()))),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    ack: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ics721::ContractError> {
    sg_ics721::state::SgIcs721Contract::default().ibc_packet_ack(deps, env, ack)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ics721::ContractError> {
    sg_ics721::state::SgIcs721Contract::default().ibc_packet_timeout(deps, env, msg)
}
