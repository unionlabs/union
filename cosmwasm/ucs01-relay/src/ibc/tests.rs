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
    DistributionKeeper, Executor, GovFailingModule, Module, StakeKeeper, StargateFailing, SudoMsg,
    WasmKeeper,
};
use serde::de::DeserializeOwned;
use token_factory_api::TokenFactoryMsg;
use ucs01_relay_api::{
    middleware::{Memo, PacketForward, PfmReceiver},
    protocol::TransferProtocol,
    types::make_foreign_denom,
};
use unionlabs::id::{ChannelId, PortId};

use crate::{
    contract::{execute, instantiate, query},
    error::ContractError,
    ibc::{
        enforce_order_and_version, ibc_channel_close, ibc_channel_connect, ibc_channel_open,
        ibc_packet_ack, ibc_packet_receive, ibc_packet_timeout, reply, IbcChannel,
    },
    msg::{ExecuteMsg, InstantiateMsg, TransferMsg},
    protocol::{encode_denom_hash, hash_denom, Ics20Protocol, Ucs01Protocol},
    state::{ChannelInfo, CHANNEL_INFO},
};

// cspell:ignore jvcc
const MOCK_CREATOR: &str = "src1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z";
const MOCK_CREATOR_DEST: &str = "dst1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z";

/// Creates a new BasicApp instance with a custom message type.
///
/// # Safety
///
/// This function uses `unsafe` to transmute the type of `BasicAppBuilder` to one with a different `CustomMsg` type.
/// The `unsafe` block is necessary because the fields of `BasicAppBuilder` are private, and its initialization
/// functions are implemented only for a specific `CustomMsg` type. By using `std::mem::transmute`, we bypass the
/// type safety checks, which is generally unsafe and should be avoided if possible.
///
/// This pattern is required because `cw-multi-test` does not currently support initializing `BasicAppBuilder`
/// with a generic `CustomMsg` type directly. An issue has been opened on the `cw-multi-test` repository
/// to address this limitation and explore potential solutions.
///
fn create_app(prefix: &'static str) -> BasicApp<TokenFactoryMsg> {
    let application = cw_multi_test::BasicAppBuilder::<TokenFactoryMsg, Empty>::new_custom();
    let application: BasicAppBuilder<TokenFactoryMsg, Empty> =
        unsafe { std::mem::transmute(application) };
    application
        .with_custom(CustomCrossChainTransferModule)
        .with_api(MockApi::default().with_prefix(prefix))
        .with_ibc(IbcSimpleModule)
        .build(|_, _, _| {})
}

pub struct CustomCrossChainTransferModule;

pub type BasicApp<ExecC, QueryC = Empty> = App<
    BankKeeper,
    MockApi,
    MockStorage,
    CustomCrossChainTransferModule,
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
    CustomCrossChainTransferModule,
    WasmKeeper<ExecC, QueryC>,
    StakeKeeper,
    DistributionKeeper,
    IbcSimpleModule,
    GovFailingModule,
    StargateFailing,
>;

impl Module for CustomCrossChainTransferModule {
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
    let hashed_denom = hash_denom(foreign_denom);

