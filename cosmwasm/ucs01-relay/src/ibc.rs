#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannel, IbcChannelCloseMsg,
    IbcChannelConnectMsg, IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, MessageInfo, Reply, Response, SubMsgResult,
};
use prost::{Message, Name};
use protos::cosmwasm::wasm::v1::MsgIbcSendResponse;
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::{
    middleware::InFlightPfmPacket,
    protocol::{TransferProtocol, IBC_SEND_ID},
};

pub type IbcResponse = IbcBasicResponse<TokenFactoryMsg>;

use crate::{
    error::ContractError,
    protocol::{protocol_ordering, Ics20Protocol, ProtocolCommon, Ucs01Protocol},
    state::{ChannelInfo, PfmRefundPacketKey, CHANNEL_INFO, IN_FLIGHT_PFM_PACKETS},
};

fn to_response<T>(
    IbcReceiveResponse {
        acknowledgement,
        messages,
        attributes,
        events,
        ..
    }: IbcReceiveResponse<T>,
) -> Response<T> {
    let response = Response::<T>::new()
        .add_submessages(messages)
        .add_attributes(attributes)
        .add_events(events);

    if let Some(ack) = acknowledgement {
        response.set_data(ack)
    } else {
        response
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(
    deps: DepsMut,
    _: Env,
    reply: Reply,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match (reply.id, reply.result) {
        // RECEIVE_REPLY_ID is associated with submessages emitted during handling of `ibc_packet_receive`
        (Ics20Protocol::RECEIVE_REPLY_ID, SubMsgResult::Err(err)) => {
            Ok(to_response(Ics20Protocol::receive_error(err)))
        }
        (Ucs01Protocol::RECEIVE_REPLY_ID, SubMsgResult::Err(err)) => {
            Ok(to_response(Ucs01Protocol::receive_error(err)))
        }
        // IBC_SEND_ID is associated with submessages emitted during handling of `send`, which is called via `execute_transfer`, which is used both in PFM and non-PFM contexts
        (IBC_SEND_ID, SubMsgResult::Ok(value)) => {
            // this means this is not pfm
            if reply.payload.is_empty() {
                return Ok(Response::new());
            }

            let msg_response = value
                .msg_responses
                .iter()
                .find(|msg_response| msg_response.type_url == MsgIbcSendResponse::type_url())
                .expect("type url is correct and exists");

            let send_response =
                MsgIbcSendResponse::decode(msg_response.value.as_slice()).expect("is type url");

            let in_flight_packet =
                serde_json_wasm::from_slice::<InFlightPfmPacket>(reply.payload.as_slice())
                    .expect("binary is type");

            let refund_packet_key = PfmRefundPacketKey {
                channel_id: in_flight_packet.forward_src_channel_id.clone(),
                port_id: in_flight_packet.forward_src_port_id.clone(),
                sequence: send_response.sequence,
            };

            IN_FLIGHT_PFM_PACKETS
                .save(deps.storage, refund_packet_key.clone(), &in_flight_packet)
                .expect("infallible update");

            Ok(
                Response::new()
                    .add_event(in_flight_packet.create_hop_event(send_response.sequence)),
            )
        }
        (IBC_SEND_ID, SubMsgResult::Err(err)) => {
            // this means this is not pfm
            if reply.payload.is_empty() {
                return Err(ContractError::PfmSendPacketError { err });
            }

            // decode the payload to figure out the source channel
            let in_flight_packet =
                serde_json_wasm::from_slice::<InFlightPfmPacket>(reply.payload.as_slice())
                    .expect("binary is type");

            match &*in_flight_packet.origin_protocol_version {
                Ucs01Protocol::VERSION => Ok(to_response(Ucs01Protocol::receive_error(err))),
                Ics20Protocol::VERSION => Ok(to_response(Ics20Protocol::receive_error(err))),
                // in_flight_packet.origin_protocol_version is only ever set by us, so if it is set incorrectly then it is a bug
                version => unreachable!("unknown protocol version: {version}"),
            }
        }
        (_, result) => Err(ContractError::UnknownReply {
            id: reply.id,
            variant: result,
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<Option<Ibc3ChannelOpenResponse>, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(None)
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// record the channel in CHANNEL_INFO
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcResponse, ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        endpoint: channel.endpoint,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
        protocol_version: channel.version,
    };
    CHANNEL_INFO.save(deps.storage, &info.endpoint.channel_id, &info)?;

    Ok(IbcResponse::default())
}

pub(crate) fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    let channel_ordering =
        protocol_ordering(&channel.version).ok_or(ContractError::UnknownProtocol {
            channel_id: channel.endpoint.channel_id.clone(),
            protocol_version: channel.version.clone(),
        })?;
    if let Some(version) = counterparty_version {
        if protocol_ordering(version).is_none() {
            return Err(ContractError::UnknownProtocol {
                channel_id: channel.endpoint.channel_id.clone(),
                protocol_version: version.to_string(),
            });
        }
        if version != channel.version {
            return Err(ContractError::ProtocolMismatch {
                channel_id: channel.endpoint.channel_id.clone(),
                protocol_version: channel.version.clone(),
                counterparty_protocol_version: version.to_string(),
            });
        }
    }
    if channel.order != channel_ordering {
        return Err(ContractError::InvalidChannelOrdering {
            expected: channel_ordering,
            actual: channel.order.clone(),
        });
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcResponse, ContractError> {
    Err(ContractError::Unauthorized)
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// Check to see if we have any balance here
/// We should not return an error if possible, but rather an acknowledgement of failure
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse<TokenFactoryMsg>, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.packet.dest.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel_info.protocol_version.as_str() {
        Ics20Protocol::VERSION => Ok((Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        })
        .receive(msg.packet)),
        Ucs01Protocol::VERSION => Ok((Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        })
        .receive(msg.packet)),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// check if success or failure and update balance, or return funds
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcResponse, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.original_packet.src.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer.clone(),
        funds: Default::default(),
    };

    match channel_info.protocol_version.as_str() {
        Ics20Protocol::VERSION => (Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        })
        .send_ack(msg),
        Ucs01Protocol::VERSION => (Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        })
        .send_ack(msg),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.original_packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// return fund to original sender (same as failure in ibc_packet_ack)
pub fn ibc_packet_timeout(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcResponse, ContractError> {
    let channel_info = CHANNEL_INFO.load(deps.storage, &msg.packet.src.channel_id)?;

    let info = MessageInfo {
        sender: msg.relayer,
        funds: Default::default(),
    };

    match channel_info.protocol_version.as_str() {
        Ics20Protocol::VERSION => (Ics20Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        })
        .send_timeout(msg.packet),
        Ucs01Protocol::VERSION => (Ucs01Protocol {
            common: ProtocolCommon {
                deps,
                env,
                info,
                channel: channel_info,
            },
        })
        .send_timeout(msg.packet),
        v => Err(ContractError::UnknownProtocol {
            channel_id: msg.packet.dest.channel_id,
            protocol_version: v.into(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coin, coins,
        testing::{MockApi, MockStorage},
        Addr, Api, Binary, BlockInfo, Coin, CustomMsg, CustomQuery, Empty, IbcEndpoint, IbcOrder,
        IbcTimeout, Querier, Storage, Uint128,
    };
    use cw_multi_test::{
        error::AnyResult,
        ibc::{
            relayer::{create_channel, create_connection, ChannelCreationResult},
            types::IbcPacketData,
            IbcSimpleModule,
        },
        App, AppBuilder, AppResponse, BankKeeper, BankSudo, ContractWrapper, CosmosRouter,
        DistributionKeeper, Executor, GovFailingModule, Module, StakeKeeper, StargateFailing,
        SudoMsg, WasmKeeper,
    };
    use serde::de::DeserializeOwned;
    use sha2::{Digest, Sha256};
    use ucs01_relay_api::{
        protocol::{TransferProtocol},
        types::make_foreign_denom,
    };

    use super::*;
    use crate::{
        contract::{execute, instantiate, query},
        msg::{ExecuteMsg, InstantiateMsg, TransferMsg},
    };
    fn create_app(prefix: &'static str) -> BasicApp<TokenFactoryMsg> {
        let application = cw_multi_test::BasicAppBuilder::<TokenFactoryMsg, Empty>::new_custom();
        let application: BasicAppBuilder<TokenFactoryMsg, Empty> =
            unsafe { std::mem::transmute(application) };
        application
            .with_custom(MyCustomModule)
            .with_api(MockApi::default().with_prefix(prefix))
            .with_ibc(IbcSimpleModule)
            .build(|_, _, _| {})
    }

    pub struct MyCustomModule;

    pub type BasicApp<ExecC = Empty, QueryC = Empty> = App<
        BankKeeper,
        MockApi,
        MockStorage,
        MyCustomModule,
        WasmKeeper<ExecC, QueryC>,
        StakeKeeper,
        DistributionKeeper,
        IbcSimpleModule,
        GovFailingModule,
        StargateFailing,
    >;

    pub type BasicAppBuilder<ExecC, QueryC> = AppBuilder<
        BankKeeper,
        MockApi,
        MockStorage,
        MyCustomModule,
        WasmKeeper<ExecC, QueryC>,
        StakeKeeper,
        DistributionKeeper,
        IbcSimpleModule,
        GovFailingModule,
        StargateFailing,
    >;

    impl Module for MyCustomModule {
        type ExecT = TokenFactoryMsg;
        type QueryT = Empty;
        type SudoT = Empty;

        fn execute<ExecC, QueryC>(
            &self,
            api: &dyn Api,
            storage: &mut dyn Storage,
            router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
            block: &BlockInfo,
            _sender: Addr,
            msg: Self::ExecT,
        ) -> AnyResult<AppResponse>
        where
            ExecC: CustomMsg + DeserializeOwned + 'static,
            QueryC: CustomQuery + DeserializeOwned + 'static,
        {
            match msg {
                TokenFactoryMsg::MintTokens {
                    denom,
                    amount,
                    mint_to_address,
                } => router.sudo(
                    api,
                    storage,
                    block,
                    SudoMsg::Bank(BankSudo::Mint {
                        amount: coins(amount.into(), denom),
                        to_address: mint_to_address,
                    }),
                ),
                // TokenFactoryMsg::BurnTokens { denom, amount, burn_from_address } =>
                // {
                //     println!("burning tokens, denom: {}, amount: {}, burn_from_address: {}", denom, amount, burn_from_address);
                //     router.sudo(
                //         api,
                //         storage,
                //         block,
                //         SudoMsg::Bank(BankSudo::Burn {
                //             amount: coins(amount.into(), denom),
                //             from_address: burn_from_address,
                //         }),
                //     )
                // }
                _ => Ok(AppResponse::default()),
            }
        }

        fn query(
            &self,
            _api: &dyn Api,
            _storage: &dyn Storage,
            _querier: &dyn Querier,
            _block: &BlockInfo,
            _request: Self::QueryT,
        ) -> AnyResult<Binary> {
            unimplemented!()
        }

        fn sudo<ExecC, QueryC>(
            &self,
            _api: &dyn Api,
            _storage: &mut dyn Storage,
            _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
            _block: &BlockInfo,
            _msg: Self::SudoT,
        ) -> AnyResult<AppResponse>
        where
            ExecC: CustomMsg + DeserializeOwned + 'static,
            QueryC: CustomQuery + DeserializeOwned + 'static,
        {
            unimplemented!()
        }
    }

    fn store_and_instantiate_contract(
        app: &mut BasicApp<TokenFactoryMsg>,
        creator_addr: &str,
        funds: Vec<Coin>,
        gov_contract: &str,
    ) -> (String, Addr) {
        let creator_addr = app.api().addr_make(creator_addr);
        let gov_contract = app.api().addr_make(gov_contract);

        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &creator_addr, funds.clone())
                .unwrap();
        });

        let contract: Box<_> = Box::new(
            ContractWrapper::new(execute, instantiate, query)
                .with_reply(reply)
                .with_ibc(
                    ibc_channel_open,
                    ibc_channel_connect,
                    ibc_channel_close,
                    ibc_packet_receive,
                    ibc_packet_ack,
                    ibc_packet_timeout,
                ),
        );

        let code_id = app.store_code_with_creator(creator_addr.clone(), contract);
        assert_eq!(1, code_id);

        let instantiate_msg = InstantiateMsg {
            channel: None,
            default_timeout: 1000,
            gov_contract: gov_contract.to_string(),
        };

        let contract_addr = app
            .instantiate_contract(
                code_id,
                creator_addr.clone(),
                &instantiate_msg,
                &[],
                "contract",
                None,
            )
            .unwrap();

        (creator_addr.to_string(), contract_addr)
    }

    fn create_ibc_connection_and_channel(
        app1: &mut BasicApp<TokenFactoryMsg>,
        app2: &mut BasicApp<TokenFactoryMsg>,
        contract_addr1: &Addr,
        contract_addr2: &Addr,
    ) -> (String, String, String, String, String, String) {
        let port1 = "wasm.".to_string() + contract_addr1.as_str();
        let port2 = "wasm.".to_string() + contract_addr2.as_str();

        let (src_connection_id, dest_connection_id) = create_connection(app1, app2).unwrap();

        let ChannelCreationResult {
            src_channel,
            dst_channel,
            ..
        } = create_channel(
            app1,
            app2,
            src_connection_id.clone(),
            port1.clone(),
            port2.clone(),
            Ics20Protocol::VERSION.to_string(),
            IbcOrder::Unordered,
        )
        .unwrap();

        (
            src_connection_id,
            dest_connection_id,
            src_channel,
            dst_channel,
            port1,
            port2,
        )
    }

    fn create_factory_denom(contract_addr: &Addr, foreign_denom: &str) -> String {
        let hashed_denom = {
            let mut hasher = Sha256::new();
            hasher.update(foreign_denom);
            hasher.finalize()
        };

        let normalized_denom = format!("0x{}", hex::encode(&hashed_denom[..21]));
        format!("factory/{}/{}", contract_addr, normalized_denom)
    }
    fn send_ibc_packet(
        app: &mut BasicApp<TokenFactoryMsg>,
        sender_addr: Addr,
        contract_addr: Addr,
        msg: ExecuteMsg,
        funds: Vec<Coin>,
    ) -> AppResponse {
        app.execute_contract(sender_addr, contract_addr, &msg, &funds)
            .unwrap()
    }

    fn receive_ibc_packet(
        app: &mut BasicApp<TokenFactoryMsg>,
        src_port_id: String,
        src_channel_id: String,
        dst_port_id: String,
        dst_channel_id: String,
        packet_data: Vec<u8>,
        timeout: IbcTimeout,
    ) -> AppResponse {
        let res = app.sudo(cw_multi_test::SudoMsg::Ibc(
            cw_multi_test::ibc::IbcPacketRelayingMsg::Receive {
                packet: IbcPacketData {
                    ack: None,
                    src_port_id,
                    src_channel_id,
                    dst_port_id,
                    dst_channel_id,
                    sequence: 1,
                    data: packet_data.into(),
                    timeout,
                },
            },
        ));
        res.unwrap()
    }

    fn check_balance(
        app: &BasicApp<TokenFactoryMsg>,
        addr: &str,
        denom: &str,
        expected_balance: Uint128,
        error_msg: &str,
    ) {
        let balance_creator = app.wrap().query_balance(addr, denom).unwrap();

        assert_eq!(
            balance_creator.amount, expected_balance,
            "Balance of {} should be {} {}. Error: {}",
            addr, expected_balance, denom, error_msg
        );
    }

    fn transfer_tokens(
        src_app: &mut BasicApp<TokenFactoryMsg>,
        dst_app: &mut BasicApp<TokenFactoryMsg>,
        src_creator_addr: &str,
        src_contract_addr: &Addr,
        dst_contract_addr: &Addr,
        src_port: &str,
        dst_port: &str,
        src_channel: &str,
        dst_channel: &str,
        denom: &str,
        amount: Uint128,
    ) {
        // Transfer message
        let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
            channel: src_channel.to_string(),
            receiver: dst_contract_addr.to_string(),
            timeout: None,
            memo: "".to_string(),
        });

        // Execute transfer
        let res = send_ibc_packet(
            src_app,
            Addr::unchecked(src_creator_addr.to_string()),
            src_contract_addr.clone(),
            transfer_msg,
            vec![coin(amount.u128(), denom)],
        );

        let packet_data_hex = res
            .events
            .iter()
            .find(|event| event.ty == "send_packet")
            .and_then(|event| {
                event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == "packet_data_hex")
            })
            .map(|attr| attr.value.clone())
            .expect("packet_data_hex should be present");

        let packet_data_bytes =
            hex::decode(packet_data_hex).expect("Decoding packet_data_hex should succeed");

        let channel_info = ChannelInfo {
            endpoint: IbcEndpoint {
                port_id: dst_port.to_string(),
                channel_id: dst_channel.to_string(),
            },
            counterparty_endpoint: IbcEndpoint {
                port_id: src_port.to_string(),
                channel_id: src_channel.to_string(),
            },
            connection_id: "connection-1".to_string(),
            protocol_version: Ics20Protocol::VERSION.to_string(),
        };

        CHANNEL_INFO
            .save(dst_app.storage_mut(), dst_channel, &channel_info)
            .unwrap();

        let timeout = IbcTimeout::with_timestamp(dst_app.block_info().time.plus_seconds(60));

        receive_ibc_packet(
            dst_app,
            src_port.to_string(),
            src_channel.to_string(),
            dst_port.to_string(),
            dst_channel.to_string(),
            packet_data_bytes,
            timeout,
        );
    }

    #[test]
    fn simple_transfer() {
        let funds = vec![coin(100_000, "muno")];
        let denom = "muno";

        // Create two apps
        let mut src_app = create_app("src");
        let mut dst_app = create_app("dst");

        // Store and instantiate contracts in both apps
        let (src_creator_addr, src_contract_addr) = store_and_instantiate_contract(
            &mut src_app,
            "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            funds.clone(),
            "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, dst_contract_addr) = store_and_instantiate_contract(
            &mut dst_app,
            "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            vec![],
            "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        // Create IBC connection and channel
        let (_, _, src_channel, dst_channel, src_port, dst_port) =
            create_ibc_connection_and_channel(
                &mut src_app,
                &mut dst_app,
                &src_contract_addr,
                &dst_contract_addr,
            );

        let endpoint = IbcEndpoint {
            port_id: dst_port.clone(),
            channel_id: dst_channel.clone(),
        };
        let foreign_denom = make_foreign_denom(&endpoint, denom);
        let factory_denom: String = create_factory_denom(&dst_contract_addr, &foreign_denom);

        // Check initial balances
        check_balance(
            &src_app,
            &src_creator_addr,
            denom,
            Uint128::new(100_000),
            "initial source balance is not 100k",
        );
        check_balance(
            &dst_app,
            &dst_contract_addr.to_string(),
            &factory_denom,
            Uint128::new(0),
            "initial destination balance is not 0",
        );

        // Transfer tokens from src_app to dst_app
        transfer_tokens(
            &mut src_app,
            &mut dst_app,
            &src_creator_addr,
            &src_contract_addr,
            &dst_contract_addr,
            &src_port,
            &dst_port,
            &src_channel,
            &dst_channel,
            denom,
            Uint128::new(100_000),
        );

        // Check balances after transfer
        check_balance(
            &src_app,
            &src_creator_addr,
            denom,
            Uint128::new(0),
            "source balance is not 0 after transfer",
        );
        check_balance(
            &dst_app,
            &dst_contract_addr.to_string(),
            &factory_denom,
            Uint128::new(100_000),
            "destination balance is not 100k after transfer",
        );
    }
    #[test]
    fn send_back_wrapped_tokens() {
        // Setup apps, instantiate contracts, create connection and channels
        let funds = vec![coin(100_000, "muno")];
        let denom = "muno";

        let mut src_app = create_app("src");
        let mut dst_app = create_app("dst");

        let (src_creator_addr, src_contract_addr) = store_and_instantiate_contract(
            &mut src_app,
            "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            funds.clone(),
            "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, dst_contract_addr) = store_and_instantiate_contract(
            &mut dst_app,
            "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            vec![],
            "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, _, src_channel, dst_channel, src_port, dst_port) =
            create_ibc_connection_and_channel(
                &mut src_app,
                &mut dst_app,
                &src_contract_addr,
                &dst_contract_addr,
            );

        let endpoint = IbcEndpoint {
            port_id: dst_port.clone(),
            channel_id: dst_channel.clone(),
        };
        let foreign_denom = make_foreign_denom(&endpoint, denom);
        let factory_denom: String = create_factory_denom(&dst_contract_addr, &foreign_denom);

        // Perform initial transfer from src_app to dst_app
        transfer_tokens(
            &mut src_app,
            &mut dst_app,
            &src_creator_addr,
            &src_contract_addr,
            &dst_contract_addr,
            &src_port,
            &dst_port,
            &src_channel,
            &dst_channel,
            denom,
            Uint128::new(100_000),
        );

        // Check balances after transfer
        check_balance(
            &src_app,
            &src_creator_addr,
            denom,
            Uint128::new(0),
            "source balance is not 0 after transfer",
        );
        check_balance(
            &dst_app,
            &dst_contract_addr.to_string(),
            &factory_denom,
            Uint128::new(100_000),
            "destination balance is not 100k after transfer",
        );

        // Perform transfer back from dst_app to src_app
        transfer_tokens(
            &mut dst_app,
            &mut src_app,
            &dst_contract_addr.as_str(),
            &dst_contract_addr,
            &Addr::unchecked(src_creator_addr.clone()),
            &dst_port,
            &src_port,
            &dst_channel,
            &src_channel,
            &factory_denom,
            Uint128::new(100_000),
        );

        // Check balances after transfer
        check_balance(
            &src_app,
            &src_creator_addr,
            denom,
            Uint128::new(100_000),
            "source balance is not 100k after transferring wrapped token back",
        );

        // it can't be 0 because we don't have burn functionality so we can't burn
        // the locked tokens in app2, lets assume it will work anyway.
        // check_balance(&dst_app, &dst_contract_addr, &factory_denom, Uint128::new(100_000));
    }

    #[test]
    fn test_pfm_valid_memo() {
        let funds = vec![coin(100_000, "muno")];
        let mut src_app = create_app("src");
        let mut dst_app = create_app("dst");
        let mut fwd_app = create_app("fwd");

        let (src_creator_addr, src_contract_addr) = store_and_instantiate_contract(
            &mut src_app,
            "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            funds.clone(),
            "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, dst_contract_addr) = store_and_instantiate_contract(
            &mut dst_app,
            "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            vec![],
            "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, fwd_contract_addr) = store_and_instantiate_contract(
            &mut fwd_app,
            "fwd1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            vec![],
            "fwd1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, _, src_channel, dst_channel, src_port, dst_port) =
            create_ibc_connection_and_channel(
                &mut src_app,
                &mut dst_app,
                &src_contract_addr,
                &dst_contract_addr,
            );

        let (_, _, dst_src_channel, fwd_dst_channel, dst_src_port, fwd_dst_port) =
            create_ibc_connection_and_channel(
                &mut dst_app,
                &mut fwd_app,
                &dst_contract_addr,
                &fwd_contract_addr,
            );

        // Save the channel info for forward channels
        let dst_channel_info = ChannelInfo {
            endpoint: IbcEndpoint {
                port_id: dst_src_port.clone(),
                channel_id: dst_src_channel.clone(),
            },
            counterparty_endpoint: IbcEndpoint {
                port_id: src_port.clone(),
                channel_id: src_channel.clone(),
            },
            connection_id: "connection-0".to_string(),
            protocol_version: Ics20Protocol::VERSION.to_string(),
        };

        CHANNEL_INFO
            .save(dst_app.storage_mut(), &dst_src_channel, &dst_channel_info)
            .unwrap();

        let fwd_channel_info = ChannelInfo {
            endpoint: IbcEndpoint {
                port_id: fwd_dst_port.clone(),
                channel_id: fwd_dst_channel.clone(),
            },
            counterparty_endpoint: IbcEndpoint {
                port_id: dst_src_port.clone(),
                channel_id: dst_src_channel.clone(),
            },
            connection_id: "connection-0".to_string(),
            protocol_version: Ics20Protocol::VERSION.to_string(),
        };

        CHANNEL_INFO
            .save(fwd_app.storage_mut(), &fwd_dst_channel, &fwd_channel_info)
            .unwrap();

        // Create memo string with forwarding information dynamically
        let memo: String = format!(
            "{{\"forward\":{{\"receiver\":\"{}\",\"port\":\"{}\",\"channel\":\"{}\"}}}}",
            fwd_contract_addr, fwd_dst_port, fwd_dst_channel
        );

        let endpoint = IbcEndpoint {
            port_id: dst_port.clone(),
            channel_id: dst_channel.clone(),
        };
        let foreign_denom = make_foreign_denom(&endpoint, "muno");
        let factory_denom: String = create_factory_denom(&dst_contract_addr, &foreign_denom);

        let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
            channel: src_channel.clone(),
            receiver: dst_contract_addr.to_string(),
            timeout: None,
            memo: memo.to_string(),
        });

        let res = send_ibc_packet(
            &mut src_app,
            Addr::unchecked(src_creator_addr.clone()),
            src_contract_addr.clone(),
            transfer_msg,
            vec![coin(100_000, "muno")],
        );

        let packet_data_hex = res
            .events
            .iter()
            .find(|event| event.ty == "send_packet")
            .and_then(|event| {
                event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == "packet_data_hex")
            })
            .map(|attr| attr.value.clone())
            .expect("packet_data_hex should be present");

        let packet_data_bytes =
            hex::decode(packet_data_hex).expect("Decoding packet_data_hex should succeed");

        let timeout = IbcTimeout::with_timestamp(dst_app.block_info().time.plus_seconds(60));

        receive_ibc_packet(
            &mut dst_app,
            src_port.clone(),
            src_channel.clone(),
            dst_port.clone(),
            dst_channel.clone(),
            packet_data_bytes.clone(),
            timeout.clone(),
        );

        check_balance(
            &dst_app,
            dst_contract_addr.as_str(),
            &factory_denom,
            Uint128::new(100_000),
            "destination balance on hop chain is not 100k right before PFM execution",
        );

        receive_ibc_packet(
            &mut fwd_app,
            dst_src_port.clone(),
            dst_src_channel.clone(),
            fwd_dst_port.clone(),
            fwd_dst_channel.clone(),
            packet_data_bytes,
            timeout,
        );

        let endpoint_forwarded = IbcEndpoint {
            port_id: fwd_dst_port.clone(),
            channel_id: fwd_dst_channel.clone(),
        };
        let foreign_denom_forwarded = make_foreign_denom(&endpoint_forwarded, "muno");
        let factory_denom_forwarded =
            create_factory_denom(&fwd_contract_addr, &foreign_denom_forwarded);

        check_balance(
            &fwd_app,
            &fwd_contract_addr.to_string(),
            &factory_denom_forwarded,
            Uint128::new(100_000),
            "destination balance is not 100k after PFM",
        );
    }

    #[test]
    fn test_pfm_broken_memo() {
        let funds = vec![coin(100_000, "muno")];
        let mut src_app = create_app("src");
        let mut dst_app = create_app("dst");
        let mut fwd_app = create_app("fwd");

        let (src_creator_addr, src_contract_addr) = store_and_instantiate_contract(
            &mut src_app,
            "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            funds.clone(),
            "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, dst_contract_addr) = store_and_instantiate_contract(
            &mut dst_app,
            "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            vec![],
            "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, fwd_contract_addr) = store_and_instantiate_contract(
            &mut fwd_app,
            "fwd1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
            vec![],
            "fwd1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        );

        let (_, _, src_channel, dst_channel, src_port, dst_port) =
            create_ibc_connection_and_channel(
                &mut src_app,
                &mut dst_app,
                &src_contract_addr,
                &dst_contract_addr,
            );

        let (_, _, dst_src_channel, fwd_dst_channel, dst_src_port, fwd_dst_port) =
            create_ibc_connection_and_channel(
                &mut dst_app,
                &mut fwd_app,
                &dst_contract_addr,
                &fwd_contract_addr,
            );

        // Save the channel info for forward channels
        let dst_channel_info = ChannelInfo {
            endpoint: IbcEndpoint {
                port_id: dst_src_port.clone(),
                channel_id: dst_src_channel.clone(),
            },
            counterparty_endpoint: IbcEndpoint {
                port_id: src_port.clone(),
                channel_id: src_channel.clone(),
            },
            connection_id: "connection-0".to_string(),
            protocol_version: Ics20Protocol::VERSION.to_string(),
        };

        CHANNEL_INFO
            .save(dst_app.storage_mut(), &dst_src_channel, &dst_channel_info)
            .unwrap();

        let fwd_channel_info = ChannelInfo {
            endpoint: IbcEndpoint {
                port_id: fwd_dst_port.clone(),
                channel_id: fwd_dst_channel.clone(),
            },
            counterparty_endpoint: IbcEndpoint {
                port_id: dst_src_port.clone(),
                channel_id: dst_src_channel.clone(),
            },
            connection_id: "connection-0".to_string(),
            protocol_version: Ics20Protocol::VERSION.to_string(),
        };

        CHANNEL_INFO
            .save(fwd_app.storage_mut(), &fwd_dst_channel, &fwd_channel_info)
            .unwrap();

        let broken_memo: String = format!(
            "{{\"forward\":{{\"receiver\":\"{}\",\"port\":\"{}\",\"channel\":\"{}\"}}}}Thisis_broken_json",
            fwd_contract_addr, fwd_dst_port, fwd_dst_channel
        );

        let endpoint = IbcEndpoint {
            port_id: dst_port.clone(),
            channel_id: dst_channel.clone(),
        };
        let foreign_denom = make_foreign_denom(&endpoint, "muno");
        let factory_denom: String = create_factory_denom(&dst_contract_addr, &foreign_denom);

        let transfer_msg = ExecuteMsg::Transfer(TransferMsg {
            channel: src_channel.clone(),
            receiver: dst_contract_addr.to_string(),
            timeout: None,
            memo: broken_memo.to_string(),
        });

        let res = send_ibc_packet(
            &mut src_app,
            Addr::unchecked(src_creator_addr.clone()),
            src_contract_addr.clone(),
            transfer_msg,
            vec![coin(100_000, "muno")],
        );

        let packet_data_hex = res
            .events
            .iter()
            .find(|event| event.ty == "send_packet")
            .and_then(|event| {
                event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == "packet_data_hex")
            })
            .map(|attr| attr.value.clone())
            .expect("packet_data_hex should be present");

        let packet_data_bytes =
            hex::decode(packet_data_hex).expect("Decoding packet_data_hex should succeed");

        let timeout = IbcTimeout::with_timestamp(dst_app.block_info().time.plus_seconds(60));

        receive_ibc_packet(
            &mut dst_app,
            src_port.clone(),
            src_channel.clone(),
            dst_port.clone(),
            dst_channel.clone(),
            packet_data_bytes.clone(),
            timeout.clone(),
        );

        check_balance(
            &dst_app,
            dst_contract_addr.as_str(),
            &factory_denom,
            Uint128::new(100_000),
            "destination balance on hop chain is not 100k right before PFM execution",
        );

        receive_ibc_packet(
            &mut fwd_app,
            dst_src_port.clone(),
            dst_src_channel.clone(),
            fwd_dst_port.clone(),
            fwd_dst_channel.clone(),
            packet_data_bytes,
            timeout,
        );

        let endpoint_forwarded = IbcEndpoint {
            port_id: fwd_dst_port.clone(),
            channel_id: fwd_dst_channel.clone(),
        };
        let foreign_denom_forwarded = make_foreign_denom(&endpoint_forwarded, "muno");
        let factory_denom_forwarded =
            create_factory_denom(&fwd_contract_addr, &foreign_denom_forwarded);

        check_balance(
            &fwd_app,
            &fwd_contract_addr.to_string(),
            &factory_denom_forwarded,
            Uint128::new(0),
            "destination balance should be 0 after PFM because pfm message was wrong.",
        );
    }

    #[test]
    fn enforce_channel_version_ucs01() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = Ucs01Protocol::VERSION;
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            None,
        )
        .unwrap();
    }

    #[test]
    fn enforce_channel_version_ics20() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = Ics20Protocol::VERSION;
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            None,
        )
        .unwrap()
    }

    #[test]
    fn enforce_channel_wrong_version() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = "ucs01-0999";
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        match enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            None,
        ) {
            Err(ContractError::UnknownProtocol {
                channel_id: unknown_channel_id,
                protocol_version: unknown_protocol_version,
            }) => {
                assert_eq!(unknown_channel_id, channel_id);
                assert_eq!(unknown_protocol_version, protocol_version);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn enforce_channel_counterparty_wrong_version() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = Ucs01Protocol::VERSION;
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        let counterparty_protocol_version = "ucs01-0999";
        match enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            Some(counterparty_protocol_version),
        ) {
            Err(ContractError::UnknownProtocol {
                channel_id: unknown_channel_id,
                protocol_version: unknown_protocol_version,
            }) => {
                assert_eq!(unknown_channel_id, channel_id);
                assert_eq!(unknown_protocol_version, counterparty_protocol_version);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn enforce_channel_protocol_mismatch() {
        let port_id = "port-1";
        let channel_id = "channel-1";
        let connection_id = "connection-1";
        let protocol_version = Ucs01Protocol::VERSION;
        let counterparty_port_id = "port-2";
        let counterparty_channel_id = "channel-2";
        let counterparty_protocol_version = Ics20Protocol::VERSION;
        let mismatch = enforce_order_and_version(
            &IbcChannel::new(
                IbcEndpoint {
                    port_id: port_id.into(),
                    channel_id: channel_id.into(),
                },
                IbcEndpoint {
                    port_id: counterparty_port_id.into(),
                    channel_id: counterparty_channel_id.into(),
                },
                cosmwasm_std::IbcOrder::Unordered,
                protocol_version,
                connection_id,
            ),
            Some(counterparty_protocol_version),
        );
        match mismatch {
            Err(ContractError::ProtocolMismatch {
                channel_id: mismatch_channel_id,
                protocol_version: mismatch_protocol_version,
                counterparty_protocol_version: mismatch_counterparty_protocol_version,
            }) => {
                assert_eq!(mismatch_channel_id, channel_id);
                assert_eq!(mismatch_protocol_version, protocol_version);
                assert_eq!(
                    mismatch_counterparty_protocol_version,
                    counterparty_protocol_version
                );
            }
            _ => panic!(),
        }
    }
}
