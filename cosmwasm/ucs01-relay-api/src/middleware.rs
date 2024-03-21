use serde::{Deserialize, Serialize};
use unionlabs::{
    id::{ChannelId, PortId},
    validated::{Validate, Validated},
};

pub const DEFAULT_PFM_TIMEOUT: &str = "10m";
pub const DEFAULT_PFM_RETRIES: u8 = 1;

fn default_pfm_timeout() -> String {
    DEFAULT_PFM_TIMEOUT.to_owned()
}

fn default_pfm_retries() -> u8 {
    DEFAULT_PFM_RETRIES
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Memo {
    Forward { forward: PacketForward },
    None {},
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PacketForward {
    receiver: PfmReceiver,
    port: PortId,
    channel: ChannelId,
    #[serde(default = "default_pfm_timeout")]
    timeout: String,
    #[serde(default = "default_pfm_retries")]
    retries: u8,
    next: Option<Box<PacketForward>>,
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

/// Returns the receiver address for a given channel and original sender.
///
/// It overrides the receiver address to be a hash of the channel/origSender so that
/// the receiver address is deterministic and can be used to identify the sender on the
/// initial chain.
pub fn GetReceiver(channel_id: ChannelId, original_sender: String) -> String {
    todo!()
}
