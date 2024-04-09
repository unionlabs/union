use cosmwasm_std::{
    wasm_execute, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, HexBinary, IbcEndpoint, IbcOrder,
    MessageInfo, Uint128, Uint512,
};
use sha2::{Digest, Sha256};
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::{
    protocol::TransferProtocol,
    types::{
        make_foreign_denom, DenomOrigin, EncodingError, Ics20Ack, Ics20Packet, TransferPacket,
        TransferToken, Ucs01Ack, Ucs01TransferPacket,
    },
};

use crate::{
    error::ContractError,
    msg::ExecuteMsg,
    state::{
        ChannelInfo, Hash, CHANNEL_STATE, FOREIGN_DENOM_TO_HASH, HASH_LENGTH, HASH_TO_FOREIGN_DENOM,
    },
};

pub fn hash_denom(denom: &str) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(denom);
    hasher.finalize()[..HASH_LENGTH]
        .try_into()
        .expect("impossible")
}

pub fn hash_denom_str(denom: &str) -> String {
    format!("0x{}", hex::encode(hash_denom(denom)))
}

pub fn protocol_ordering(version: &str) -> Option<IbcOrder> {
    match version {
        Ics20Protocol::VERSION => Some(Ics20Protocol::ORDERING),
        Ucs01Protocol::VERSION => Some(Ucs01Protocol::ORDERING),
        _ => None,
    }
}

fn batch_submessages(
    self_addr: &cosmwasm_std::Addr,
    msgs: Vec<CosmosMsg<TokenFactoryMsg>>,
) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
    Ok(vec![wasm_execute(
        self_addr,
        &ExecuteMsg::BatchExecute { msgs },
        vec![],
    )?
    .into()])
}

fn update_outstanding<F>(
    deps: DepsMut,
    channel_id: &str,
    denom: &str,
    f: F,
) -> Result<(), ContractError>
where
    F: FnOnce(Option<Uint512>) -> Result<Uint512, ContractError>,
{
    CHANNEL_STATE.update(
        deps.storage,
        (channel_id, denom),
        |state| -> Result<_, ContractError> {
            let new_outstanding = f(state.as_ref().map(|x| x.outstanding))?;
            let mut state = state.unwrap_or_default();
            state.outstanding = new_outstanding;
            Ok(state)
        },
    )?;
    Ok(())
}

fn increase_outstanding(
    deps: DepsMut,
    channel_id: &str,
    denom: &str,
    amount: Uint128,
) -> Result<(), ContractError> {
    update_outstanding(deps, channel_id, denom, |outstanding| {
        let new_outstanding = outstanding.unwrap_or_default().checked_add(amount.into())?;
        Ok(new_outstanding)
    })
}

fn decrease_outstanding(
    deps: DepsMut,
    channel_id: &str,
    denom: &str,
    amount: Uint128,
) -> Result<(), ContractError> {
    update_outstanding(deps, channel_id, denom, |outstanding| {
        let new_outstanding = outstanding
            .ok_or(ContractError::InsufficientFunds)?
            .checked_sub(amount.into())?;
        Ok(new_outstanding)
    })
}

fn normalize_for_ibc_transfer(
    mut hash_to_denom: impl FnMut(Hash) -> Result<Option<String>, ContractError>,
    contract_address: &str,
    counterparty_endpoint: &IbcEndpoint,
    token: TransferToken,
) -> Result<TransferToken, ContractError> {
    let normalized_denom = match token
        .denom
        .strip_prefix("factory/")
        .and_then(|denom| denom.strip_prefix(contract_address))
        .and_then(|denom| denom.strip_prefix("/0x"))
    {
        Some(denom_hash) => {
            if let Some(normalized_denom) = hash_to_denom(
                hex::decode(denom_hash)
                    .expect("impossible")
                    .try_into()
                    .expect("impossible"),
            )? {
                // Check whether it's a remotely created denom by switching
                // to the remote POV and using the DenomOrigin parser.
                match DenomOrigin::from((normalized_denom.as_ref(), counterparty_endpoint)) {
                    DenomOrigin::Local { .. } => normalized_denom,
                    DenomOrigin::Remote { .. } => token.denom,
                }
            } else {
                token.denom
            }
        }
        None => token.denom,
    };
    Ok(TransferToken {
        denom: normalized_denom.to_string(),
        amount: token.amount,
    })
}
trait OnReceive {
    fn foreign_toggle(
        &mut self,
        contract_address: &Addr,
        local_endpoint: &IbcEndpoint,
        denom: &str,
    ) -> Result<(bool, Hash, CosmosMsg<TokenFactoryMsg>), ContractError>;

