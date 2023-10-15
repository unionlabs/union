use cosmwasm_std::{
    Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, IbcEndpoint, IbcOrder, MessageInfo, Uint128,
    Uint512,
};
use token_factory_api::{TokenFactoryMsg, TokenMsg};
use ucs01_relay_api::{
    protocol::TransferProtocol,
    types::{
        make_foreign_denom, DenomOrigin, Ics20Ack, Ics20Packet, TransferToken, Ucs01Ack,
        Ucs01TransferPacket,
    },
};

use crate::{
    error::ContractError,
    state::{ChannelInfo, CHANNEL_STATE, FOREIGN_TOKEN_CREATED},
};

pub fn protocol_ordering(version: &str) -> Option<IbcOrder> {
    match version {
        Ics20Protocol::VERSION => Some(Ics20Protocol::ORDERING),
        Ucs01Protocol::VERSION => Some(Ucs01Protocol::ORDERING),
        _ => None,
    }
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
        (channel_id, &denom),
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

trait OnReceive {
    fn foreign_toggle(&mut self, denom: &str) -> Result<bool, ContractError>;

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
                        let factory_denom =
                            format!("factory/{}/{}", contract_address, foreign_denom);
                        let exists = self.foreign_toggle(&factory_denom)?;
                        Ok(if exists {
                            vec![TokenMsg::MintTokens {
                                denom: factory_denom,
                                amount,
                                mint_to_address: receiver.to_string(),
                            }
                            .into()]
                        } else {
                            vec![
                                TokenMsg::CreateDenom {
                                    subdenom: foreign_denom.clone(),
                                    metadata: None,
                                }
                                .into(),
                                TokenMsg::MintTokens {
                                    denom: factory_denom,
                                    amount,
                                    mint_to_address: receiver.to_string(),
                                }
                                .into(),
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
    fn foreign_toggle(&mut self, denom: &str) -> Result<bool, ContractError> {
        let exists = FOREIGN_TOKEN_CREATED.has(self.deps.storage, denom);
        FOREIGN_TOKEN_CREATED.save(self.deps.storage, denom, &())?;
        Ok(exists)
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
        channel_id: &str,
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
                    let foreign_denom = make_foreign_denom(counterparty_endpoint, denom);
                    let factory_denom = format!("factory/{}/{}", contract_address, foreign_denom);
                    messages.append(&mut self.on_remote(channel_id, &factory_denom, amount)?);
                }
                DenomOrigin::Remote { denom } => {
                    messages.append(&mut self.on_local(channel_id, denom, amount)?);
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
        Ok(vec![TokenMsg::BurnTokens {
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
        Ok(vec![TokenMsg::MintTokens {
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
        Ics20Ack::Result(b"1".into())
    }

    fn ack_failure(error: String) -> Self::Ack {
        Ics20Ack::Error(error)
    }

    fn send_tokens(
        &mut self,
        _sender: &str,
        _receiver: &str,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulSendTokens {
            deps: self.common.deps.branch(),
            contract_address: self.common.env.contract.address.to_string(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint.channel_id,
            &self.common.channel.counterparty_endpoint,
            tokens,
        )
    }

    fn send_tokens_success(
        &mut self,
        _sender: &str,
        _receiver: &str,
        _tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        Ok(Default::default())
    }

    fn send_tokens_failure(
        &mut self,
        sender: &str,
        _receiver: &str,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulRefundTokens {
            deps: self.common.deps.branch(),
            receiver: sender.into(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint.channel_id,
            &self.common.channel.counterparty_endpoint,
            tokens,
        )
    }

    fn receive_transfer(
        &mut self,
        receiver: &str,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, ContractError> {
        StatefulOnReceive {
            deps: self.common.deps.branch(),
        }
        .receive_phase1_transfer(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            receiver,
            tokens,
        )
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
        _sender: &str,
        _receiver: &str,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulSendTokens {
            deps: self.common.deps.branch(),
            contract_address: self.common.env.contract.address.to_string(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint.channel_id,
            &self.common.channel.counterparty_endpoint,
            tokens,
        )
    }

    fn send_tokens_success(
        &mut self,
        _sender: &str,
        _receiver: &str,
        _tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        Ok(Default::default())
    }

    fn send_tokens_failure(
        &mut self,
        sender: &str,
        _receiver: &str,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error> {
        StatefulRefundTokens {
            deps: self.common.deps.branch(),
            receiver: sender.into(),
        }
        .execute(
            &self.common.env.contract.address,
            &self.common.channel.endpoint.channel_id,
            &self.common.channel.counterparty_endpoint,
            tokens,
        )
    }

    fn receive_transfer(
        &mut self,
        receiver: &str,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, ContractError> {
        StatefulOnReceive {
            deps: self.common.deps.branch(),
        }
        .receive_phase1_transfer(
            &self.common.env.contract.address,
            &self.common.channel.endpoint,
            &self.common.channel.counterparty_endpoint,
            receiver,
            tokens,
        )
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, BankMsg, Coin, IbcEndpoint, Uint128, Uint256};
    use token_factory_api::TokenMsg;
    use ucs01_relay_api::types::TransferToken;

    use super::{ForTokens, OnReceive};

    struct TestOnReceive {
        toggle: bool,
    }
    impl OnReceive for TestOnReceive {
        fn foreign_toggle(&mut self, _denom: &str) -> Result<bool, crate::error::ContractError> {
            Ok(self.toggle)
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
                TokenMsg::CreateDenom {
                    subdenom: "transfer/channel-34/from-counterparty".into(),
                    metadata: None
                }
                .into(),
                TokenMsg::MintTokens {
                    denom: "factory/0xDEADC0DE/transfer/channel-34/from-counterparty".into(),
                    amount: Uint128::from(100u128),
                    mint_to_address: "receiver".into()
                }
                .into(),
            ])
        );
    }

    #[test]
    fn receive_transfer_foreign() {
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
            Ok(vec![TokenMsg::MintTokens {
                denom: "factory/0xDEADC0DE/transfer/channel-34/from-counterparty".into(),
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
    fn send_tokens_burn_channel_remote() {
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
                todo!()
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
                Ok(vec![TokenMsg::BurnTokens {
                    denom: denom.into(),
                    amount,
                    burn_from_address: "0xCAFEBABE".into(),
                }
                .into()])
            }
        }
        assert_eq!(
            OnRemoteOnly.execute(
                &Addr::unchecked("0xCAFEBABE"),
                "blabla",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-1".into()
                },
                vec![TransferToken {
                    denom: "transfer/channel-1/remote-denom".into(),
                    amount: Uint256::from(119u128)
                }],
            ),
            Ok(vec![TokenMsg::BurnTokens {
                denom: "factory/0xCAFEBABE/transfer/channel-1/remote-denom".into(),
                amount: Uint128::from(119u128),
                burn_from_address: "0xCAFEBABE".into()
            }
            .into()])
        );
    }

    #[test]
    fn send_tokens_escrow_local() {
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
                "blabla",
                &IbcEndpoint {
                    port_id: "transfer".into(),
                    channel_id: "channel-1".into()
                },
                vec![TransferToken {
                    denom: "transfer/channel-2/remote-denom".into(),
                    amount: Uint256::from(119u128)
                }],
            ),
            Ok(vec![])
        );
        assert_eq!(state.total, Uint128::from(119u128));
    }
}