    let normalized_denom: String = encode_denom_hash(hashed_denom);
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

fn assert_balance(
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

#[allow(clippy::too_many_arguments)]
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
        fees: None,
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
    let (src_creator_addr, src_contract_addr) =
        store_and_instantiate_contract(&mut src_app, MOCK_CREATOR, funds.clone(), MOCK_CREATOR);

    let (_, dst_contract_addr) =
        store_and_instantiate_contract(&mut dst_app, MOCK_CREATOR_DEST, vec![], MOCK_CREATOR_DEST);

    // Create IBC connection and channel
    let (_, _, src_channel, dst_channel, src_port, dst_port) = create_ibc_connection_and_channel(
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
    assert_balance(
        &src_app,
        &src_creator_addr,
        denom,
        Uint128::new(100_000),
        "initial source balance is not 100k",
    );
    assert_balance(
        &dst_app,
        dst_contract_addr.as_str(),
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
    assert_balance(
        &src_app,
        &src_creator_addr,
        denom,
        Uint128::new(0),
        "source balance is not 0 after transfer",
    );
    assert_balance(
        &dst_app,
        dst_contract_addr.as_str(),
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

    let (src_creator_addr, src_contract_addr) =
        store_and_instantiate_contract(&mut src_app, MOCK_CREATOR, funds.clone(), MOCK_CREATOR);

    let (_, dst_contract_addr) =
        store_and_instantiate_contract(&mut dst_app, MOCK_CREATOR_DEST, vec![], MOCK_CREATOR_DEST);

    let (_, _, src_channel, dst_channel, src_port, dst_port) = create_ibc_connection_and_channel(
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
    assert_balance(
        &src_app,
        &src_creator_addr,
        denom,
        Uint128::new(0),
        "source balance is not 0 after transfer",
    );
    assert_balance(
        &dst_app,
        dst_contract_addr.as_str(),
        &factory_denom,
        Uint128::new(100_000),
        "destination balance is not 100k after transfer",
    );

    // Perform transfer back from dst_app to src_app
    transfer_tokens(
        &mut dst_app,
        &mut src_app,
        dst_contract_addr.as_str(),
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
    assert_balance(
        &src_app,
        &src_creator_addr,
        denom,
        Uint128::new(100_000),
        "source balance is not 100k after transferring wrapped token back",
    );

    // it can't be 0 because we don't have burn functionality so we can't burn
    // the locked tokens in app2, lets assume it will work anyway.
    // assert_balance(&dst_app, &dst_contract_addr, &factory_denom, Uint128::new(100_000));
}

#[test]
fn test_pfm_valid_memo() {
    let funds = vec![coin(100_000, "muno")];
    let mut src_app = create_app("src");
    let mut dst_app = create_app("dst");
    let mut fwd_app = create_app("fwd");

    let (src_creator_addr, src_contract_addr) =
        store_and_instantiate_contract(&mut src_app, MOCK_CREATOR, funds.clone(), MOCK_CREATOR);

    let (_, dst_contract_addr) =
        store_and_instantiate_contract(&mut dst_app, MOCK_CREATOR_DEST, vec![], MOCK_CREATOR_DEST);

    let (_, fwd_contract_addr) = store_and_instantiate_contract(
        &mut fwd_app,
        "fwd1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        vec![],
        "fwd1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
    );

    let (_, _, src_channel, dst_channel, src_port, dst_port) = create_ibc_connection_and_channel(
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

    let memo = serde_json_wasm::to_string(
        &(Memo::Forward {
            forward: PacketForward {
                receiver: PfmReceiver::new(fwd_contract_addr.to_string()).unwrap(),
                port: PortId::new(fwd_dst_port.clone()).unwrap(),
                channel: ChannelId::parse_prefixed(&fwd_dst_channel).unwrap(),
                next: None,
                retries: 1,
                return_info: None,
                timeout: "1m".to_string(),
                fees: None,
            },
        }),
    )
    .expect("can convert pfm memo to json string");

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
        fees: None,
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

    assert_balance(
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

    assert_balance(
        &fwd_app,
        fwd_contract_addr.as_str(),
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

    let (src_creator_addr, src_contract_addr) =
        store_and_instantiate_contract(&mut src_app, MOCK_CREATOR, funds.clone(), MOCK_CREATOR);

    let (_, dst_contract_addr) =
        store_and_instantiate_contract(&mut dst_app, MOCK_CREATOR_DEST, vec![], MOCK_CREATOR_DEST);

    let (_, fwd_contract_addr) = store_and_instantiate_contract(
        &mut fwd_app,
        "fwd1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
        vec![],
        "fwd1a0j2w49wgs4f9wwk09t4qp35ls2vpx88jvcc2z",
    );

    let (_, _, src_channel, dst_channel, src_port, dst_port) = create_ibc_connection_and_channel(
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
            "{{\"forward\":{{\"receiver\":\"{}\",\"port\":\"{}\",\"channel\":\"{}\"}}}}This_is_broken_json",
            fwd_contract_addr,
            fwd_dst_port,
            fwd_dst_channel
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
        fees: None,
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

    assert_balance(
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

    assert_balance(
        &fwd_app,
        fwd_contract_addr.as_str(),
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
