use std::{collections::BTreeSet, marker::PhantomData};

use cosmwasm_std::{Addr, StdError, StdResult};
use depolama::{KeyCodec, Prefix, Store, ValueCodec, value::ValueCodecViaEncoding};
use ibc_union_spec::{Channel, ChannelId, ClientId, Connection, ConnectionId};
use unionlabs::{
    encoding::Bincode,
    primitives::{ByteArrayExt, Bytes, H256},
};

macro_rules! id_key {
    ($ty:ty) => {
        impl KeyCodec<<$ty as Store>::Key> for $ty {
            fn encode_key(key: &<$ty as Store>::Key) -> Bytes {
                key.raw().to_be_bytes().into()
            }

            fn decode_key(raw: &Bytes) -> StdResult<<$ty as Store>::Key> {
                read_fixed_bytes(raw)
                    .map(u32::from_be_bytes)
                    .map(<$ty as Store>::Key::from_raw)
                    .and_then(|opt| opt.ok_or_else(invalid_id))
            }
        }
    };
}

macro_rules! id_value {
    ($ty:ty) => {
        impl ValueCodec<<$ty as Store>::Value> for $ty {
            fn encode_value(value: &<$ty as Store>::Value) -> Bytes {
                value.raw().to_be_bytes().into()
            }

            fn decode_value(raw: &Bytes) -> StdResult<<$ty as Store>::Value> {
                read_fixed_bytes(raw)
                    .map(u32::from_be_bytes)
                    .map(<$ty as Store>::Value::from_raw)
                    .and_then(|opt| opt.ok_or_else(invalid_id))
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

    type Key = ChannelId;
    type Value = Addr;
}
id_key!(ChannelOwner);
addr_value!(ChannelOwner);

pub enum Channels {}
impl Store for Channels {
    const PREFIX: Prefix = Prefix::new(b"channels");

    type Key = ChannelId;
    type Value = Channel;
}
id_key!(Channels);
impl ValueCodecViaEncoding for Channels {
    type Encoding = Bincode;
}

pub enum ContractChannels {}
impl Store for ContractChannels {
    const PREFIX: Prefix = Prefix::new(b"contract_channels");

    type Key = Addr;
    type Value = BTreeSet<ChannelId>;
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
impl ValueCodec<BTreeSet<ChannelId>> for ContractChannels {
    fn encode_value(value: &BTreeSet<ChannelId>) -> Bytes {
        value.iter().flat_map(|n| n.raw().to_be_bytes()).collect()
    }

    fn decode_value(raw: &Bytes) -> StdResult<BTreeSet<ChannelId>> {
        if raw.len() % 4 != 0 {
            Err(StdError::generic_err(format!(
                "invalid length; expected multiple of 4 bytes but found {}: raw",
                raw.len()
            )))
        } else {
            let set = raw
                .chunks_exact(4)
                .map(|arr| {
                    ChannelId::from_raw(u32::from_be_bytes(
                        arr.try_into().expect("chunks size is valid; qed;"),
                    ))
                    .ok_or_else(|| StdError::generic_err("channel ids must be non-zero"))
                })
                .collect::<Result<BTreeSet<ChannelId>, StdError>>()?;

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

    type Key = ConnectionId;
    type Value = Connection;
}
id_key!(Connections);
impl ValueCodecViaEncoding for Connections {
    type Encoding = Bincode;
}

pub enum ClientStates {}
impl Store for ClientStates {
    const PREFIX: Prefix = Prefix::new(b"client_states");

    type Key = ClientId;
    type Value = Bytes;
}
id_key!(ClientStates);
bytes_value!(ClientStates);

pub enum ClientConsensusStates {}
impl Store for ClientConsensusStates {
    const PREFIX: Prefix = Prefix::new(b"client_consensus_states");

    type Key = (ClientId, u64);
    type Value = Bytes;
}
impl KeyCodec<(ClientId, u64)> for ClientConsensusStates {
    fn encode_key(key: &(ClientId, u64)) -> Bytes {
        key.0
            .raw()
            .to_be_bytes()
            .into_iter()
            .chain(key.1.to_be_bytes())
            .collect()
    }

    fn decode_key(raw: &Bytes) -> StdResult<(ClientId, u64)> {
        read_fixed_bytes::<12>(raw).and_then(|arr| {
            Ok((
                ClientId::from_raw(u32::from_be_bytes(arr.array_slice::<0, 4>()))
                    .ok_or_else(invalid_id)?,
                u64::from_be_bytes(arr.array_slice::<4, 8>()),
            ))
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

    type Key = ClientId;
    type Value = String;
}
id_key!(ClientTypes);
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

    type Key = ClientId;
    type Value = Addr;
}
id_key!(ClientImpls);
addr_value!(ClientImpls);

// From client id to client storage
pub enum ClientStore<S: Store> {
    #[doc(hidden)]
    __(PhantomData<fn() -> S>),
}
impl<S: Store> Store for ClientStore<S> {
    const PREFIX: Prefix = Prefix::new(b"client_store");

    type Key = (ClientId, S::Key);
    type Value = S::Value;
}
impl<S: Store> KeyCodec<(ClientId, S::Key)> for ClientStore<S> {
    fn encode_key((client_id, key): &(ClientId, S::Key)) -> Bytes {
        client_id
            .raw()
            .to_be_bytes()
            .into_iter()
            .chain(S::PREFIX.iter_with_separator().copied())
            .chain(S::encode_key(key))
            .collect()
    }
    fn decode_key(raw: &Bytes) -> StdResult<(ClientId, S::Key)> {
        if raw.len() >= 4 {
            let client_id = ClientId::from_raw(u32::from_be_bytes(
                raw[0..4].try_into().expect("size is checked; qed;"),
            ))
            .ok_or_else(invalid_id)?;

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
    type Value = ClientId;
}
id_value!(NextClientId);

pub enum NextConnectionId {}
impl Store for NextConnectionId {
    const PREFIX: Prefix = Prefix::new(b"next_connection_id");

    type Key = ();
    type Value = ConnectionId;
}
id_value!(NextConnectionId);

pub enum NextChannelId {}
impl Store for NextChannelId {
    const PREFIX: Prefix = Prefix::new(b"next_channel_id");

    type Key = ();
    type Value = ChannelId;
}
id_value!(NextChannelId);

pub enum Commitments {}
impl Store for Commitments {
    const PREFIX: Prefix = Prefix::new(b"");

    type Key = H256;

    type Value = H256;
}
impl KeyCodec<H256> for Commitments {
    fn encode_key(key: &H256) -> Bytes {
        key.into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<H256> {
        read_fixed_bytes(raw).map(H256::new)
    }
}
impl ValueCodec<H256> for Commitments {
    fn encode_value(value: &H256) -> Bytes {
        value.into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<H256> {
        read_fixed_bytes(raw).map(H256::new)
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

fn invalid_id() -> StdError {
    StdError::generic_err("invalid id, must be > 0")
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Storage, testing::mock_dependencies};
    use depolama::StorageExt;
    use hex_literal::hex;
    use ibc_union_spec::{ChannelState, ConnectionState};

    use super::*;

    // nix run .# -- query wasm contract-state raw union1hnuj8f6d3wy3fcprt55vddv7v2650t6uudnvd2hukqrteeam8wjqvcmecf --hex 6368616E6E656C730000000001 --node https://rpc.rpc-node.union-testnet-10.union.build --height 495206
    #[test]
    fn channel() {
        let mut deps = mock_dependencies();

        deps.storage.set(
            &hex!("6368616E6E656C730000000001"),
            &hex!("0200000001000000010100000014000000000000005fbe74a283f7954f10aa04c2edf55578811aeb030c0000000000000075637330332d7a6b676d2d30"),
        );

        let channel = deps.storage.read::<Channels>(&ChannelId!(1)).unwrap();

        assert_eq!(
            channel,
            Channel {
                state: ChannelState::Open,
                connection_id: ConnectionId!(1),
                counterparty_channel_id: Some(ChannelId!(1)),
                counterparty_port_id: hex!("5fbe74a283f7954f10aa04c2edf55578811aeb03").into(),
                version: "ucs03-zkgm-0".to_owned()
            }
        );
    }

    // nix run .# -- query wasm contract-state raw union1hnuj8f6d3wy3fcprt55vddv7v2650t6uudnvd2hukqrteeam8wjqvcmecf --hex 636F6E6E656374696F6E730000000001 --node https://rpc.rpc-node.union-testnet-10.union.build --height 495206
    #[test]
    fn connection() {
        let mut deps = mock_dependencies();

        deps.storage.set(
            &hex!("636F6E6E656374696F6E730000000001"),
            &hex!("0200000001000000010000000101000000"),
        );

        let connection = deps.storage.read::<Connections>(&ConnectionId!(1)).unwrap();

        assert_eq!(
            connection,
            Connection {
                state: ConnectionState::Open,
                client_id: ClientId!(1),
                counterparty_client_id: ClientId!(1),
                counterparty_connection_id: Some(ConnectionId!(1)),
            }
        );
    }

    // nix run .# -- query wasm contract-state raw union1hnuj8f6d3wy3fcprt55vddv7v2650t6uudnvd2hukqrteeam8wjqvcmecf --hex 636F6E74726163745F6368616E6E656C7300756E696F6E313333366A6A386572746C3868377264766E7A3464683572716168643039637930783433677568737878367879727A747832393271706536346668 --node https://rpc.rpc-node.union-testnet-10.union.build --height 495206
    #[test]
    fn contract_channels() {
        let mut deps = mock_dependencies();

        deps.storage.set(
            &hex!("636F6E74726163745F6368616E6E656C7300756E696F6E313333366A6A386572746C3868377264766E7A3464683572716168643039637930783433677568737878367879727A747832393271706536346668"),
            &hex!("000000010000000200000003"),
        );

        let contract_channels = deps
            .storage
            .read::<ContractChannels>(&Addr::unchecked(
                "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh",
            ))
            .unwrap();

        assert_eq!(
            contract_channels,
            [ChannelId!(1), ChannelId!(2), ChannelId!(3)]
                .into_iter()
                .collect::<BTreeSet<_>>()
        );
    }
}
