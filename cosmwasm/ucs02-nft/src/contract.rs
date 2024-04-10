use std::str::FromStr;

use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_json, to_json_binary, Binary, ContractInfoResponse, CosmosMsg, Decimal, Deps, DepsMut,
    Env, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcMsg, IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, MessageInfo, Never, Reply, Response, StdError,
    StdResult, SubMsg, Timestamp,
};
use cw_storage_plus::{Item, Map};
use ethabi::{ParamType, Token};
use ics721::{
    execute::Ics721Execute, ibc::Ics721Ibc, query::Ics721Query, NonFungibleTokenPacketData,
};
use thiserror::Error;
use unionlabs::uint::U256;

pub const CHANNEL_VERSION: Map<&str, Version> = Map::new("channel_version");

pub const CURRENT_VERSION: Item<Version> = Item::new("current_version");

pub const UCS_NFT_IBC_VERSION: &str = "ucs02-nft-1";

// TODO: provide copy url
#[cw_serde]
pub struct RoyaltyInfoResponse {
    pub payment_address: String,
    pub share: Decimal,
}
#[cw_serde]
pub struct CollectionInfoResponse {
    pub creator: String,
    pub description: String,
    pub image: String,
    pub external_link: Option<String>,
    pub explicit_content: Option<bool>,
    pub start_trading_time: Option<Timestamp>,
    pub royalty_info: Option<RoyaltyInfoResponse>,
}
#[cw_serde]
pub struct SgCollectionData {
    pub owner: Option<String>,
    pub contract_info: Option<ContractInfoResponse>,
    pub name: String,
    pub symbol: String,
    pub num_tokens: Option<u64>,
    /// SG721 specific collection info
    pub collection_info: Option<CollectionInfoResponse>,
}

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
    #[error("We expected an UCS02 acknowledgement, but got: {acknowledgement}")]
    InvalidAcknowledgement { acknowledgement: Binary },
    #[error(transparent)]
    TryFromNonFungibleTokenPacketDataError(#[from] TryFromNonFungibleTokenPacketDataError),
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

pub struct UCS02NonFungibleTokenPacketData {
    /// Owner
    pub class_owner: String,
    /// Uniquely identifies the collection which the tokens being
    /// transfered belong to on the sending chain. Must be non-empty.
    pub class_id: ics721::ClassId,
    /// Collection name
    pub class_name: String,
    /// Collection symbol
    pub class_symbol: String,
    /// Uniquely identifies the tokens in the NFT collection being
    /// transfered. This MUST be non-empty.
    pub token_ids: Vec<U256>,
    /// Optional URL that points to metadata for each token being
    /// transfered. `tokenUris[N]` should hold the metadata for
    /// `tokenIds[N]` and both lists should have the same if
    /// provided. Must be non-empty if provided.
    pub token_uris: Option<Vec<String>>,
    /// The address sending the tokens on the sending chain.
    pub sender: String,
    /// The address that should receive the tokens on the receiving
    /// chain.
    pub receiver: String,
    /// Memo to add custom string to the msg
    pub memo: Option<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum TryFromNonFungibleTokenPacketDataError {
    #[error("only u256 token ids are bridgeable: {token_id}")]
    InvalidTokenId { token_id: String },
    #[error("class data must be of type `ics721::state::CollectionData`: {class_data}")]
    InvalidClassData { class_data: Binary },
}

impl TryFrom<NonFungibleTokenPacketData> for UCS02NonFungibleTokenPacketData {
    type Error = TryFromNonFungibleTokenPacketDataError;
    fn try_from(value: NonFungibleTokenPacketData) -> Result<Self, Self::Error> {
        // We only support ics721/sg_ics721 collectiondata compatible nfts
        let class_data = value.class_data.as_ref();
        let (owner, name, symbol) = match from_json::<ics721::state::CollectionData>(
            class_data.unwrap_or(&Default::default()),
        )
        .or_else(|_| {
            Err(from_json::<SgCollectionData>(
                class_data.unwrap_or(&Default::default()),
            ))
        }) {
            Ok(data) => Ok((data.owner, data.name, data.symbol)),
            Err(Ok(data)) => Ok((data.owner, data.name, data.symbol)),
            _ => Err(TryFromNonFungibleTokenPacketDataError::InvalidClassData {
                class_data: value.class_data.unwrap_or_default(),
            }),
        }?;
        Ok(Self {
            class_owner: owner.unwrap_or_default(),
            class_id: value.class_id,
            class_name: name,
            class_symbol: symbol,
            token_ids: value
                .token_ids
                .into_iter()
                .map(|token_id| {
                    // Sadly, ics721 do not impl asref/deref for tokenid :')
                    U256::from_str(&String::from(token_id.clone())).map_err(|_| {
                        TryFromNonFungibleTokenPacketDataError::InvalidTokenId {
                            token_id: token_id.into(),
                        }
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            token_uris: value.token_uris,
            sender: value.sender,
            receiver: value.receiver,
            memo: value.memo,
        })
    }
}

impl From<UCS02NonFungibleTokenPacketData> for NonFungibleTokenPacketData {
    fn from(value: UCS02NonFungibleTokenPacketData) -> Self {
        Self {
            class_id: value.class_id,
            class_uri: Default::default(),
            class_data: Some(
                to_json_binary(&ics721::state::CollectionData {
                    owner: Some(value.class_owner),
                    contract_info: None,
                    name: value.class_name,
                    symbol: value.class_symbol,
                    num_tokens: None,
                })
                .expect("impossible"),
            ),
            token_ids: value
                .token_ids
                .into_iter()
                .map(|token_id| ics721::TokenId::new(token_id.to_string()))
                .collect(),
            token_uris: value.token_uris,
            token_data: Default::default(),
            sender: value.sender,
            receiver: value.receiver,
            memo: value.memo,
        }
    }
}

impl UCS02NonFungibleTokenPacketData {
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self, ContractError> {
        let values = ethabi::decode(
            &[
                ParamType::String,
                ParamType::String,
                ParamType::String,
                ParamType::String,
                ParamType::Array(Box::new(ParamType::Uint(256))),
                ParamType::Array(Box::new(ParamType::String)),
                ParamType::String,
                ParamType::String,
                ParamType::String,
            ],
            bz.as_ref(),
        )
        .map_err(|_| ContractError::EthAbiDecoding)?;
        match &values[..] {
            [Token::String(class_owner), Token::String(class_id), Token::String(class_name), Token::String(class_symbol), Token::Array(token_ids), Token::Array(token_uris), Token::String(sender), Token::String(receiver), Token::String(memo)] => {
                Ok(UCS02NonFungibleTokenPacketData {
                    class_owner: class_owner.into(),
                    class_id: ics721::ClassId::new(class_id),
                    class_name: class_name.into(),
                    class_symbol: class_symbol.into(),
                    token_ids: token_ids
                        .iter()
                        .cloned()
                        .map(|token_id| match token_id {
                            Token::Uint(token_id) => Ok(U256::from(token_id)),
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
            Token::String(self.class_owner.into()),
            Token::String(self.class_id.into()),
            Token::String(self.class_name.into()),
            Token::String(self.class_symbol.into()),
            Token::Array(
                self.token_ids
                    .into_iter()
                    .map(|token_id| Token::Uint(token_id.into()))
                    .collect(),
            ),
            Token::Array(
                self.token_uris
                    .unwrap_or_default()
                    .into_iter()
                    .map(|token_uri| Token::String(token_uri.into()))
                    .collect(),
            ),
            Token::String(self.sender),
            Token::String(self.receiver),
            Token::String(self.memo.unwrap_or_default()),
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
    ics721_base::state::Ics721Contract::default().instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ics721::msg::ExecuteMsg,
) -> Result<Response, ContractError> {
    let mut response =
        ics721_base::state::Ics721Contract::default().execute(deps.branch(), env, info, msg)?;
    // If the target protocol is UCS02, transform the packet
    for message in &mut response.messages {
        if let CosmosMsg::Ibc(IbcMsg::SendPacket {
            channel_id, data, ..
        }) = &mut message.msg
        {
            let version = CHANNEL_VERSION
                .load(deps.storage, &channel_id)
                .expect("impossible");
            if let Version::UCS02 = version {
                let nft_data = from_json::<NonFungibleTokenPacketData>(&data).expect("impossible");
                *data = UCS02NonFungibleTokenPacketData::try_from(nft_data)?
                    .encode()
                    .into();
            }
        }
    }
    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: ics721::msg::QueryMsg) -> StdResult<Binary> {
    ics721_base::state::Ics721Contract::default().query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    env: Env,
    msg: ics721::msg::MigrateMsg,
) -> Result<Response, ContractError> {
    ics721_base::state::Ics721Contract::default()
        .migrate(deps, env, msg)
        .map_err(Into::into)
}

// TODO: From ics721, make it public...
pub(crate) const ACK_AND_DO_NOTHING_REPLY_ID: u64 = 3;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(mut deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let mut response =
        ics721_base::state::Ics721Contract::default().reply(deps.branch(), env, reply.clone())?;
    if reply.id == ACK_AND_DO_NOTHING_REPLY_ID {
        if CURRENT_VERSION.load(deps.storage)? == Version::UCS02 {
            response.data = Some(
                match from_json(&response.data.unwrap_or_default())
                    .unwrap_or_else(|_| ics721::ibc_helpers::Ics721Ack::Error(Default::default()))
                {
                    ics721::ibc_helpers::Ics721Ack::Result(_) => [1].into(),
                    ics721::ibc_helpers::Ics721Ack::Error(_) => [0].into(),
                },
            );
        }
    }
    Ok(response)
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
    ics721_base::state::Ics721Contract::default()
        .ibc_channel_close(deps, env, msg)
        .map_err(Into::into)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    mut msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let version = CHANNEL_VERSION
        .load(deps.storage, &msg.packet.dest.channel_id)
        .expect("impossible");
    let msg = match version {
        Version::ICS721 => msg,
        Version::UCS02 => {
            // Decode eth abi encoded NFT packet, reencode in JSON
            let nft_data = NonFungibleTokenPacketData::from(
                UCS02NonFungibleTokenPacketData::decode(&msg.packet.data).expect("impossible"),
            );
            msg.packet.data = to_json_binary(&nft_data).expect("impossible");
            msg
        }
    };
    // We store the version we are handling to hook the ack on reply
    CURRENT_VERSION
        .save(deps.storage, &version)
        .expect("impossible");
    match ics721_base::state::Ics721Contract::default().ibc_packet_receive(deps, env, msg) {
        Ok(mut response) => {
            response.acknowledgement = match version {
                Version::ICS721 => response.acknowledgement,
                // Reencode the acknowledgement
                Version::UCS02 => {
                    match from_json(&response.acknowledgement).unwrap_or_else(|_| {
                        ics721::ibc_helpers::Ics721Ack::Error(Default::default())
                    }) {
                        ics721::ibc_helpers::Ics721Ack::Result(_) => [1].into(),
                        ics721::ibc_helpers::Ics721Ack::Error(_) => [0].into(),
                    }
                }
            };
            Ok(response)
        }
        Err(_) => panic!("impossible"),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    mut msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let version = CHANNEL_VERSION
        .load(deps.storage, &msg.original_packet.src.channel_id)
        .expect("impossible");
    let normalized_msg = match version {
        Version::ICS721 => msg,
        Version::UCS02 => {
            // Decode eth abi encoded NFT packet, reencode in JSON
            msg.original_packet.data = to_json_binary(&NonFungibleTokenPacketData::from(
                UCS02NonFungibleTokenPacketData::decode(&msg.original_packet.data)?,
            ))?;
            // Decode eth abi encoded ACK, reencode in JSON
            msg.acknowledgement.data = match msg.acknowledgement.data.as_ref() {
                [0] => ics721::ibc_helpers::ack_fail("evm execution reverted".into()),
                [1] => ics721::ibc_helpers::ack_success(),
                _ => {
                    return Err(ContractError::InvalidAcknowledgement {
                        acknowledgement: msg.acknowledgement.data,
                    })
                }
            };
            msg
        }
    };
    ics721_base::state::Ics721Contract::default()
        .ibc_packet_ack(deps, env, normalized_msg)
        .map_err(Into::into)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ics721::ContractError> {
    ics721_base::state::Ics721Contract::default().ibc_packet_timeout(deps, env, msg)
}
