use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Env,
    IbcBasicResponse, IbcChannel, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcEndpoint, IbcOrder, IbcPacket, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, Reply, Response, SubMsg, SubMsgResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use ethabi::{ParamType, Token};

use crate::{
    amount::Amount,
    error::{ContractError, Never},
    state::{
        reduce_channel_balance, undo_reduce_channel_balance, ChannelInfo, ReplyArgs, ALLOW_LIST,
        CHANNEL_INFO, CONFIG, REPLY_ARGS,
    },
};

pub const ICS20_VERSION: &str = "ics20-1";
pub const ICS20_ORDERING: IbcOrder = IbcOrder::Unordered;

/// The format for sending an ics20 packet.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/applications/transfer/v1/transfer.proto#L11-L20
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Ics20Packet {
    /// amount of tokens to transfer is encoded as a string, but limited to u64 max
    pub amount: u64,
    /// the token denomination to be transferred
    pub denom: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// the sender address
    pub sender: String,
}

impl Ics20Packet {
    pub fn new<T: Into<String>>(amount: u64, denom: T, sender: &str, receiver: &str) -> Self {
        Ics20Packet {
            denom: denom.into(),
            amount,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
        }
    }

    pub fn eth_encode(self) -> Binary {
        ethabi::encode(&[
            Token::Uint(self.amount.into()),
            Token::String(self.denom),
            Token::String(self.receiver),
            Token::String(self.sender),
        ])
        .into()
    }
}

impl TryFrom<&Binary> for Ics20Packet {
    type Error = ContractError;
    fn try_from(value: &Binary) -> Result<Self, Self::Error> {
        let values = ethabi::decode(
            &[
                ParamType::Uint(256),
                ParamType::String,
                ParamType::String,
                ParamType::String,
            ],
            &value.0,
        )
        .map_err(|_| ContractError::EthAbiDecoding)?;
        match &values[..] {
            [Token::Uint(amount), Token::String(denom), Token::String(receiver), Token::String(sender)] => {
                Ok(Ics20Packet {
                    denom: denom.clone(),
                    amount: (*amount)
                        .try_into()
                        .map_err(|_| ContractError::AmountOverflow {})?,
                    sender: sender.clone(),
                    receiver: receiver.clone(),
                })
            }
            _ => Err(ContractError::EthAbiDecoding),
        }
    }
}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// This is compatible with the JSON serialization
#[cw_serde]
pub enum Ics20Ack {
    Result(Binary),
    Error(String),
}

// create a serialized success message
fn ack_success() -> Binary {
    let res = Ics20Ack::Result(b"1".into());
    to_binary(&res).unwrap()
}

// create a serialized error message
fn ack_fail(err: String) -> Binary {
    let res = Ics20Ack::Error(err);
    to_binary(&res).unwrap()
}

const RECEIVE_ID: u64 = 1337;
const ACK_FAILURE_ID: u64 = 0xfa17;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        RECEIVE_ID => match reply.result {
            SubMsgResult::Ok(_) => Ok(Response::new()),
            SubMsgResult::Err(err) => {
                // Important design note:  with ibcv2 and wasmd 0.22 we can implement this all much easier.
                // No reply needed... the receive function and submessage should return error on failure and all
                // state gets reverted with a proper app-level message auto-generated

                // Since we need compatibility with Juno (Jan 2022), we need to ensure that optimisitic
                // state updates in ibc_packet_receive get reverted in the (unlikely) chance of an
                // error while sending the token

                // However, this requires passing some state between the ibc_packet_receive function and
                // the reply handler. We do this with a singleton, with is "okay" for IBC as there is no
                // reentrancy on these functions (cannot be called by another contract). This pattern
                // should not be used for ExecuteMsg handlers
                let reply_args = REPLY_ARGS.load(deps.storage)?;
                undo_reduce_channel_balance(
                    deps.storage,
                    &reply_args.channel,
                    &reply_args.denom,
                    reply_args.amount,
                )?;

                Ok(Response::new().set_data(ack_fail(err)))
            }
        },
        ACK_FAILURE_ID => match reply.result {
            SubMsgResult::Ok(_) => Ok(Response::new()),
            SubMsgResult::Err(err) => Ok(Response::new().set_data(ack_fail(err))),
        },
        _ => Err(ContractError::UnknownReplyId { id: reply.id }),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// enforces ordering and versioning constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<(), ContractError> {
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// record the channel in CHANNEL_INFO
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // we need to check the counter party version in try and ack (sometimes here)
    enforce_order_and_version(msg.channel(), msg.counterparty_version())?;

    let channel: IbcChannel = msg.into();
    let info = ChannelInfo {
        id: channel.endpoint.channel_id,
        counterparty_endpoint: channel.counterparty_endpoint,
        connection_id: channel.connection_id,
    };
    CHANNEL_INFO.save(deps.storage, &info.id, &info)?;

    Ok(IbcBasicResponse::default())
}