    fn local_unescrow(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<(), ContractError>;

    fn receive_phase1_transfer(
        &mut self,
        contract_address: &Addr,
        endpoint: &IbcEndpoint,
        counterparty_endpoint: &IbcEndpoint,
        receiver: &str,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        tokens
            .into_iter()
            .map(|TransferToken { denom, amount }| {
                let amount = amount
                    .try_into()
                    .expect("CosmWasm require transferred amount to be Uint128...");
                match DenomOrigin::from((denom.as_str(), endpoint)) {
                    DenomOrigin::Local { denom } => {
                        self.local_unescrow(&endpoint.channel_id, denom, amount)?;
                        Ok(vec![BankMsg::Send {
                            to_address: receiver.to_string(),
                            amount: vec![Coin {
                                denom: denom.to_string(),
                                amount,
                            }],
                        }
                        .into()])
                    }
                    DenomOrigin::Remote { denom } => {
                        let foreign_denom = make_foreign_denom(counterparty_endpoint, denom);
                        let (exists, hashed_foreign_denom, register_msg) =
                            self.foreign_toggle(contract_address, endpoint, &foreign_denom)?;
                        let normalized_foreign_denom =
                            format!("0x{}", hex::encode(hashed_foreign_denom));
                        let factory_denom =
                            format!("factory/{}/{}", contract_address, normalized_foreign_denom);
                        let mint = TokenFactoryMsg::MintTokens {
                            denom: factory_denom,
                            amount,
                            mint_to_address: receiver.to_string(),
                        };
                        Ok(if exists {
                            vec![mint.into()]
                        } else {
                            vec![
                                register_msg,
                                TokenFactoryMsg::CreateDenom {
                                    subdenom: normalized_foreign_denom.clone(),
                                    metadata: None,
                                }
                                .into(),
                                mint.into(),
                            ]
                        })
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|x| x.into_iter().flatten().collect())
    }
}

pub struct StatefulOnReceive<'a> {
    deps: DepsMut<'a>,
}
impl<'a> OnReceive for StatefulOnReceive<'a> {
    fn foreign_toggle(
        &mut self,
        contract_address: &Addr,
        local_endpoint: &IbcEndpoint,
        denom: &str,
    ) -> Result<(bool, Hash, CosmosMsg<TokenFactoryMsg>), ContractError> {
        let exists = FOREIGN_DENOM_TO_HASH.has(
            self.deps.storage,
            (local_endpoint.clone().into(), denom.to_string()),
        );
        let hash = hash_denom(&format!(
            "{}/{}/{}",
            local_endpoint.port_id, local_endpoint.channel_id, denom
        ));
        Ok((
            exists,
            hash,
            wasm_execute(
                contract_address,
                &ExecuteMsg::RegisterDenom {
                    local_endpoint: local_endpoint.clone(),
                    denom: denom.to_string(),
                    hash: hash.into(),
                },
                Default::default(),
            )?
            .into(),
        ))
    }

    fn local_unescrow(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<(), ContractError> {
        decrease_outstanding(self.deps.branch(), channel_id, denom, amount)?;
        Ok(())
    }
}

trait ForTokens {
    fn on_local(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError>;

    fn on_remote(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError>;

    fn execute(
        &mut self,
        contract_address: &Addr,
        endpoint: &IbcEndpoint,
        counterparty_endpoint: &IbcEndpoint,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        let mut messages = Vec::with_capacity(tokens.len());
        for TransferToken { denom, amount } in tokens {
            let amount = amount
                .try_into()
                .expect("CosmWasm require transferred amount to be Uint128...");
            // This is the origin from the counterparty POV
            match DenomOrigin::from((denom.as_str(), counterparty_endpoint)) {
                DenomOrigin::Local { denom } => {
                    // the denom has been previously normalized (factory/{}/ prefix removed), we must reconstruct to burn
                    let foreign_denom = hash_denom_str(&format!(
                        "{}/{}/{}",
                        endpoint.port_id,
                        endpoint.channel_id,
                        make_foreign_denom(counterparty_endpoint, denom)
                    ));
                    let factory_denom = format!("factory/{}/{}", contract_address, foreign_denom);
                    messages.append(&mut self.on_remote(
                        &endpoint.channel_id,
                        &factory_denom,
                        amount,
                    )?);
                }
                DenomOrigin::Remote { denom } => {
                    messages.append(&mut self.on_local(&endpoint.channel_id, denom, amount)?);
                }
            }
        }
        Ok(messages)
    }
}

struct StatefulSendTokens<'a> {
    deps: DepsMut<'a>,
    contract_address: String,
}

impl<'a> ForTokens for StatefulSendTokens<'a> {
    fn on_local(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        increase_outstanding(self.deps.branch(), channel_id, denom, amount)?;
        Ok(Default::default())
    }

    fn on_remote(
        &mut self,
        _channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        Ok(vec![TokenFactoryMsg::BurnTokens {
            denom: denom.into(),
            amount,
            burn_from_address: self.contract_address.clone(),
        }
        .into()])
    }
}

struct StatefulRefundTokens<'a> {
    deps: DepsMut<'a>,
    receiver: String,
}

impl<'a> ForTokens for StatefulRefundTokens<'a> {
    fn on_local(
        &mut self,
        channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        decrease_outstanding(self.deps.branch(), channel_id, denom, amount)?;
        Ok(vec![BankMsg::Send {
            to_address: self.receiver.clone(),
            amount: vec![Coin {
                denom: denom.into(),
                amount,
            }],
        }
        .into()])
    }

    fn on_remote(
        &mut self,
        _channel_id: &str,
        denom: &str,
        amount: Uint128,
    ) -> Result<Vec<CosmosMsg<TokenFactoryMsg>>, ContractError> {
        Ok(vec![TokenFactoryMsg::MintTokens {
            denom: denom.into(),
            amount,
            mint_to_address: self.receiver.clone(),
        }
        .into()])
    }
}

pub struct ProtocolCommon<'a> {
    pub deps: DepsMut<'a>,
    pub env: Env,
    pub info: MessageInfo,
    pub channel: ChannelInfo,
}

pub struct Ics20Protocol<'a> {
    pub common: ProtocolCommon<'a>,
}

impl<'a> TransferProtocol for Ics20Protocol<'a> {
    const VERSION: &'static str = "ics20-1";
    const ORDERING: IbcOrder = IbcOrder::Unordered;
    const RECEIVE_REPLY_ID: u64 = 0;

    type Packet = Ics20Packet;
    type Ack = Ics20Ack;
    type CustomMsg = TokenFactoryMsg;
    type Error = ContractError;

    fn channel_endpoint(&self) -> &cosmwasm_std::IbcEndpoint {
        &self.common.channel.endpoint
    }

    fn caller(&self) -> &cosmwasm_std::Addr {
        &self.common.info.sender
    }

    fn self_addr(&self) -> &cosmwasm_std::Addr {
        &self.common.env.contract.address
    }

    fn ack_success() -> Self::Ack {
        Ics20Ack::Result(vec![1].into())
    }

    fn ack_failure(error: String) -> Self::Ack {
        Ics20Ack::Error(error)
    }

    fn send_tokens(
        &mut self,
        _sender: &String,
        _receiver: &String,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulSendTokens {
            deps: self.common.deps.branch(),
            contract_address: self.common.env.contract.address.to_string(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            tokens,
        )
    }

    fn send_tokens_success(
        &mut self,
        _sender: &String,
        _receiver: &String,
        _tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        Ok(Default::default())
    }

    fn send_tokens_failure(
        &mut self,
        sender: &String,
        _receiver: &String,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulRefundTokens {
            deps: self.common.deps.branch(),
            receiver: sender.into(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            tokens,
        )
    }

    fn receive_transfer(
        &mut self,
        receiver: &<Self::Packet as TransferPacket>::Addr,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, ContractError> {
        StatefulOnReceive {
            deps: self.common.deps.branch(),
        }
        .receive_phase1_transfer(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            receiver.as_str(),
            tokens,
        )
        .map(|msgs| batch_submessages(self.self_addr(), msgs))?
    }
    fn normalize_for_ibc_transfer(
        &mut self,
        token: TransferToken,
    ) -> Result<TransferToken, Self::Error> {
        normalize_for_ibc_transfer(
            |hash| {
                HASH_TO_FOREIGN_DENOM
                    .may_load(
                        self.common.deps.storage,
                        (self.common.channel.endpoint.clone().into(), hash),
                    )
                    .map_err(Into::into)
            },
            self.common.env.contract.address.as_str(),
            &self.common.channel.counterparty_endpoint,
            token,
        )
    }

    fn common_to_protocol_packet(
        &self,
        packet: ucs01_relay_api::types::TransferPacketCommon<
            ucs01_relay_api::protocol::PacketExtensionOf<Self>,
        >,
    ) -> Result<Self::Packet, ucs01_relay_api::types::EncodingError> {
        Ics20Packet::try_from(packet)
    }
}

pub struct Ucs01Protocol<'a> {
    pub common: ProtocolCommon<'a>,
}

impl<'a> TransferProtocol for Ucs01Protocol<'a> {
    const VERSION: &'static str = "ucs01-0";
    const ORDERING: IbcOrder = IbcOrder::Unordered;
    const RECEIVE_REPLY_ID: u64 = 1;

    type Packet = Ucs01TransferPacket;
    type Ack = Ucs01Ack;
    type CustomMsg = TokenFactoryMsg;
    type Error = ContractError;

    fn channel_endpoint(&self) -> &cosmwasm_std::IbcEndpoint {
        &self.common.channel.endpoint
    }

    fn caller(&self) -> &cosmwasm_std::Addr {
        &self.common.info.sender
    }

    fn self_addr(&self) -> &cosmwasm_std::Addr {
        &self.common.env.contract.address
    }

    fn ack_success() -> Self::Ack {
        Ucs01Ack::Success
    }

    fn ack_failure(_: String) -> Self::Ack {
        Ucs01Ack::Failure
    }

    fn send_tokens(
        &mut self,
        _sender: &HexBinary,
        _receiver: &HexBinary,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulSendTokens {
            deps: self.common.deps.branch(),
            contract_address: self.common.env.contract.address.to_string(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            tokens,
        )
    }

    fn send_tokens_success(
        &mut self,
        _sender: &HexBinary,
        _receiver: &HexBinary,
        _tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        Ok(Default::default())
    }

    fn send_tokens_failure(
        &mut self,
        sender: &HexBinary,
        _receiver: &HexBinary,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        let addr = self.common.deps.api.addr_humanize(&sender.clone().into())?;
        StatefulRefundTokens {
            deps: self.common.deps.branch(),
            receiver: addr.to_string(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            tokens,
        )
    }

    fn receive_transfer(
        &mut self,
        receiver: &<Self::Packet as TransferPacket>::Addr,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, ContractError> {
        let receiver = self
            .common
            .deps
            .api
            .addr_humanize(&receiver.clone().into())?;
        StatefulOnReceive {
            deps: self.common.deps.branch(),
        }
        .receive_phase1_transfer(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            receiver.as_str(),
            tokens,
        )
        .map(|msgs| batch_submessages(self.self_addr(), msgs))?
    }

    fn normalize_for_ibc_transfer(
        &mut self,
        token: TransferToken,
    ) -> Result<TransferToken, Self::Error> {
        normalize_for_ibc_transfer(
            |hash| {
                HASH_TO_FOREIGN_DENOM
                    .may_load(
                        self.common.deps.storage,
                        (self.common.channel.endpoint.clone().into(), hash),
                    )
                    .map_err(Into::into)
            },
            self.common.env.contract.address.as_str(),
            &self.common.channel.counterparty_endpoint,
            token,
        )
    }

    fn common_to_protocol_packet(
        &self,
        packet: ucs01_relay_api::types::TransferPacketCommon<
            ucs01_relay_api::protocol::PacketExtensionOf<Self>,
        >,
    ) -> Result<Self::Packet, EncodingError> {
        Ok(Ucs01TransferPacket::new(
            self.common
                .deps
                .api
                .addr_canonicalize(&packet.sender)
                .map_err(|_| EncodingError::InvalidEncoding)?
                .into(),
            HexBinary::from_hex(&packet.receiver).map_err(|_| EncodingError::InvalidEncoding)?,
            packet.tokens,
        ))
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        wasm_execute, Addr, BankMsg, Coin, CosmosMsg, IbcChannel, IbcEndpoint, Uint128, Uint256,
        Uint512,
    };
    use token_factory_api::TokenFactoryMsg;
    use ucs01_relay_api::types::TransferToken;

    use super::{hash_denom, ForTokens, OnReceive, StatefulOnReceive};
    use crate::{
        contract::{execute, instantiate},
        error::ContractError,
        ibc::ibc_channel_connect,
        msg::{ExecuteMsg, InitMsg, TransferMsg},
        protocol::{hash_denom_str, normalize_for_ibc_transfer},
        state::{Hash, CHANNEL_STATE},
    };

    struct TestOnReceive {
        toggle: bool,
    }
    impl OnReceive for TestOnReceive {
        fn foreign_toggle(
            &mut self,
            contract_address: &Addr,
            local_endpoint: &IbcEndpoint,
            denom: &str,
        ) -> Result<(bool, Hash, CosmosMsg<TokenFactoryMsg>), ContractError> {
            let mut deps = mock_dependencies();
            let (_, hash, msg) = StatefulOnReceive {
                deps: deps.as_mut(),
            }
            .foreign_toggle(contract_address, local_endpoint, denom)?;
            Ok((self.toggle, hash, msg))
        }

        fn local_unescrow(
            &mut self,
            _channel_id: &str,
            _denom: &str,
            _amount: Uint128,
        ) -> Result<(), crate::error::ContractError> {
            Ok(())
        }
    }

    #[test]
    fn receive_transfer_create_foreign() {
        let denom = hash_denom("wasm.0xDEADC0DE/channel-1/transfer/channel-34/from-counterparty");
        let denom_str =
            hash_denom_str("wasm.0xDEADC0DE/channel-1/transfer/channel-34/from-counterparty");
        assert_eq!(
            TestOnReceive { toggle: false }.receive_phase1_transfer(
                &Addr::unchecked("0xDEADC0DE"),
                &IbcEndpoint {
                    port_id: "wasm.0xDEADC0DE".into(),
                    channel_id: "channel-1".into(),
                },
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-34".into(),
                },
                "receiver",
                vec![TransferToken {
                    denom: "from-counterparty".into(),
                    amount: Uint256::from(100u128)
                },],
            ),
            Ok(vec![
                wasm_execute(
                    Addr::unchecked("0xDEADC0DE"),
                    &ExecuteMsg::RegisterDenom {
                        local_endpoint: IbcEndpoint {
                            port_id: "wasm.0xDEADC0DE".into(),
                            channel_id: "channel-1".into(),
                        },
                        denom: "transfer/channel-34/from-counterparty".into(),
                        hash: denom.into(),
                    },
                    Default::default()
                )
                .unwrap()
                .into(),
                TokenFactoryMsg::CreateDenom {
                    subdenom: denom_str.clone(),
                    metadata: None
                }
                .into(),
                TokenFactoryMsg::MintTokens {
                    denom: format!("factory/0xDEADC0DE/{}", denom_str),
                    amount: Uint128::from(100u128),
                    mint_to_address: "receiver".into()
                }
                .into(),
            ])
        );
    }

    /// When this contract have 2 channels open to 2 different ucs relays and the counterparty
    /// channel/ports are the same, the denoms collide. The flow here is:
    /// 1. Receive `denom` from `A` with `port-id-N/channel-id-N` over `channel-id-A`
    /// 2. Receive `denom` from `B` with `port-id-N/channel-id-N` over `channel-id-B`
    /// 3. Send the token that is received from `A` to `B`.
    /// Expected result: Although the counterparty port id/channel id are the same, there are
    /// two different counterparty, so this token should be considered to be local and outstanding
    /// should be increased instead of burning.
    #[test]
    fn receive_transfer_destination_collusion_outstanding_instead_of_burn() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let source_endpoint_1 = IbcEndpoint {
            port_id: "wasm.0xDEADC0DE".into(),
            channel_id: "channel-1".into(),
        };
        let source_endpoint_2 = IbcEndpoint {
            port_id: "wasm.0xDEADC0DE".into(),
            channel_id: "channel-2".into(),
        };
        let conflicting_destination = IbcEndpoint {
            port_id: "transfer".into(),
            channel_id: "channel-34".into(),
        };

        instantiate(
            deps.as_mut(),
            env.clone(),
            mock_info("sender", &[]),
            InitMsg {
                default_timeout: 10000,
                gov_contract: env.contract.address.to_string(),
                channel: None,
            },
        )
        .unwrap();

        ibc_channel_connect(
            deps.as_mut(),
            env.clone(),
            cosmwasm_std::IbcChannelConnectMsg::OpenAck {
                channel: IbcChannel::new(
                    source_endpoint_1.clone(),
                    conflicting_destination.clone(),
                    cosmwasm_std::IbcOrder::Unordered,
                    "ucs01-0".to_string(),
                    "connection-1".to_string(),
                ),
                counterparty_version: "ucs01-0".to_string(),
            },
        )
        .unwrap();

        ibc_channel_connect(
            deps.as_mut(),
            env.clone(),
            cosmwasm_std::IbcChannelConnectMsg::OpenAck {
                channel: IbcChannel::new(
                    source_endpoint_2.clone(),
                    conflicting_destination.clone(),
                    cosmwasm_std::IbcOrder::Unordered,
                    "ucs01-0".to_string(),
                    "connection-1".to_string(),
                ),
                counterparty_version: "ucs01-0".to_string(),
            },
        )
        .unwrap();

        let (_, hash_1, _) = StatefulOnReceive {
            deps: deps.as_mut(),
        }
        .foreign_toggle(
            &Addr::unchecked("cosmos2contract"),
            &source_endpoint_1,
            "transfer/channel-34/denom",
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info(env.contract.address.as_str(), &[]),
            ExecuteMsg::RegisterDenom {
                local_endpoint: source_endpoint_1.clone(),
                denom: "transfer/channel-34/denom".to_string(),
                hash: hash_1.as_slice().into(),
            },
        )
        .unwrap();

        let (_, hash_2, _) = StatefulOnReceive {
            deps: deps.as_mut(),
        }
        .foreign_toggle(
            &Addr::unchecked("cosmos2contract"),
            &source_endpoint_2,
            "transfer/channel-34/denom",
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            mock_info(env.contract.address.as_str(), &[]),
            ExecuteMsg::RegisterDenom {
                local_endpoint: source_endpoint_2.clone(),
                denom: "transfer/channel-34/denom".to_string(),
                hash: hash_2.as_slice().into(),
            },
        )
        .unwrap();

        let denom = format!("factory/cosmos2contract/0x{}", hex::encode(hash_1));

        // We initially expect no outstanding for this token
        assert!(CHANNEL_STATE
            .load(&deps.storage, (&source_endpoint_2.channel_id, &denom))
            .is_err());

        let execute_msg = execute(
            deps.as_mut(),
            env.clone(),
            mock_info(
                env.contract.address.as_ref(),
                &[Coin {
                    denom: denom.clone(),
                    amount: Uint128::new(50),
                }],
            ),
            ExecuteMsg::Transfer(TransferMsg {
                channel: source_endpoint_2.channel_id.clone(),
                receiver: "1234123412341234".to_string(),
                timeout: None,
                memo: "memo".to_string(),
            }),
        )
        .unwrap();

        // We shouldn't burn but only increase the outstanding
        assert!(!matches!(
            execute_msg.messages[0].msg,
            CosmosMsg::Custom(TokenFactoryMsg::BurnTokens { .. })
        ));

        let outstanding: Uint512 = CHANNEL_STATE
            .load(&deps.storage, (&source_endpoint_2.channel_id, &denom))
            .unwrap()
            .outstanding;

        assert_eq!(outstanding, Into::<Uint512>::into(Uint128::new(50)));

        let denom = format!("factory/cosmos2contract/0x{}", hex::encode(hash_2));
        let execute_msg = execute(
            deps.as_mut(),
            env.clone(),
            mock_info(
                env.contract.address.as_ref(),
                &[Coin {
                    denom: denom.clone(),
                    amount: Uint128::new(50),
                }],
            ),
            ExecuteMsg::Transfer(TransferMsg {
                channel: source_endpoint_2.channel_id.clone(),
                receiver: "1234123412341234".to_string(),
                timeout: None,
                memo: "memo".to_string(),
            }),
        )
        .unwrap();

        assert_eq!(
            execute_msg.messages[0].msg,
            CosmosMsg::Custom(TokenFactoryMsg::BurnTokens {
                denom: denom.clone(),
                amount: Uint128::new(50),
                burn_from_address: "cosmos2contract".to_string()
            })
        );

        // we burn this time
        assert!(CHANNEL_STATE
            .load(&deps.storage, (&source_endpoint_2.channel_id, &denom))
            .is_err());
    }

    #[test]
    fn receive_transfer_destination_collision_yields_different_hashes() {
        let source_endpoint_1 = IbcEndpoint {
            port_id: "wasm.0xDEADC0DE".into(),
            channel_id: "channel-1".into(),
        };
        let source_endpoint_2 = IbcEndpoint {
            port_id: "wasm.0xDEADC0DE".into(),
            channel_id: "channel-2".into(),
        };
        let conflicting_destination = IbcEndpoint {
            port_id: "transfer".into(),
            channel_id: "channel-34".into(),
        };

        let CosmosMsg::Custom(TokenFactoryMsg::MintTokens { denom: denom1, .. }) =
            &TestOnReceive { toggle: true }
                .receive_phase1_transfer(
                    &Addr::unchecked("0xDEADC0DE"),
                    &source_endpoint_1,
                    &conflicting_destination,
                    "receiver",
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint256::from(100u128),
                    }],
                )
                .unwrap()[0]
        else {
            panic!("invalid msg");
        };

        let CosmosMsg::Custom(TokenFactoryMsg::MintTokens { denom: denom2, .. }) =
            &TestOnReceive { toggle: true }
                .receive_phase1_transfer(
                    &Addr::unchecked("0xDEADC0DE"),
                    &source_endpoint_2,
                    &conflicting_destination,
                    "receiver",
                    vec![TransferToken {
                        denom: "from-counterparty".into(),
                        amount: Uint256::from(100u128),
                    }],
                )
                .unwrap()[0]
        else {
            panic!("invalid msg");
        };

        assert_ne!(denom1, denom2);
    }

    #[test]
    fn receive_transfer_foreign() {
        let denom =
            hash_denom_str("wasm.0xDEADC0DE/channel-1/transfer/channel-34/from-counterparty");
        assert_eq!(
            TestOnReceive { toggle: true }.receive_phase1_transfer(
                &Addr::unchecked("0xDEADC0DE"),
                &IbcEndpoint {
                    port_id: "wasm.0xDEADC0DE".into(),
                    channel_id: "channel-1".into(),
                },
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-34".into(),
                },
                "receiver",
                vec![TransferToken {
                    denom: "from-counterparty".into(),
                    amount: Uint256::from(100u128)
                },],
            ),
            Ok(vec![TokenFactoryMsg::MintTokens {
                denom: format!("factory/0xDEADC0DE/{}", denom),
                amount: Uint128::from(100u128),
                mint_to_address: "receiver".into()
            }
            .into(),])
        );
    }

    #[test]
    fn receive_transfer_unwraps_local() {
        assert_eq!(
            TestOnReceive { toggle: true }.receive_phase1_transfer(
                &Addr::unchecked("0xDEADC0DE"),
                &IbcEndpoint {
                    port_id: "wasm.0xDEADC0DE".into(),
                    channel_id: "channel-1".into(),
                },
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-34".into(),
                },
                "receiver",
                vec![TransferToken {
                    denom: "wasm.0xDEADC0DE/channel-1/local-denom".into(),
                    amount: Uint256::from(119u128)
                }],
            ),
            Ok(vec![BankMsg::Send {
                to_address: "receiver".into(),
                amount: vec![Coin {
                    denom: "local-denom".into(),
                    amount: Uint128::from(119u128)
                }]
            }
            .into()])
        );
    }

    #[test]
    fn send_tokens_channel_remote_burn() {
        struct OnRemoteOnly;
        impl ForTokens for OnRemoteOnly {
            fn on_local(
                &mut self,
                _channel_id: &str,
                _denom: &str,
                _amount: Uint128,
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                Ok(vec![])
            }

            fn on_remote(
                &mut self,
                _channel_id: &str,
                denom: &str,
                amount: Uint128,
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                Ok(vec![TokenFactoryMsg::BurnTokens {
                    denom: denom.into(),
                    amount,
                    burn_from_address: "0xCAFEBABE".into(),
                }
                .into()])
            }
        }

        let denom1 = hash_denom_str("transfer-source/blabla/transfer/channel-1/remote-denom");
        let denom2 = hash_denom_str("transfer-source/blabla/transfer/channel-1/remote-denom2");

        assert_eq!(
            OnRemoteOnly.execute(
                &Addr::unchecked("0xCAFEBABE"),
                &IbcEndpoint {
                    port_id: "transfer-source".into(),
                    channel_id: "blabla".into()
                },
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-1".into()
                },
                vec![
                    TransferToken {
                        denom: "transfer/channel-1/remote-denom".into(),
                        amount: Uint256::from(119u128)
                    },
                    TransferToken {
                        denom: "transfer/channel-2/remote-denom".into(),
                        amount: Uint256::from(10u128)
                    },
                    TransferToken {
                        denom: "transfer/channel-1/remote-denom2".into(),
                        amount: Uint256::from(129u128)
                    },
                ],
            ),
            Ok(vec![
                TokenFactoryMsg::BurnTokens {
                    denom: format!("factory/0xCAFEBABE/{}", denom1),
                    amount: Uint128::from(119u128),
                    burn_from_address: "0xCAFEBABE".into()
                }
                .into(),
                TokenFactoryMsg::BurnTokens {
                    denom: format!("factory/0xCAFEBABE/{}", denom2),
                    amount: Uint128::from(129u128),
                    burn_from_address: "0xCAFEBABE".into()
                }
                .into()
            ])
        );
    }

