use std::{collections::BTreeMap, num::NonZero};

use cosmwasm_std::{StdError, StdResult};
use depolama::{
    key::KeyCodecViaEncoding,
    value::{ValueCodecViaEncoding, ValueUnitEncoding},
    KeyCodec, Prefix, Store,
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
impl KeyCodecViaEncoding for Attestations {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for Attestations {
    type Encoding = Bincode;
}

/// `Set<AttestorPk>`
pub enum Attestors {}
impl Store for Attestors {
    const PREFIX: Prefix = Prefix::new(b"attestors");
    type Key = H256;
    type Value = ();
}
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
impl KeyCodecViaEncoding for AttestationAttestors {
    type Encoding = Bincode;
}
impl ValueCodecViaEncoding for AttestationAttestors {
    type Encoding = Bincode;
}

/// `Map<Height, Timestamp>`
pub enum HeightTimestamps {}
impl Store for HeightTimestamps {
    const PREFIX: Prefix = Prefix::new(b"height_timestamps");
    type Key = u64;
    type Value = Timestamp;
}
// implement manually since bincode uses LE but we need BE for iteration
impl KeyCodec<u64> for HeightTimestamps {
    fn encode_key(key: &u64) -> Bytes {
        key.to_be_bytes().into()
    }

    fn decode_key(raw: &Bytes) -> StdResult<u64> {
        raw.try_into().map(u64::from_be_bytes).map_err(|_| {
            StdError::generic_err(format!(
                "invalid key: expected 8 bytes, found {} (raw: {raw})",
                raw.len()
            ))
        })
    }
}
impl ValueCodecViaEncoding for HeightTimestamps {
    type Encoding = Bincode;
}

/// `Item<Quorum>`
pub enum Quorum {}
impl Store for Quorum {
    const PREFIX: Prefix = Prefix::new(b"quorum");
    type Key = ();
    type Value = NonZero<u8>;
}
impl ValueCodecViaEncoding for Quorum {
    type Encoding = Bincode;
}
