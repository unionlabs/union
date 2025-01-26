use std::{collections::BTreeSet, marker::PhantomData};

use cosmwasm_std::{Addr, StdError, StdResult};
use depolama::{KeyCodec, Prefix, Store, ValueCodec};
use ibc_union_spec::types::{Channel, Connection};
use unionlabs::{
    encoding::{Bincode, DecodeAs, EncodeAs},
    primitives::Bytes,
    ByteArrayExt,
};

macro_rules! u32_key {
    ($ty:ty) => {
        impl KeyCodec<u32> for $ty {
            fn encode_key(key: &u32) -> Bytes {
                key.to_be_bytes().into()
            }

            fn decode_key(raw: &Bytes) -> StdResult<u32> {
                read_fixed_bytes(raw).map(u32::from_be_bytes)
            }
        }
    };
}

macro_rules! u32_value {
    ($ty:ty) => {
        impl ValueCodec<u32> for $ty {
            fn encode_value(value: &u32) -> Bytes {
                value.to_be_bytes().into()
            }

            fn decode_value(raw: &Bytes) -> StdResult<u32> {
                read_fixed_bytes(raw).map(u32::from_be_bytes)
            }
        }
    };
}

macro_rules! bytes_value {
    ($ty:ty) => {
        impl ValueCodec<Bytes> for $ty {
            fn encode_value(value: &Bytes) -> Bytes {
                value.clone()
            }

            fn decode_value(raw: &Bytes) -> StdResult<Bytes> {
                Ok(raw.clone())
            }
        }
    };
}

macro_rules! bincode_value {
    ($ty:ty) => {
        impl ValueCodec<<$ty as Store>::Value> for $ty {
            fn encode_value(value: &<$ty as Store>::Value) -> Bytes {
                value.encode_as::<Bincode>().into()
            }

            fn decode_value(raw: &Bytes) -> StdResult<<$ty as Store>::Value> {
                <<$ty as Store>::Value>::decode_as::<Bincode>(raw).map_err(|e| {
                    StdError::generic_err(format!("unable to decode {}: {e}", stringify!($ty)))
                })
            }
        }
    };
}

macro_rules! addr_value {
    ($ty:ty) => {
        impl ValueCodec<Addr> for $ty {
            fn encode_value(value: &Addr) -> Bytes {
                value.as_bytes().into()
            }

            fn decode_value(raw: &Bytes) -> StdResult<Addr> {
                String::from_utf8(raw.to_vec())
                    .map(Addr::unchecked)
                    .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
            }
        }
    };
}

pub enum QueryStore {}
impl Store for QueryStore {
    const PREFIX: Prefix = Prefix::new(b"query_store");

    type Key = ();
    type Value = Bytes;
}
bytes_value!(QueryStore);

pub enum ChannelOwner {}
impl Store for ChannelOwner {
    const PREFIX: Prefix = Prefix::new(b"channel_owner");

    type Key = u32;
    type Value = Addr;
}
u32_key!(ChannelOwner);
addr_value!(ChannelOwner);

pub enum Channels {}
impl Store for Channels {
    const PREFIX: Prefix = Prefix::new(b"channels");

    type Key = u32;
    type Value = Channel;
}
u32_key!(Channels);
bincode_value!(Channels);

pub enum ContractChannels {}
impl Store for ContractChannels {
    const PREFIX: Prefix = Prefix::new(b"contract_channels");

    type Key = Addr;
    type Value = BTreeSet<u32>;
}
impl KeyCodec<Addr> for ContractChannels {
    fn encode_key(key: &Addr) -> Bytes {
        key.as_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<Addr> {
        String::from_utf8(raw.to_vec())
            .map(Addr::unchecked)
            .map_err(|e| StdError::generic_err(format!("invalid key: {e}")))
    }
}
impl ValueCodec<BTreeSet<u32>> for ContractChannels {
    fn encode_value(value: &BTreeSet<u32>) -> Bytes {
        value.iter().flat_map(|n| n.to_be_bytes()).collect()
    }

    fn decode_value(raw: &Bytes) -> StdResult<BTreeSet<u32>> {
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

    type Key = u32;
    type Value = Connection;
}
u32_key!(Connections);
bincode_value!(Connections);

pub enum ClientStates {}
impl Store for ClientStates {
    const PREFIX: Prefix = Prefix::new(b"client_states");

    type Key = u32;

    type Value = Bytes;
}
u32_key!(ClientStates);
bytes_value!(ClientStates);

pub enum ClientConsensusStates {}
impl Store for ClientConsensusStates {
    const PREFIX: Prefix = Prefix::new(b"client_consensus_states");