    #[test]
    fn send_tokens_channel_local_escrow() {
        struct OnLocalOnly {
            total: Uint128,
        }
        impl ForTokens for OnLocalOnly {
            fn on_local(
                &mut self,
                _channel_id: &str,
                _denom: &str,
                amount: Uint128,
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                self.total += amount;
                Ok(Default::default())
            }

            fn on_remote(
                &mut self,
                _channel_id: &str,
                _denom: &str,
                _amount: Uint128,
            ) -> Result<
                Vec<cosmwasm_std::CosmosMsg<token_factory_api::TokenFactoryMsg>>,
                crate::error::ContractError,
            > {
                todo!()
            }
        }
        let mut state = OnLocalOnly { total: 0u8.into() };
        assert_eq!(
            state.execute(
                &Addr::unchecked("0xCAFEBABE"),
                &IbcEndpoint {
                    port_id: "transfer-source".into(),
                    channel_id: "blabla".into()
                },
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-1".into()
                },
                vec![
                    TransferToken {
                        denom: "transfer/channel-2/remote-denom".into(),
                        amount: Uint256::from(119u128)
                    },
                    TransferToken {
                        denom: "transfer/channel-2/remote-denom2".into(),
                        amount: Uint256::from(129u128)
                    }
                ],
            ),
            Ok(vec![])
        );
        assert_eq!(state.total, Uint128::from(119u128 + 129u128));
    }

    #[test]
    fn normalize_identity() {
        assert_eq!(
            normalize_for_ibc_transfer(
                |_| Ok(Some("transfer/channel-331/from-counterparty".into())),
                "0xDEADC0DE",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-332".into()
                },
                TransferToken {
                    denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                    amount: Uint256::MAX
                }
            ),
            Ok(TransferToken {
                denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                amount: Uint256::MAX
            })
        );
        assert_eq!(
            normalize_for_ibc_transfer(
                |_| Ok(Some("transfer1/channel-332/from-counterparty".into())),
                "0xDEADC0DE",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-332".into()
                },
                TransferToken {
                    denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                    amount: Uint256::MAX
                }
            ),
            Ok(TransferToken {
                denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                amount: Uint256::MAX
            })
        );
    }

    #[test]
    fn normalize_strip() {
        assert_eq!(
            normalize_for_ibc_transfer(
                |_| Ok(Some("transfer/channel-332/blabla-1".into())),
                "0xDEADC0DE",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-332".into()
                },
                TransferToken {
                    denom: "factory/0xDEADC0DE/0xaf30fd00576e1d27471a4d2b0c0487dc6876e0589e".into(),
                    amount: Uint256::MAX
                }
            ),
            Ok(TransferToken {
                denom: "transfer/channel-332/blabla-1".into(),
                amount: Uint256::MAX
            })
        );
    }
}
