use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, IbcEndpoint, Uint512};
use cw_controllers::Admin;
use cw_storage_plus::{Item, KeyDeserialize, Map, Prefixer, PrimaryKey};
use serde::{Deserialize, Serialize};
use ucs01_relay_api::middleware::InFlightPfmPacket;
use unionlabs::hash::H256;

pub const ADMIN: Admin = Admin::new("admin");

pub const CONFIG: Item<Config> = Item::new("config");

/// indexed by (channel_id, denom) maintaining the balance of the channel in that currency
pub const CHANNEL_STATE: Map<(&str, &str), ChannelState> = Map::new("channel_state");

/// In flight PFM packets, stored for refund information.
/// Indexed by `PfmRefundPacketKey` (channel_id, port_id, sequence).
pub const IN_FLIGHT_PFM_PACKETS: Map<PfmRefundPacketKey, InFlightPfmPacket> =
    Map::new("in_flight_pfm_packets");

pub const MAX_SUBDENOM_LENGTH: usize = 44;

/// Used for indexing in flight packets for refunds and acknowledgements.
///
/// Given a PFM sequence of A -> B -> C, these keys refer to the packet going from B -> C.
///
/// All of the information in this key is available both when the packet is sent, and when
/// the acknowledgement is received for this packet.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct PfmRefundPacketKey(pub(crate) H256);

impl From<H256> for PfmRefundPacketKey {
    fn from(value: H256) -> Self {
        Self(value)
    }
}

impl<'a> PrimaryKey<'a> for PfmRefundPacketKey {
    type Prefix = <[u8; 32] as PrimaryKey<'a>>::Prefix;

    type SubPrefix = <[u8; 32] as PrimaryKey<'a>>::SubPrefix;

    type Suffix = <[u8; 32] as PrimaryKey<'a>>::Suffix;

    type SuperSuffix = <[u8; 32] as PrimaryKey<'a>>::SuperSuffix;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.get().key()
    }
}

#[derive(Debug, Clone)]
pub struct IbcEndpointKey(IbcEndpoint);

impl From<IbcEndpoint> for IbcEndpointKey {
    fn from(value: IbcEndpoint) -> Self {
        Self(value)
    }
}

impl Prefixer<'_> for IbcEndpointKey {
    fn prefix(&self) -> Vec<cw_storage_plus::Key> {
        let mut res = self.0.port_id.prefix();
        res.extend(self.0.channel_id.prefix());
        res
    }
}

impl KeyDeserialize for IbcEndpointKey {
    type Output = <(String, String) as KeyDeserialize>::Output;

    fn from_vec(value: Vec<u8>) -> cosmwasm_std::StdResult<Self::Output> {
        <(String, String) as KeyDeserialize>::from_vec(value)
    }

    // PrimaryKey is made up of 2 elements
    const KEY_ELEMS: u16 = 2;
}

impl<'a> PrimaryKey<'a> for IbcEndpointKey {
    type Prefix = <(String, String) as PrimaryKey<'a>>::Prefix;

    type SubPrefix = <(String, String) as PrimaryKey<'a>>::SubPrefix;

    type Suffix = <(String, String) as PrimaryKey<'a>>::Suffix;

    type SuperSuffix = <(String, String) as PrimaryKey<'a>>::SuperSuffix;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        let mut keys = self.0.port_id.key();
        keys.extend(self.0.channel_id.key());
        keys
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct DenomHash(pub(crate) H256);

impl From<H256> for DenomHash {
    fn from(value: H256) -> Self {
        Self(value)
    }
}

impl<'a> PrimaryKey<'a> for DenomHash {
    type Prefix = <[u8; 32] as PrimaryKey<'a>>::Prefix;

    type SubPrefix = <[u8; 32] as PrimaryKey<'a>>::SubPrefix;

    type Suffix = <[u8; 32] as PrimaryKey<'a>>::Suffix;

    type SuperSuffix = <[u8; 32] as PrimaryKey<'a>>::SuperSuffix;

    fn key(&self) -> Vec<cw_storage_plus::Key> {
        self.0.get().key()
    }
}

/// Mapping from `source_port/source_channel/denom` to `h(source_port/source_channel/denom)`.
/// This exists in order to verify whether we already created the voucher denom or not.
pub const FOREIGN_DENOM_TO_HASH: Map<(IbcEndpointKey, String), DenomHash> =
    Map::new("foreign_denom_to_hash");

/// Mapping from `h(source_port/source_channel/denom)` to `denom`.
pub const HASH_TO_FOREIGN_DENOM: Map<DenomHash, String> = Map::new("hash_to_foreign_denom");

#[cw_serde]
#[derive(Default)]
pub struct ChannelState {
    pub outstanding: Uint512,
}

#[cw_serde]
pub struct Config {
    pub default_timeout: u64,
    pub ibc_host: Addr,
}
