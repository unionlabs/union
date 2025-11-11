use std::{collections::BTreeMap, num::NonZero};

use cosmwasm_std::{StdError, StdResult};
use depolama::{
    KeyCodec, Prefix, Store,
    key::KeyCodecViaEncoding,
    value::{ValueCodecViaEncoding, ValueUnitEncoding},
};
use ibc_union_light_client::spec::Timestamp;
use unionlabs::{
    encoding::Bincode,
    primitives::{Bytes, H256, H512},
};

use crate::types::{Attestation, AttestationKey, AttestationValue};

/// `Map<(Height, Timestamp, Key, Value), Map<AttestorPk, AttestationSig>>`
pub enum PendingAttestations {}
impl Store for PendingAttestations {
    const PREFIX: Prefix = Prefix::new(b"pending_attestations");
    type Key = Attestation;
    type Value = BTreeMap<H256, H512>;
}
// not intended to be iterable in any meaningful way, so bincode is fine
impl KeyCodecViaEncoding for PendingAttestations {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for PendingAttestations {
    type Encoding = Bincode;
}

/// `Map<Height, Map<Key, Value>>`
pub enum Attestations {}
impl Store for Attestations {
    const PREFIX: Prefix = Prefix::new(b"attestations");
    type Key = AttestationKey;
    type Value = AttestationValue;
}
// not intended to be iterable in any meaningful way, so bincode is fine
impl KeyCodecViaEncoding for Attestations {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for Attestations {
    type Encoding = Bincode;
}

/// `Map<ChainId, Set<AttestorPk>>`
pub enum Attestors {}
impl Store for Attestors {
    const PREFIX: Prefix = Prefix::new(b"attestors");
    type Key = (String, H256);
    type Value = ();
}
// not intended to be iterable in any meaningful way, so bincode is fine
// note that this storage *is* iterated, but only ever all attestors under
// a specific chain id
impl KeyCodecViaEncoding for Attestors {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for Attestors {
    type Encoding = ValueUnitEncoding;
}

/// `Map<(Height, Timestamp, Key, Value), Map<AttestorPk, AttestationSig>>`
pub enum AttestationAttestors {}
impl Store for AttestationAttestors {
    const PREFIX: Prefix = Prefix::new(b"attestation_attestors");
    type Key = Attestation;
    type Value = BTreeMap<H256, H512>;
}
// not intended to be iterable in any meaningful way, so bincode is fine
impl KeyCodecViaEncoding for AttestationAttestors {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for AttestationAttestors {
    type Encoding = Bincode;
}

/// `Map<(ChainId, Height), Timestamp>`
pub enum HeightTimestamps {}
impl Store for HeightTimestamps {
    const PREFIX: Prefix = Prefix::new(b"height_timestamps");
    type Key = (String, u64);
    type Value = Timestamp;
}
// implement manually since bincode uses LE but we need BE for iteration
impl KeyCodec<(String, u64)> for HeightTimestamps {
    fn encode_key((chain_id, height): &(String, u64)) -> Bytes {
        chain_id
            .as_bytes()
            .iter()
            .copied()
            .chain(height.to_be_bytes())
            .collect()
    }

    fn decode_key(raw: &Bytes) -> StdResult<(String, u64)> {
        if raw.len() < 8 {
            Err(StdError::generic_err(format!(
                "invalid key: expected at least 8 bytes, found {} (raw: {raw})",
                raw.len()
            )))
        } else {
            let height = raw[raw.len() - 8..]
                .try_into()
                .map(u64::from_be_bytes)
                .expect("8 bytes; qed;");

            let chain_id = str::from_utf8(&raw[..raw.len() - 8])
                .map_err(|e| StdError::generic_err(format!("invalid chain id: {e}",)))?
                .to_owned();

            Ok((chain_id, height))
        }
    }
}
impl ValueCodecViaEncoding for HeightTimestamps {
    type Encoding = Bincode;
}

/// `Item<Quorum>`
pub enum Quorum {}
impl Store for Quorum {
    const PREFIX: Prefix = Prefix::new(b"quorum");
    type Key = String;
    type Value = NonZero<u8>;
}
impl KeyCodecViaEncoding for Quorum {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for Quorum {
    type Encoding = Bincode;
}
