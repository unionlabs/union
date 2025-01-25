use std::collections::BTreeSet;

use cosmwasm_std::{Addr, Binary, StdError, StdResult};
use cw_storage_plus::{Item, Map};
use depolama::{Prefix, Store};
use ibc_union_spec::types::{Channel, Connection};
use unionlabs::{
    encoding::{Bincode, DecodeAs, EncodeAs},
    primitives::Bytes,
    ByteArrayExt,
};

pub const QUERY_STORE: Item<Binary> = Item::new("query_store");

pub const CHANNEL_OWNER: Map<u32, Addr> = Map::new("channel_owner");

pub const CHANNELS: Map<u32, Channel> = Map::new("channels");

pub const CONTRACT_CHANNELS: Map<Addr, BTreeSet<u32>> = Map::new("contract_channels");

pub const CONNECTIONS: Map<u32, Connection> = Map::new("connections");

pub const CLIENT_STATES: Map<u32, Binary> = Map::new("client_states");

pub const CLIENT_CONSENSUS_STATES: Map<(u32, u64), Binary> = Map::new("client_consensus_states");

// From client type to contract implementation
pub const CLIENT_REGISTRY: Map<&str, Addr> = Map::new("client_registry");

// From client id to client type
pub const CLIENT_TYPES: Map<u32, String> = Map::new("client_types");

// From client id to contract implementation
pub const CLIENT_IMPLS: Map<u32, Addr> = Map::new("client_impls");

pub const NEXT_CLIENT_ID: Item<u32> = Item::new("next_client_id");

pub const NEXT_CONNECTION_ID: Item<u32> = Item::new("next_connection_id");

pub const NEXT_CHANNEL_ID: Item<u32> = Item::new("next_channel_id");

macro_rules! unit_key {
    () => {
        type Key = ();

        fn encode_key((): &Self::Key) -> Bytes {
            [].into()
        }

        fn decode_key(raw: &Bytes) -> StdResult<Self::Key> {
            decode_unit_key(raw)
        }
    };
}

macro_rules! u32_key {
    () => {
        type Key = u32;

        fn encode_key(key: &Self::Key) -> Bytes {
            key.to_be_bytes().into()
        }

        fn decode_key(raw: &Bytes) -> StdResult<Self::Key> {
            read_fixed_bytes(raw).map(u32::from_be_bytes)
        }
    };
}

macro_rules! u32_value {
    () => {
        type Value = u32;

        fn encode_value(value: &Self::Value) -> Bytes {
            value.to_be_bytes().into()
        }

        fn decode_value(raw: &Bytes) -> StdResult<Self::Value> {
            read_fixed_bytes(raw).map(u32::from_be_bytes)
        }
    };
}

macro_rules! bytes_value {
    () => {
        type Value = Bytes;

        fn encode_value(value: &Self::Value) -> Bytes {
            value.clone()
        }

        fn decode_value(raw: &Bytes) -> StdResult<Self::Value> {
            Ok(raw.clone())
        }
    };
}

macro_rules! bincode_value {
    ($ty:ty) => {
        type Value = $ty;

        fn encode_value(value: &Self::Value) -> Bytes {
            value.encode_as::<Bincode>().into()
        }

        fn decode_value(raw: &Bytes) -> StdResult<Self::Value> {
            <$ty>::decode_as::<Bincode>(raw).map_err(|e| {
                StdError::generic_err(format!("unable to decode {}: {e}", stringify!($ty)))
            })
        }
    };
}

macro_rules! addr_value {
    () => {
        type Value = Addr;

        fn encode_value(value: &Self::Value) -> Bytes {
            value.as_bytes().into()
        }

        fn decode_value(raw: &Bytes) -> StdResult<Self::Value> {
            String::from_utf8(raw.to_vec())
                .map(Addr::unchecked)
                .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
        }
    };
}

pub enum QueryStore {}
impl Store for QueryStore {
    const PREFIX: Prefix = Prefix::new(b"query_store");

    unit_key!();

    bytes_value!();
}

pub enum ChannelOwner {}
impl Store for ChannelOwner {
    const PREFIX: Prefix = Prefix::new(b"channel_owner");

    u32_key!();

    addr_value!();
}

