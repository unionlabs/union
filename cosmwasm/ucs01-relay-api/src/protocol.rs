use std::fmt::Debug;

use cosmwasm_std::{
    Addr, Attribute, Binary, Coin, CosmosMsg, Event, IbcBasicResponse, IbcEndpoint, IbcMsg,
    IbcOrder, IbcPacket, IbcPacketAckMsg, IbcReceiveResponse, Response, SubMsg, Timestamp,
};
use thiserror::Error;
use unionlabs::encoding::{self, Decode, DecodeErrorOf, Encode};

use crate::{
    middleware::{Memo, PacketForward},
    types::{EncodingError, GenericAck, TransferPacket, TransferPacketCommon, TransferToken},
};

// https://github.com/cosmos/ibc-go/blob/8218aeeef79d556852ec62a773f2bc1a013529d4/modules/apps/transfer/types/keys.go#L12
pub const TRANSFER_MODULE: &str = "transfer";

// https://github.com/cosmos/ibc-go/blob/8218aeeef79d556852ec62a773f2bc1a013529d4/modules/apps/transfer/types/events.go#L4-L22
pub const PACKET_EVENT: &str = "fungible_token_packet";
pub const TRANSFER_EVENT: &str = "ibc_transfer";
pub const TIMEOUT_EVENT: &str = "timeout";
pub const MESSAGE_EVENT: &str = "message";

pub const ATTR_MODULE: &str = "module";
pub const ATTR_SENDER: &str = "sender";
pub const ATTR_RECEIVER: &str = "receiver";
pub const ATTR_REFUND_RECEIVER: &str = "refund_receiver";
pub const ATTR_MEMO: &str = "memo";
pub const ATTR_SUCCESS: &str = "success";
pub const ATTR_ERROR: &str = "error";
pub const ATTR_ACK: &str = "acknowledgement";
pub const ATTR_PFM: &str = "pfm";
pub const ATTR_ASSETS: &str = "assets";

pub const ATTR_VALUE_PFM_ACK: &str = "pfm_ack";
pub const ATTR_VALUE_TRUE: &str = "true";
pub const ATTR_VALUE_FALSE: &str = "false";

/// NOTE: This number holds no significance, and was arbitrarily chosen.
pub const IBC_SEND_ID: u64 = 69;

const ACK_ERR_TIMEOUT_MSG: &[u8] = b"giving up on forwarded packet after timeout";

#[derive(Error, Debug, PartialEq)]
pub enum ProtocolError {
    #[error("channel doesn't exist: {channel_id}")]
    NoSuchChannel { channel_id: String },
    #[error("protocol must be caller")]
    Unauthorized,
}

pub type PacketExtensionOf<T> = <<T as TransferProtocol>::Packet as TransferPacket>::Extension;

pub struct TransferInput {
    pub current_time: Timestamp,
    pub timeout_delta: u64,
    pub sender: Addr,
    pub receiver: String,
    pub tokens: Vec<TransferToken>,
}

pub fn tokens_to_attr(tokens: impl IntoIterator<Item = TransferToken>) -> Attribute {
    (
        ATTR_ASSETS,
        cosmwasm_std::to_json_string(
            &tokens
                .into_iter()
                .map(|token| Coin::new(token.amount.u128(), token.denom))
                .collect::<Vec<_>>(),
        )
        .expect("impossible"),
    )
        .into()
}

pub type AddrOf<T> = <T as TransferPacket>::Addr;

// We follow the following module implementation, events and attributes are
// almost 1:1 with the traditional go implementation. As we generalized the base
// implementation for multi-tokens transfer, the events are not containing a
// single ('denom', 'value') and ('amount', 'value') attributes but rather a set
// of ('denom', json_string(('x', '10'))) attributes for each denom `x` with
// amount 10 that is transferred.
// https://github.com/cosmos/ibc-go/blob/7be17857b10457c67cbf66a49e13a9751eb10e8e/modules/apps/transfer/ibc_module.go
pub trait TransferProtocol {
    /// Must be unique per Protocol
    const VERSION: &'static str;
    const ORDERING: IbcOrder;
    /// Must be unique per Protocol
    const RECEIVE_REPLY_ID: u64;

    type Packet: Decode<Self::Encoding> + Encode<Self::Encoding> + TransferPacket;

