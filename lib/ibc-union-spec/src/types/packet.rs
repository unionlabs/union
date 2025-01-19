use unionlabs::primitives::Bytes;

use crate::types::ChannelId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub struct Packet {
    pub source_channel_id: ChannelId,
    pub destination_channel_id: ChannelId,
    pub data: Bytes,
    // TODO: Group these into a single PacketTimeout type (one of these fields must be set, but both *can*)
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
}

#[cfg(feature = "ethabi")]
pub mod ethabi {
    use alloy_sol_types::SolValue;

    use super::*;

    unionlabs::impl_ethabi_via_try_from_into!(Packet => ibc_solidity::Packet);

    impl From<Packet> for ibc_solidity::Packet {
        fn from(value: Packet) -> Self {
            Self {
                source_channel_id: value.source_channel_id,
                destination_channel_id: value.destination_channel_id,
                data: value.data.into(),
                timeout_height: value.timeout_height,
                timeout_timestamp: value.timeout_timestamp,
            }
        }
    }

    impl From<ibc_solidity::Packet> for Packet {
        fn from(value: ibc_solidity::Packet) -> Self {
            Self {
                source_channel_id: value.source_channel_id,
                destination_channel_id: value.destination_channel_id,
                data: value.data.into(),
                timeout_height: value.timeout_height,
                timeout_timestamp: value.timeout_timestamp,
            }
        }
    }
}