pub enum Channels {}
impl Store for Channels {
    const PREFIX: Prefix = Prefix::new(b"channels");

    u32_key!();

    bincode_value!(Channel);
}

pub enum ContractChannels {}
impl Store for ContractChannels {
    const PREFIX: Prefix = Prefix::new(b"channels");

    type Key = Addr;

    type Value = BTreeSet<u32>;

    fn encode_key(key: &Self::Key) -> Bytes {
        key.as_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<Self::Key> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid key: {e}")))
    }

    fn encode_value(value: &Self::Value) -> Bytes {
        value.iter().flat_map(|n| n.to_be_bytes()).collect()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Self::Value> {
        if raw.len() % 4 != 0 {
            Err(StdError::generic_err(format!(
                "invalid length; expected multiple of 4 bytes but found {}: raw",
                raw.len()
            )))
        } else {
            let set = raw
                .chunks_exact(4)
                .map(|arr| u32::from_be_bytes(arr.try_into().expect("chunks size is valid; qed;")))
                .collect::<BTreeSet<u32>>();

            if set.len() != raw.len() / 4 {
                Err(StdError::generic_err("duplicate elements in set"))
            } else {
                Ok(set)
            }
        }
    }
}

pub enum Connections {}
impl Store for Connections {
    const PREFIX: Prefix = Prefix::new(b"connections");

    u32_key!();

    bincode_value!(Connection);
}

pub enum ClientStates {}
impl Store for ClientStates {
    const PREFIX: Prefix = Prefix::new(b"client_states");

    u32_key!();

    bytes_value!();
}

pub enum ClientConsensusStates {}
impl Store for ClientConsensusStates {
    const PREFIX: Prefix = Prefix::new(b"client_consensus_states");

    type Key = (u32, u64);

    fn encode_key(key: &Self::Key) -> Bytes {
        key.0
            .to_be_bytes()
            .into_iter()
            .chain(key.1.to_be_bytes())
            .collect()
    }

    fn decode_key(raw: &Bytes) -> StdResult<Self::Key> {
        read_fixed_bytes::<12>(raw).map(|arr| {
            (
                u32::from_be_bytes(arr.array_slice::<0, 4>()),
                u64::from_be_bytes(arr.array_slice::<4, 8>()),
            )
        })
    }

    bytes_value!();
}

pub enum ClientRegistry {}
impl Store for ClientRegistry {
    const PREFIX: Prefix = Prefix::new(b"client_registry");

    type Key = String;

    fn encode_key(key: &Self::Key) -> Bytes {
        key.as_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<Self::Key> {
        String::from_utf8(raw.to_vec())
            .map_err(|e| StdError::generic_err(format!("invalid key: {e}")))
    }

    addr_value!();
}

pub enum ClientTypes {}
impl Store for ClientTypes {
    const PREFIX: Prefix = Prefix::new(b"client_types");

    u32_key!();

    type Value = String;

    fn encode_value(value: &Self::Value) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<Self::Value> {
        String::from_utf8(raw.to_vec())
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

pub enum ClientImpls {}
impl Store for ClientImpls {
    const PREFIX: Prefix = Prefix::new(b"client_impls");

    u32_key!();

    addr_value!();
}

pub enum NextClientId {}
impl Store for NextClientId {
    const PREFIX: Prefix = Prefix::new(b"next_client_id");

    unit_key!();

    u32_value!();
}

pub enum NextConnectionId {}
impl Store for NextConnectionId {
    const PREFIX: Prefix = Prefix::new(b"next_connection_id");

    unit_key!();

    u32_value!();
}

pub enum NextChannelId {}
impl Store for NextChannelId {
    const PREFIX: Prefix = Prefix::new(b"next_channel_id");

    unit_key!();

    u32_value!();
}

fn decode_unit_key(raw: &Bytes) -> Result<(), StdError> {
    if raw.is_empty() {
        Ok(())
    } else {
        Err(StdError::generic_err(format!(
            "key must be empty, found {raw}"
        )))
    }
}

fn read_fixed_bytes<const N: usize>(raw: &Bytes) -> StdResult<[u8; N]> {
    raw.try_into().map_err(|_| {
        StdError::generic_err(format!(
            "invalid key: expected {N} bytes, found {}: {raw}",
            raw.len()
        ))
    })
}
