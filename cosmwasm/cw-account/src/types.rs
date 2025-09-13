use core::fmt;

use depolama::Bytes;
use ibc_union_spec::ChannelId;
use serde::{Deserialize, Serialize};
use unionlabs_primitives::U256;

/// A local admin.
///
/// This is an address on this chain. If this contract was instantiated with [`InitMsg::Zkgm`], then a local admin can be added with [`ExecuteMsg::AddAdmin`].
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    bincode::Encode,
    bincode::Decode,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct LocalAdmin {
    #[cfg_attr(feature = "schemars", schemars(with = "cosmwasm_std::Addr"))]
    pub address: String,
}

/// A remote admin.
///
/// A remote admin is configured automatically if this contract is instantiated with [`InitMsg::Zkgm`]. Additional remote admins can be added with [`ExecuteMsg::AddAdmin`].
///
/// This tuple of `(source address, destination channel id, path)` is required to uniquely
/// identify cross-chain remote ownership.
#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    bincode::Encode,
    bincode::Decode,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct RemoteAdmin {
    /// The address of the admin on the *counterparty* chain.
    pub address: Bytes,
    /// The id of the channel on *this* chain.
    pub channel_id: ChannelId,
    /// The path of the packet. This will typically be `0`, but if the packet was a forward
    /// packet, then this will contain the path back to the source chain.
    pub path: U256,
}

#[derive(
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    bincode::Encode,
    bincode::Decode,
)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Admin {
    Local(LocalAdmin),
    Remote(RemoteAdmin),
}

impl fmt::Display for Admin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local(LocalAdmin { address }) => write!(f, "local:{address}"),
            Self::Remote(RemoteAdmin {
                address,
                channel_id,
                path,
            }) => write!(f, "remote:{path}/{channel_id}/{address}"),
        }
    }
}