pub(crate) fn enforce_order_and_version(
    channel: &IbcChannel,
    counterparty_version: Option<&str>,
) -> Result<(), ContractError> {
    if channel.version != ICS20_VERSION {
        return Err(ContractError::InvalidIbcVersion {
            version: channel.version.clone(),
        });
    }
    if let Some(version) = counterparty_version {
        if version != ICS20_VERSION {
            return Err(ContractError::InvalidIbcVersion {
                version: version.to_string(),
            });
        }
    }
    if channel.order != ICS20_ORDERING {
        return Err(ContractError::OnlyOrderedChannel {});
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: what to do here?
    // we will have locked funds that need to be returned somehow
    unimplemented!();
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// Check to see if we have any balance here
/// We should not return an error if possible, but rather an acknowledgement of failure
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    let packet = msg.packet;

    do_ibc_packet_receive(deps, &packet).or_else(|err| {
        Ok(IbcReceiveResponse::new()
            .set_ack(ack_fail(err.to_string()))
            .add_attributes(vec![
                attr("action", "receive"),
                attr("success", "false"),
                attr("error", err.to_string()),
            ]))
    })
}

// Returns local denom if the denom is an encoded voucher from the expected endpoint
// Otherwise, error
fn parse_voucher_denom<'a>(
    voucher_denom: &'a str,
    remote_endpoint: &IbcEndpoint,
) -> Result<&'a str, ContractError> {
    let split_denom: Vec<&str> = voucher_denom.splitn(3, '/').collect();
    if split_denom.len() != 3 {
        return Err(ContractError::NoForeignTokens {});
    }
    // a few more sanity checks
    if split_denom[0] != remote_endpoint.port_id {
        return Err(ContractError::FromOtherPort {
            port: split_denom[0].into(),
        });
    }
    if split_denom[1] != remote_endpoint.channel_id {
        return Err(ContractError::FromOtherChannel {
            channel: split_denom[1].into(),
        });
    }

    Ok(split_denom[2])
}

// this does the work of ibc_packet_receive, we wrap it to turn errors into acknowledgements
fn do_ibc_packet_receive(
    deps: DepsMut,
    packet: &IbcPacket,
) -> Result<IbcReceiveResponse, ContractError> {
    let msg: Ics20Packet = (&packet.data).try_into()?;
    let channel = packet.dest.channel_id.clone();

    // If the token originated on the remote chain, it looks like "ucosm".
    // If it originated on our chain, it looks like "port/channel/ucosm".
    let denom = parse_voucher_denom(&msg.denom, &packet.src)?;

    // make sure we have enough balance for this
    reduce_channel_balance(deps.storage, &channel, denom, msg.amount)?;

    // we need to save the data to update the balances in reply
    let reply_args = ReplyArgs {
        channel,
        denom: denom.to_string(),
        amount: msg.amount,
    };
    REPLY_ARGS.save(deps.storage, &reply_args)?;

    let to_send = Amount::from_parts(denom.to_string(), msg.amount);
    let gas_limit = check_gas_limit(deps.as_ref(), &to_send)?;
    let send = send_amount(to_send, msg.receiver.clone());
    let mut submsg = SubMsg::reply_on_error(send, RECEIVE_ID);
    submsg.gas_limit = gas_limit;

    let res = IbcReceiveResponse::new()
        .set_ack(ack_success())
        .add_submessage(submsg)
        .add_attribute("action", "receive")
        .add_attribute("sender", msg.sender)
        .add_attribute("receiver", msg.receiver)
        .add_attribute("denom", denom)
        .add_attribute("amount", Uint128::from(msg.amount))
        .add_attribute("success", "true");

    Ok(res)
}

