use cosmwasm_std::{Addr, Binary, Event, IbcPacket, IbcTimeout};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unionlabs::{
    id::{ChannelId, PortId},
    validated::{Validate, Validated},
};

pub const DEFAULT_PFM_TIMEOUT: &str = "1m";
pub const DEFAULT_PFM_RETRIES: u8 = 0;

pub const PFM_MODULE_NAME: &str = "packetforwardmiddleware";

pub const PFM_ERROR_EVENT: &str = "packet_forward_error";
pub const PFM_HOP_EVENT: &str = "packet_forward_hop";

pub const RECV_SEQUENCE_ATTR: &str = "recv_sequence";
pub const SENT_SEQUENCE_ATTR: &str = "sent_sequence";
pub const DEST_CHANNEL_ATTR: &str = "dest_channel";
pub const DEST_PORT_ATTR: &str = "dest_port";
pub const SRC_CHANNEL_ATTR: &str = "src_channel";
pub const SRC_PORT_ATTR: &str = "src_port";

#[derive(Error, Debug, PartialEq)]
pub enum MiddlewareError {
    #[error("{0}")]
    PacketForward(#[from] PacketForwardError),
}

#[derive(Error, Debug, PartialEq)]
pub enum PacketForwardError {
    #[error("A packet returned via timeout or ack did not contain a refund index")]
    NoPacketRefundInformation,
    #[error("Unable to find a packet with the given refund index")]
    PacketNotInRefundStore,
    #[error("Unable to encode/decode packet")]
    InvalidEncoding,
    #[error("Unable to index for reply message in stack")]
    NoReplyMessageInStack,
}

pub fn default_pfm_timeout() -> String {
    DEFAULT_PFM_TIMEOUT.to_owned()
}

pub fn default_pfm_retries() -> u8 {
    DEFAULT_PFM_RETRIES
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PacketReturnInfo {
    InFlight(Box<InFlightPfmPacket>),
    NewPacket(PacketId),
}

/// Given that we can't know the IBC packet sequence of a new packet before it's sent, we instead construct and store this information about a packet to index it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PacketId {
    pub height: u64,
    pub index: u64,
}

/// Information about an in flight packet, used to process retries and refunds.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct InFlightPfmPacket {
    pub original_sender_addr: Addr,
    pub packet_data: Binary,
    pub packet_src_channel_id: String,
    pub packet_src_port_id: String,
    pub refund_channel_id: String,
    pub refund_port_id: String,
    pub packet_sequence: u64,
    pub timeout: u64,
    pub src_packet_timeout: IbcTimeout,
    pub forward_channel_id: String,
    pub forward_port_id: String,
    pub original_protocol_version: String,
}

impl InFlightPfmPacket {
    pub fn new(
        original_sender_addr: Addr,
        original_packet: IbcPacket,
        timeout: u64,
        forward_channel_id: String,
        forward_port_id: String,
        original_protocol_version: String,
    ) -> Self {
        Self {
            original_sender_addr,
            packet_data: original_packet.data,
            packet_src_channel_id: original_packet.src.channel_id,
            packet_src_port_id: original_packet.src.port_id,
            refund_channel_id: original_packet.dest.channel_id,
            refund_port_id: original_packet.dest.port_id,
            timeout,
            src_packet_timeout: original_packet.timeout,
            packet_sequence: original_packet.sequence,
            forward_channel_id,
            forward_port_id,
            original_protocol_version,
        }
    }

    pub fn create_hop_event(&self, sent_sequence: u64) -> Event {
        Event::new(PFM_HOP_EVENT)
            .add_attribute(RECV_SEQUENCE_ATTR, self.packet_sequence.to_string())
            .add_attribute(DEST_CHANNEL_ATTR, self.forward_channel_id.clone())
            .add_attribute(DEST_PORT_ATTR, self.forward_port_id.clone())
            .add_attribute(SENT_SEQUENCE_ATTR, sent_sequence.to_string())
            .add_attribute(SRC_CHANNEL_ATTR, self.packet_src_channel_id.clone())
            .add_attribute(SRC_PORT_ATTR, self.packet_src_port_id.clone())
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Memo {
    Forward { forward: PacketForward },
    None {},
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PacketForward {
    pub receiver: PfmReceiver,
    pub port: PortId,
    pub channel: ChannelId,
    #[serde(default = "default_pfm_timeout")]
    pub timeout: String,
    #[serde(default = "default_pfm_retries")]
    pub retries: u8,
    pub next: Option<Box<PacketForward>>,
    pub return_info: Option<PacketId>,
}

impl PacketForward {
    /// Effective timeout is equivalent to `timeout * retries`.
    ///
    /// If the `timeout` is invalid or cannot be parsed, the default timeout is used.
    /// Timeouts are considered invalid if they are less than or equal to zero.
    pub fn get_effective_timeout(&self) -> u64 {
        let retries = self.retries as i64 + 1;
        let default_timeout = go_parse_duration::parse_duration(&default_pfm_timeout())
            .expect("default timeout is correctly formatted")
            * retries;

        (match go_parse_duration::parse_duration(&self.timeout) {
            Ok(timeout) => {
                if timeout <= 0 {
                    default_timeout
                } else {
                    timeout * retries
                }
            }
            Err(_error) => default_timeout,
        }) as u64
    }
}

pub type PfmReceiverValidator = NotEmptyString;
pub type PfmReceiver = Validated<String, PfmReceiverValidator>;

pub struct NotEmptyString;

// TODO: Not specific to receiver
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("failed to validate metadata. receiver cannot be empty")]
pub struct EmptyStringError;

impl<T: Into<String> + From<String>> Validate<T> for NotEmptyString {
    type Error = EmptyStringError;

    fn validate(t: T) -> Result<T, Self::Error> {
        let s = t.into();

        if s.is_empty() {
            Err(EmptyStringError)
        } else {
            Ok(s.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Memo;

    #[test]
    fn serde_parses_memo() {
        // let memo = "\"balls\": \"string\"";
        let memo = "{\"forward\": {\"receiver\": \"[eth_addr]\",\"port\": \"[union-eth port]\",\"channel\": \"[union-eth channel]\",\"timeout\": \"1000000\",\"retries\": 0}}";

        let parsed = serde_json_wasm::from_str::<Memo>(memo).expect("works");

        dbg!(parsed);

        // panic!()
    }
}