    type Key = (u32, u64);
    type Value = Bytes;
}
impl KeyCodec<(u32, u64)> for ClientConsensusStates {
    fn encode_key(key: &(u32, u64)) -> Bytes {
        key.0
            .to_be_bytes()
            .into_iter()
            .chain(key.1.to_be_bytes())
            .collect()
    }

    fn decode_key(raw: &Bytes) -> StdResult<(u32, u64)> {
        read_fixed_bytes::<12>(raw).map(|arr| {
            (
                u32::from_be_bytes(arr.array_slice::<0, 4>()),
                u64::from_be_bytes(arr.array_slice::<4, 8>()),
            )
        })
    }
}
bytes_value!(ClientConsensusStates);

// From client type to contract implementation
pub enum ClientRegistry {}
impl Store for ClientRegistry {
    const PREFIX: Prefix = Prefix::new(b"client_registry");

    type Value = Addr;
    type Key = String;
}
impl KeyCodec<String> for ClientRegistry {
    fn encode_key(key: &String) -> Bytes {
        key.as_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<String> {
        String::from_utf8(raw.to_vec())
            .map_err(|e| StdError::generic_err(format!("invalid key: {e}")))
    }
}
addr_value!(ClientRegistry);

// From client id to client type
pub enum ClientTypes {}
impl Store for ClientTypes {
    const PREFIX: Prefix = Prefix::new(b"client_types");

    type Key = u32;

    type Value = String;
}
u32_key!(ClientTypes);
impl ValueCodec<String> for ClientTypes {
    fn encode_value(value: &String) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<String> {
        String::from_utf8(raw.to_vec())
            .map_err(|e| StdError::generic_err(format!("invalid value: {e}")))
    }
}

// From client id to contract implementation
pub enum ClientImpls {}
impl Store for ClientImpls {
    const PREFIX: Prefix = Prefix::new(b"client_impls");

    type Key = u32;
    type Value = Addr;
}
u32_key!(ClientImpls);
addr_value!(ClientImpls);

// From client id to client storage
pub enum ClientStore<S: Store> {
    __(PhantomData<fn() -> S>),
}
impl<S: Store> Store for ClientStore<S> {
    const PREFIX: Prefix = Prefix::new(b"client_store");

    type Key = (u32, S::Key);
    type Value = S::Value;
}
impl<S: Store> KeyCodec<(u32, S::Key)> for ClientStore<S> {
    fn encode_key((client_id, key): &(u32, S::Key)) -> Bytes {
        client_id
            .to_be_bytes()
            .into_iter()
            .chain(S::PREFIX.iter_with_separator().copied())
            .chain(S::encode_key(key))
            .collect()
    }
    fn decode_key(raw: &Bytes) -> StdResult<(u32, S::Key)> {
        if raw.len() >= 4 {
            let client_id =
                u32::from_be_bytes(raw[0..4].try_into().expect("size is checked; qed;"));

            // TODO: Improve the Bytes type such that we don't need to re-allocate here
            let key = S::decode_key(&raw[4..].into())?;

            Ok((client_id, key))
        } else {
            Err(StdError::generic_err(format!(
                "invalid key: expected at least {N} bytes, found {}: {raw}",
                raw.len(),
                N = u32::BITS / 8,
            )))
        }
    }
}
impl<S: Store> ValueCodec<S::Value> for ClientStore<S> {
    fn encode_value(value: &S::Value) -> Bytes {
        S::encode_value(value)
    }
    fn decode_value(raw: &Bytes) -> StdResult<S::Value> {
        S::decode_value(raw)
    }
}

pub enum NextClientId {}
impl Store for NextClientId {
    const PREFIX: Prefix = Prefix::new(b"next_client_id");

    type Key = ();
    type Value = u32;
}
u32_value!(NextClientId);

pub enum NextConnectionId {}
impl Store for NextConnectionId {
    const PREFIX: Prefix = Prefix::new(b"next_connection_id");

    type Key = ();
    type Value = u32;
}
u32_value!(NextConnectionId);

pub enum NextChannelId {}
impl Store for NextChannelId {
    const PREFIX: Prefix = Prefix::new(b"next_channel_id");

    type Key = ();
    type Value = u32;
}
u32_value!(NextChannelId);

fn read_fixed_bytes<const N: usize>(raw: &Bytes) -> StdResult<[u8; N]> {
    raw.try_into().map_err(|_| {
        StdError::generic_err(format!(
            "invalid key: expected {N} bytes, found {}: {raw}",
            raw.len()
        ))
    })
}