fn check_gas_limit(deps: Deps, amount: &Amount) -> Result<Option<u64>, ContractError> {
    match amount {
        Amount::Cw20(coin) => {
            // if cw20 token, use the registered gas limit, or error if not whitelisted
            let addr = deps.api.addr_validate(&coin.address)?;
            let allowed = ALLOW_LIST.may_load(deps.storage, &addr)?;
            match allowed {
                Some(allow) => Ok(allow.gas_limit),
                None => match CONFIG.load(deps.storage)?.default_gas_limit {
                    Some(base) => Ok(Some(base)),
                    None => Err(ContractError::NotOnAllowList),
                },
            }
        }
        _ => Ok(None),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// check if success or failure and update balance, or return funds
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // Design decision: should we trap error like in receive?
    // TODO: unsure... as it is now a failed ack handling would revert the tx and would be
    // retried again and again. is that good?
    let ics20msg: Ics20Ack = from_binary(&msg.acknowledgement.data)?;
    match ics20msg {
        Ics20Ack::Result(_) => on_packet_success(deps, msg.original_packet),
        Ics20Ack::Error(err) => on_packet_failure(deps, msg.original_packet, err),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// return fund to original sender (same as failure in ibc_packet_ack)
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    // TODO: trap error like in receive? (same question as ack above)
    let packet = msg.packet;
    on_packet_failure(deps, packet, "timeout".to_string())
}

// update the balance stored on this (channel, denom) index
fn on_packet_success(_deps: DepsMut, packet: IbcPacket) -> Result<IbcBasicResponse, ContractError> {
    let msg: Ics20Packet = (&packet.data).try_into()?;

    // similar event messages like ibctransfer module
    let attributes = vec![
        attr("action", "acknowledge"),
        attr("sender", &msg.sender),
        attr("receiver", &msg.receiver),
        attr("denom", &msg.denom),
        attr("amount", Uint128::from(msg.amount)),
        attr("success", "true"),
    ];

    Ok(IbcBasicResponse::new().add_attributes(attributes))
}

// return the tokens to sender
fn on_packet_failure(
    deps: DepsMut,
    packet: IbcPacket,
    err: String,
) -> Result<IbcBasicResponse, ContractError> {
    let msg: Ics20Packet = (&packet.data).try_into()?;

    // undo the balance update on failure (as we pre-emptively added it on send)
    reduce_channel_balance(deps.storage, &packet.src.channel_id, &msg.denom, msg.amount)?;

    let to_send = Amount::from_parts(msg.denom.clone(), msg.amount);
    let gas_limit = check_gas_limit(deps.as_ref(), &to_send)?;
    let send = send_amount(to_send, msg.sender.clone());
    let mut submsg = SubMsg::reply_on_error(send, ACK_FAILURE_ID);
    submsg.gas_limit = gas_limit;

    // similar event messages like ibctransfer module
    let res = IbcBasicResponse::new()
        .add_submessage(submsg)
        .add_attribute("action", "acknowledge")
        .add_attribute("sender", msg.sender)
        .add_attribute("receiver", msg.receiver)
        .add_attribute("denom", msg.denom)
        .add_attribute("amount", msg.amount.to_string())
        .add_attribute("success", "false")
        .add_attribute("error", err);

    Ok(res)
}

fn send_amount(amount: Amount, recipient: String) -> CosmosMsg {
    match amount {
        Amount::Native(coin) => BankMsg::Send {
            to_address: recipient,
            amount: vec![coin],
        }
        .into(),
        Amount::Cw20(coin) => {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient,
                amount: coin.amount,
            };
            WasmMsg::Execute {
                contract_addr: coin.address,
                msg: to_binary(&msg).unwrap(),
                funds: vec![],
            }
            .into()
        }
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{
        coins,
        testing::{mock_env, mock_info},
        to_vec, IbcEndpoint, IbcMsg, IbcTimeout, Timestamp,
    };
    use cw20::Cw20ReceiveMsg;

    use super::*;
    use crate::{
        contract::{execute, migrate, query_channel},
        msg::{ExecuteMsg, MigrateMsg, TransferMsg},
        test_helpers::*,
    };

    #[test]
    fn check_ack_json() {
        let success = Ics20Ack::Result(b"1".into());
        let fail = Ics20Ack::Error("bad coin".into());

        let success_json = String::from_utf8(to_vec(&success).unwrap()).unwrap();
        assert_eq!(r#"{"result":"MQ=="}"#, success_json.as_str());

        let fail_json = String::from_utf8(to_vec(&fail).unwrap()).unwrap();
        assert_eq!(r#"{"error":"bad coin"}"#, fail_json.as_str());
    }

    #[test]
    fn check_encode_decode_iso() {
        let packet = Ics20Packet::new(
            12345,
            "ucosm",
            "cosmos1zedxv25ah8fksmg2lzrndrpkvsjqgk4zt5ff7n",
            "wasm1fucynrfkrt684pm8jrt8la5h2csvs5cnldcgqc",
        );
        assert_eq!(Ok(packet.clone()), (&packet.eth_encode()).try_into());
    }

    fn cw20_payment(
        amount: u128,
        address: &str,
        recipient: &str,
        gas_limit: Option<u64>,
    ) -> SubMsg {
        let msg = Cw20ExecuteMsg::Transfer {
            recipient: recipient.into(),
            amount: Uint128::new(amount),
        };
        let exec = WasmMsg::Execute {
            contract_addr: address.into(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        };
        let mut msg = SubMsg::reply_on_error(exec, RECEIVE_ID);
        msg.gas_limit = gas_limit;
        msg
    }

    fn native_payment(amount: u128, denom: &str, recipient: &str) -> SubMsg {
        SubMsg::reply_on_error(
            BankMsg::Send {
                to_address: recipient.into(),
                amount: coins(amount, denom),
            },
            RECEIVE_ID,
        )
    }

    fn mock_receive_packet(
        my_channel: &str,
        amount: u64,
        denom: &str,
        receiver: &str,
    ) -> IbcPacket {
        let data = Ics20Packet {
            // this is returning a foreign (our) token, thus denom is <port>/<channel>/<denom>
            denom: format!("{}/{}/{}", REMOTE_PORT, "channel-1234", denom),
            amount,
            sender: "remote-sender".to_string(),
            receiver: receiver.to_string(),
        };
        print!("Packet denom: {}", &data.denom);
        IbcPacket::new(
            data.eth_encode(),
            IbcEndpoint {
                port_id: REMOTE_PORT.to_string(),
                channel_id: "channel-1234".to_string(),
            },
            IbcEndpoint {
                port_id: CONTRACT_PORT.to_string(),
                channel_id: my_channel.to_string(),
            },
            3,
            Timestamp::from_seconds(1665321069).into(),
        )
    }

    #[test]
    fn send_receive_cw20() {
        let send_channel = "channel-9";
        let cw20_addr = "token-addr";
        let cw20_denom = "cw20:token-addr";
        let gas_limit = 1234567;
        let mut deps = setup(
            &["channel-1", "channel-7", send_channel],
            &[(cw20_addr, gas_limit)],
        );

        // prepare some mock packets
        let recv_packet = mock_receive_packet(send_channel, 876543210, cw20_denom, "local-rcpt");
        let recv_high_packet =
            mock_receive_packet(send_channel, 1876543210, cw20_denom, "local-rcpt");

        // cannot receive this denom yet
        let msg = IbcPacketReceiveMsg::new(recv_packet.clone());
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        assert!(res.messages.is_empty());
        let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        let no_funds = Ics20Ack::Error(ContractError::InsufficientFunds {}.to_string());
        assert_eq!(ack, no_funds);

        // we send some cw20 tokens over
        let transfer = TransferMsg {
            channel: send_channel.to_string(),
            remote_address: "remote-rcpt".to_string(),
            timeout: None,
        };
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: "local-sender".to_string(),
            amount: Uint128::new(987654321),
            msg: to_binary(&transfer).unwrap(),
        });
        let info = mock_info(cw20_addr, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        let expected = Ics20Packet {
            denom: cw20_denom.into(),
            amount: 987654321,
            sender: "local-sender".to_string(),
            receiver: "remote-rcpt".to_string(),
        };
        let timeout = mock_env().block.time.plus_seconds(DEFAULT_TIMEOUT);
        assert_eq!(
            &res.messages[0],
            &SubMsg::new(IbcMsg::SendPacket {
                channel_id: send_channel.to_string(),
                data: expected.eth_encode(),
                timeout: IbcTimeout::with_timestamp(timeout),
            })
        );

        // query channel state|_|
        let state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap();
        assert_eq!(state.balances, vec![Amount::cw20(987654321, cw20_addr)]);
        assert_eq!(state.total_sent, vec![Amount::cw20(987654321, cw20_addr)]);

        // cannot receive more than we sent
        let msg = IbcPacketReceiveMsg::new(recv_high_packet);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        assert!(res.messages.is_empty());
        let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        assert_eq!(ack, no_funds);

        // we can receive less than we sent
        let msg = IbcPacketReceiveMsg::new(recv_packet);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            cw20_payment(876543210, cw20_addr, "local-rcpt", Some(gas_limit)),
            res.messages[0]
        );
        let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        assert!(matches!(ack, Ics20Ack::Result(_)));

        // TODO: we need to call the reply block

        // query channel state
        let state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap();
        assert_eq!(state.balances, vec![Amount::cw20(111111111, cw20_addr)]);
        assert_eq!(state.total_sent, vec![Amount::cw20(987654321, cw20_addr)]);
    }

    #[test]
    fn send_receive_native() {
        let send_channel = "channel-9";
        let mut deps = setup(&["channel-1", "channel-7", send_channel], &[]);

        let denom = "uatom";

        // prepare some mock packets
        let recv_packet = mock_receive_packet(send_channel, 876543210, denom, "local-rcpt");
        let recv_high_packet = mock_receive_packet(send_channel, 1876543210, denom, "local-rcpt");

        // cannot receive this denom yet
        let msg = IbcPacketReceiveMsg::new(recv_packet.clone());
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        assert!(res.messages.is_empty());
        let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        let no_funds = Ics20Ack::Error(ContractError::InsufficientFunds {}.to_string());
        assert_eq!(ack, no_funds);

        // we transfer some tokens
        let msg = ExecuteMsg::Transfer(TransferMsg {
            channel: send_channel.to_string(),
            remote_address: "my-remote-address".to_string(),
            timeout: None,
        });
        let info = mock_info("local-sender", &coins(987654321, denom));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // query channel state|_|
        let state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap();
        assert_eq!(state.balances, vec![Amount::native(987654321, denom)]);
        assert_eq!(state.total_sent, vec![Amount::native(987654321, denom)]);

        // cannot receive more than we sent
        let msg = IbcPacketReceiveMsg::new(recv_high_packet);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        assert!(res.messages.is_empty());
        let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        assert_eq!(ack, no_funds);

        // we can receive less than we sent
        let msg = IbcPacketReceiveMsg::new(recv_packet);
        let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            native_payment(876543210, denom, "local-rcpt"),
            res.messages[0]
        );
        let ack: Ics20Ack = from_binary(&res.acknowledgement).unwrap();
        assert!(matches!(ack, Ics20Ack::Result(_)));

        // only need to call reply block on error case

        // query channel state
        let state = query_channel(deps.as_ref(), send_channel.to_string()).unwrap();
        assert_eq!(state.balances, vec![Amount::native(111111111, denom)]);
        assert_eq!(state.total_sent, vec![Amount::native(987654321, denom)]);
    }

    #[test]
    fn check_gas_limit_handles_all_cases() {
        let send_channel = "channel-9";
        let allowed = "foobar";
        let allowed_gas = 777666;
        let mut deps = setup(&[send_channel], &[(allowed, allowed_gas)]);

        // allow list will get proper gas
        let limit = check_gas_limit(deps.as_ref(), &Amount::cw20(500, allowed)).unwrap();
        assert_eq!(limit, Some(allowed_gas));

        // non-allow list will error
        let random = "tokenz";
        check_gas_limit(deps.as_ref(), &Amount::cw20(500, random)).unwrap_err();

        // add default_gas_limit
        let def_limit = 54321;
        migrate(
            deps.as_mut(),
            mock_env(),
            MigrateMsg {
                default_gas_limit: Some(def_limit),
            },
        )
        .unwrap();

        // allow list still gets proper gas
        let limit = check_gas_limit(deps.as_ref(), &Amount::cw20(500, allowed)).unwrap();
        assert_eq!(limit, Some(allowed_gas));

        // non-allow list will now get default
        let limit = check_gas_limit(deps.as_ref(), &Amount::cw20(500, random)).unwrap();
        assert_eq!(limit, Some(def_limit));
    }
}
