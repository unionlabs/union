use cosmwasm_std::{StdError, StdResult};
use depolama::{Bytes, KeyCodec, Prefix, Store, value::ValueCodecViaEncoding};
use parlia_types::Valset;
use unionlabs::encoding::Bincode;

pub enum ValsetStore {}

impl Store for ValsetStore {
    const PREFIX: Prefix = Prefix::new(b"sync_committee");

    type Key = u64;
    type Value = Valset;
}

impl KeyCodec<u64> for ValsetStore {
    fn encode_key(key: &u64) -> Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<u64> {
        raw.try_into()
            .map_err(|_| {
                StdError::generic_err(format!(
                    "invalid key: expected {N} bytes, found {}: {raw}",
                    raw.len(),
                    N = u64::BITS / 8,
                ))
            })
            .map(u64::from_be_bytes)
    }
}

impl ValueCodecViaEncoding for ValsetStore {
    type Encoding = Bincode;
}
