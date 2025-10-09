use core::fmt;

use depolama::Bytes;
use ibc_union_spec::ChannelId;
use serde::{Deserialize, Serialize};
use unionlabs_primitives::U256;

#[cfg(doc)]
use crate::msg::{ExecuteMsg, InitMsg};

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
#[repr(u32)]
pub enum Admin {
    Local(LocalAdmin) = 0,
    Remote(RemoteAdmin) = 1,
}

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

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs_encoding::{Bincode, DecodeAs, EncodeAs};

    use super::*;

    #[test]
    fn local_admin_bincode_smoke_test() {
        let local_admin_bz = hex!(
            "00000000"                      // variant (local)
            "0500000000000000" "61646d696e" // address (length, bytes)
        );

        let local_admin = Admin::Local(LocalAdmin {
            address: "admin".to_owned(),
        });

        assert_eq!(
            &Admin::decode_as::<Bincode>(&local_admin_bz).unwrap(),
            &local_admin
        );

        assert_eq!(
            <Bytes>::from(local_admin.encode_as::<Bincode>()),
            <Bytes>::from(local_admin_bz)
        );
    }

    #[test]
    fn remote_admin_bincode_smoke_test() {
        let remote_admin_bz = hex!(
            "01000000"                                                         // variant (remote)
            "0200000000000000" "beef"                                          // address (length, bytes)
            "15000000"                                                         // channel_id
            "0100000000000000000000000000000000000000000000000000000000000000" // path
        );

        let remote_admin = Admin::Remote(RemoteAdmin {
            address: <Bytes>::new_static(&[0xBE, 0xEF]),
            channel_id: ChannelId!(0x15),
            path: U256::ONE,
        });

        assert_eq!(
            &Admin::decode_as::<Bincode>(&remote_admin_bz).unwrap(),
            &remote_admin
        );

        assert_eq!(
            <Bytes>::from(remote_admin.encode_as::<Bincode>()),
            <Bytes>::from(remote_admin_bz)
        );
    }
}