    type Ack: Decode<Self::Encoding> + Encode<Self::Encoding> + Into<GenericAck>;

    type Encoding: encoding::Encoding;

    type CustomMsg;

    type Error: Debug
        + From<ProtocolError>
        + From<EncodingError>
        + From<DecodeErrorOf<Self::Encoding, Self::Packet>>
        + From<DecodeErrorOf<Self::Encoding, Self::Ack>>;

    fn channel_endpoint(&self) -> &IbcEndpoint;

    fn caller(&self) -> &Addr;

    fn self_addr(&self) -> &Addr;

    // TODO: Remove use of Encoding Error
    fn common_to_protocol_packet(
        &self,
        packet: TransferPacketCommon<PacketExtensionOf<Self>>,
    ) -> Result<Self::Packet, EncodingError>;

    fn ack_success() -> Self::Ack;

    fn ack_failure(error: String) -> Self::Ack;

    fn normalize_for_ibc_transfer(
        &mut self,
        token: TransferToken,
    ) -> Result<TransferToken, Self::Error>;

    fn send_tokens(
        &mut self,
        sender: &AddrOf<Self::Packet>,
        receiver: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error>;

    fn send_tokens_success(
        &mut self,
        sender: &AddrOf<Self::Packet>,
        receiver: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error>;

    fn send_tokens_failure(
        &mut self,
        sender: &AddrOf<Self::Packet>,
        receiver: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
    ) -> Result<Vec<CosmosMsg<Self::CustomMsg>>, Self::Error>;

    fn send(
        &mut self,
        mut input: TransferInput,
        extension: PacketExtensionOf<Self>,
    ) -> Result<Response<Self::CustomMsg>, Self::Error> {
        input.tokens = input
            .tokens
            .into_iter()
            .map(|token| self.normalize_for_ibc_transfer(token))
            .collect::<Result<Vec<_>, _>>()?;

        let packet = self.common_to_protocol_packet(TransferPacketCommon {
            sender: input.sender.clone().to_string(),
            receiver: input.receiver.clone(),
            tokens: input.tokens.clone(),
            extension: extension.clone(),
        })?;

        let send_msgs = self.send_tokens(packet.sender(), packet.receiver(), packet.tokens())?;

        let memo = extension.to_string();
        let transfer_event = if memo.is_empty() {
            Event::new(TRANSFER_EVENT)
        } else {
            Event::new(TRANSFER_EVENT).add_attribute(ATTR_MEMO, &memo)
        };

        let tokens = packet.tokens();
        let sub = SubMsg::reply_on_success(
            IbcMsg::SendPacket {
                channel_id: self.channel_endpoint().channel_id.clone(),
                data: packet.encode().into(),
                timeout: input.current_time.plus_seconds(input.timeout_delta).into(),
            },
            IBC_SEND_ID,
        );
        Ok(Response::new()
            .add_messages(send_msgs)
            .add_submessage(sub)
            .add_events([
                transfer_event
                    .add_attributes([
                        (ATTR_SENDER, input.sender.as_str()),
                        (ATTR_RECEIVER, input.receiver.as_str()),
                    ])
                    .add_attributes([tokens_to_attr(tokens)]),
                Event::new(MESSAGE_EVENT).add_attribute(ATTR_MODULE, TRANSFER_MODULE),
            ]))
    }

    fn send_ack(
        &mut self,
        ibc_packet: IbcPacketAckMsg,
    ) -> Result<IbcBasicResponse<Self::CustomMsg>, Self::Error> {
        // NOTE: `ibc_packet.original_packet` here refers to the packet that is being acknowledged, not the
        // original packet in the pfm chain. At this point in the ack handling process, we don't even know
        // if this is a pfm message anyways.

        let packet = Self::Packet::decode(ibc_packet.original_packet.data.as_slice())?;

        // https://github.com/cosmos/ibc-go/blob/5ca37ef6e56a98683cf2b3b1570619dc9b322977/modules/apps/transfer/ibc_module.go#L261
        let ack: GenericAck = Self::Ack::decode(ibc_packet.acknowledgement.data.as_slice())?.into();
        let memo = packet.extension().to_string();

        let (ack_msgs, ack_attr) = if let Some((ack_msgs, ack_attr)) = self.pfm_ack(
            ack.clone(),
            ibc_packet.original_packet.clone(),
            packet.sender(),
            packet.tokens(),
        )? {
            (ack_msgs, ack_attr)
        } else {
            match ack {
                Ok(value) => {
                    let value_string = Binary::from(value).to_string();
                    (
                        self.send_tokens_success(
                            packet.sender(),
                            packet.receiver(),
                            packet.tokens(),
                        )?,
                        Vec::from_iter(
                            (!value_string.is_empty()).then_some((ATTR_SUCCESS, value_string)),
                        ),
                    )
                }
                Err(error) => {
                    let error_string = Binary::from(error).to_string();
                    (
                        self.send_tokens_failure(
                            packet.sender(),
                            packet.receiver(),
                            packet.tokens(),
                        )?,
                        Vec::from_iter(
                            (!error_string.is_empty()).then_some((ATTR_ERROR, error_string)),
                        ),
                    )
                }
            }
        };

        let packet_event = {
            Event::new(PACKET_EVENT)
                .add_attributes((!memo.is_empty()).then_some((ATTR_MEMO, &memo)))
        };

        Ok(IbcBasicResponse::new()
            .add_event(
                packet_event
                    .add_attributes([
                        (ATTR_MODULE, TRANSFER_MODULE),
                        (ATTR_SENDER, packet.sender().to_string().as_str()),
                        (ATTR_RECEIVER, packet.receiver().to_string().as_str()),
                        (
                            ATTR_ACK,
                            ibc_packet.acknowledgement.data.to_string().as_str(),
                        ),
                    ])
                    .add_attributes([tokens_to_attr(packet.tokens())]),
            )
            .add_event(Event::new(PACKET_EVENT).add_attributes(ack_attr))
            .add_messages(ack_msgs))
    }

    fn send_timeout(
        &mut self,
        ibc_packet: IbcPacket,
    ) -> Result<IbcBasicResponse<Self::CustomMsg>, Self::Error> {
        let packet = Self::Packet::decode(ibc_packet.clone().data.as_slice())?;
        // same branch as failure ack
        let memo = packet.extension().to_string();
        let ack = GenericAck::Err(ACK_ERR_TIMEOUT_MSG.to_vec());
        let refund_msgs = if let Some((ack_msgs, _)) =
            self.pfm_ack(ack, ibc_packet.clone(), packet.sender(), packet.tokens())?
        {
            ack_msgs
        } else {
            self.send_tokens_failure(packet.sender(), packet.receiver(), packet.tokens())?
        };

        let timeout_event = if memo.is_empty() {
            Event::new(PACKET_EVENT)
        } else {
            Event::new(PACKET_EVENT).add_attribute(ATTR_MEMO, &memo)
        };

        Ok(IbcBasicResponse::new()
            .add_event(
                timeout_event
                    .add_attributes([
                        (ATTR_MODULE, TRANSFER_MODULE),
                        (ATTR_REFUND_RECEIVER, packet.sender().to_string().as_str()),
                    ])
                    .add_attributes([tokens_to_attr(packet.tokens())]),
            )
            .add_messages(refund_msgs))
    }

    #[allow(clippy::type_complexity)]
    fn receive_transfer(
        &mut self,
        receiver: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
    ) -> Result<(Vec<TransferToken>, Vec<CosmosMsg<Self::CustomMsg>>), Self::Error>;

    fn receive(&mut self, original_packet: IbcPacket) -> IbcReceiveResponse<Self::CustomMsg> {
        let handle = || -> Result<IbcReceiveResponse<Self::CustomMsg>, Self::Error> {
            let packet = Self::Packet::decode(original_packet.data.as_slice())?;

            let memo = packet.extension().to_string();

            if let Ok(memo) = serde_json_wasm::from_str::<Memo>(&memo) {
                match memo {
                    Memo::Forward { forward } => {
                        return Ok(self.packet_forward(packet, original_packet, forward, false))
                    }
                    Memo::None { .. } => {}
                };
            }

            // NOTE: The default message ack is always successful and only
            // overwritten if the submessage execution revert via the reply
            // handler. The caller must ensure that the protocol is called in
            // the reply handler via the `receive_error` for the acknowledgement
            // to be overwritten.
            let transfer_msgs = self
                .receive_transfer(packet.receiver(), packet.tokens())?
                .1
                .into_iter()
                .map(|msg| SubMsg::reply_on_error(msg, Self::RECEIVE_REPLY_ID));

            let packet_event = if memo.is_empty() {
                Event::new(PACKET_EVENT)
            } else {
                Event::new(PACKET_EVENT).add_attribute(ATTR_MEMO, &memo)
            };

            Ok(IbcReceiveResponse::new(Self::ack_success().encode())
                .add_event(
                    packet_event
                        .add_attributes([
                            (ATTR_MODULE, TRANSFER_MODULE),
                            (ATTR_SENDER, packet.sender().to_string().as_str()),
                            (ATTR_RECEIVER, packet.receiver().to_string().as_str()),
                            (ATTR_SUCCESS, ATTR_VALUE_TRUE),
                        ])
                        .add_attributes([tokens_to_attr(packet.tokens())]),
                )
                .add_submessages(transfer_msgs))
        };

        match handle() {
            Ok(response) => response,
            // NOTE: same branch as if the submessage fails
            Err(err) => Self::receive_error(err),
        }
    }

    fn receive_error(error: impl Debug) -> IbcReceiveResponse<Self::CustomMsg> {
        let error = format!("{:?}", error);

        IbcReceiveResponse::new(Self::ack_failure(error.clone()).encode()).add_event(
            Event::new(PACKET_EVENT).add_attributes([
                (ATTR_MODULE, TRANSFER_MODULE),
                (ATTR_SUCCESS, ATTR_VALUE_FALSE),
                (ATTR_ERROR, &error),
            ]),
        )
    }

    /// Extracts and processes the forward information from a messages memo. Initiates the forward transfer process.
    fn packet_forward(
        &mut self,
        packet: Self::Packet,
        original_packet: IbcPacket,
        forward: PacketForward,
        processed: bool,
    ) -> IbcReceiveResponse<Self::CustomMsg>;

    /// Create the IBC transfer message from the provided forward information.
    ///
    /// Modifies the payload of the sub message with the ID [`IBC_SEND_ID`] to contain the [`InFlightPfmPacket`] serde json encoded struct.
    fn forward_transfer_packet(
        &mut self,
        tokens: Vec<Coin>,
        original_packet: IbcPacket,
        forward: PacketForward,
        receiver: Addr,
    ) -> Result<IbcReceiveResponse<Self::CustomMsg>, Self::Error>;

    /// On packet acknowledgment (be it success, failure, or timeout) check if the packet is a PFM packet.
    /// If the packet is a PFM packet, overwrite the acknowledgment process to forward the acknowledgment to the original sender.
    /// Handle and overwrites required for refunding tokens correctly.
    /// Returns a tuple of `(messages, attributes)`.
    #[allow(clippy::type_complexity)]
    fn pfm_ack(
        &mut self,
        ack: GenericAck,
        ibc_packet: IbcPacket,
        sender: &AddrOf<Self::Packet>,
        tokens: Vec<TransferToken>,
    ) -> Result<Option<(Vec<CosmosMsg<Self::CustomMsg>>, Vec<(&str, String)>)>, Self::Error>;

    fn convert_ack_to_foreign_protocol(
        &self,
        foreign_protocol: &str,
        ack: GenericAck,
    ) -> Result<GenericAck, Self::Error>;
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Coin, Uint128};

    use crate::{
        protocol::{tokens_to_attr, ATTR_ASSETS},
        types::TransferToken,
    };

    #[test]
    fn test_token_attr() {
        let token = TransferToken {
            denom: "factory/1/2/3".into(),
            amount: 0xDEAD_u64.into(),
        };
        let token2 = TransferToken {
            denom: "factory/1/3/3".into(),
            amount: 0xC0DE_u64.into(),
        };
        let attr = tokens_to_attr([token, token2]);
        let coins = cosmwasm_std::from_json::<Vec<Coin>>(attr.value).unwrap();
        assert_eq!(attr.key, ATTR_ASSETS);
        assert_eq!(coins[0].denom, "factory/1/2/3");
        assert_eq!(coins[0].amount, Uint128::from(0xDEAD_u64));
        assert_eq!(coins[1].denom, "factory/1/3/3");
        assert_eq!(coins[1].amount, Uint128::from(0xC0DE_u64));
    }
}
