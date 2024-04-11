use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use unionlabs::{
    id::{ChannelId, PortId},
    validated::{Validate, Validated},
};

pub const DEFAULT_PFM_TIMEOUT: &str = "1m";
pub const DEFAULT_PFM_RETRIES: u8 = 0;
pub const PFM_MODULE_NAME: &str = "packetforwardmiddleware";

pub fn default_pfm_timeout() -> String {
    DEFAULT_PFM_TIMEOUT.to_owned()
}

pub fn default_pfm_retries() -> u8 {
    DEFAULT_PFM_RETRIES
}

/// Information about an in flight packet, used to process retries and refunds.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct InFlightPfmPacket {
    pub nonrefundable: bool,
    pub original_sender_addr: Addr,
    pub packet_data: String,
    pub packet_src_channel_id: String,
    pub packet_src_port_id: String,
    pub refund_channel_id: String,
    pub refund_port_id: String,
    pub refund_sequence: u64,
    pub retries_remaining: u32,
    pub timeout: u64,
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
}

impl PacketForward {
    /// Effective timeout is equivilant to `timeout * retries`.
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
