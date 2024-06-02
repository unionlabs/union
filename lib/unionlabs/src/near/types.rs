use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::{
    hash::CryptoHash,
    types::{Balance, BlockHeight, MerkleHash},
};
use near_sdk::AccountId;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct MerklePathItem {
    pub hash: MerkleHash,
    pub direction: Direction,
}

pub type MerklePath = Vec<MerklePathItem>;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Direction {
    Left,
    Right,
}

#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    BorshDeserialize,
    BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct BlockHeaderInnerLiteView {
    pub height: BlockHeight,
    pub epoch_id: CryptoHash,
    pub next_epoch_id: CryptoHash,
    pub prev_state_root: CryptoHash,
    pub outcome_root: CryptoHash,
    /// Legacy json number. Should not be used.
    pub timestamp: u64,
    // TODO(aeryz): #[serde(with = "dec_format")]
    pub timestamp_nanosec: u64,
    pub next_bp_hash: CryptoHash,
    pub block_merkle_root: CryptoHash,
}

/// Epoch identifier -- wrapped hash, to make it easier to distinguish.
/// EpochId of epoch T is the hash of last block in T-2
/// EpochId of first two epochs is 0
#[derive(
    Debug,
    Clone,
    Default,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    derive_more::AsRef,
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[as_ref(forward)]
pub struct EpochId(pub CryptoHash);

impl std::str::FromStr for EpochId {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    /// Decodes base58-encoded string into a 32-byte crypto hash.
    fn from_str(epoch_id_str: &str) -> Result<Self, Self::Err> {
        Ok(EpochId(CryptoHash::from_str(epoch_id_str)?))
    }
}

#[derive(BorshSerialize, BorshDeserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct BlockHeaderInnerLite {
    /// Height of this block.
    pub height: BlockHeight,
    /// Epoch start hash of this block's epoch.
    /// Used for retrieving validator information
    pub epoch_id: EpochId,
    pub next_epoch_id: EpochId,
    /// Root hash of the state at the previous block.
    pub prev_state_root: MerkleHash,
    /// Root of the outcomes of transactions and receipts from the previous chunks.
    pub prev_outcome_root: MerkleHash,
    /// Timestamp at which the block was built (number of non-leap-nanoseconds since January 1, 1970 0:00:00 UTC).
    pub timestamp: u64,
    /// Hash of the next epoch block producers set
    pub next_bp_hash: CryptoHash,
    /// Merkle root of block hashes up to the current block.
    pub block_merkle_root: CryptoHash,
}

impl From<BlockHeaderInnerLiteView> for BlockHeaderInnerLite {
    fn from(value: BlockHeaderInnerLiteView) -> Self {
        Self {
            height: value.height,
            epoch_id: EpochId(value.epoch_id),
            next_epoch_id: EpochId(value.next_epoch_id),
            prev_state_root: value.prev_state_root,
            prev_outcome_root: value.outcome_root,
            timestamp: value.timestamp,
            next_bp_hash: value.next_bp_hash,
            block_merkle_root: value.block_merkle_root,
        }
    }
}

#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    BorshDeserialize,
    BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct LightClientBlockView {
    pub prev_block_hash: CryptoHash,
    pub next_block_inner_hash: CryptoHash,
    pub inner_lite: BlockHeaderInnerLiteView,
    pub inner_rest_hash: CryptoHash,
    pub next_bps: Option<Vec<ValidatorStakeView>>,
    pub approvals_after_next: Vec<Option<Box<Signature>>>,
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    Clone,
    Eq,
    PartialEq,
)]
#[serde(tag = "validator_stake_struct_version")]
pub enum ValidatorStakeView {
    V1(ValidatorStakeViewV1),
}

#[derive(
    BorshSerialize,
    BorshDeserialize,
    Debug,
    Clone,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ValidatorStakeViewV1 {
    pub account_id: AccountId,
    // TODO(aeryz): implement the public key type and also impl BorshSerialize for it
    pub public_key: PublicKey,
    // TODO(aeryz): #[serde(with = "dec_format")]
    pub stake: Balance,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PublicKey {
    Ed25519([u8; 32]),
    Secp256k1([u8; 64]),
}

impl FromStr for KeyType {
    type Err = ParseKeyTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let lowercase_key_type = value.to_ascii_lowercase();
        match lowercase_key_type.as_str() {
            "ed25519" => Ok(KeyType::ED25519),
            "secp256k1" => Ok(KeyType::SECP256K1),
            _ => Err(Self::Err::UnknownKeyType {
                unknown_key_type: lowercase_key_type,
            }),
        }
    }
}
fn split_key_type_data(value: &str) -> Result<(KeyType, &str), ParseKeyTypeError> {
    if let Some((prefix, key_data)) = value.split_once(':') {
        Ok((KeyType::from_str(prefix)?, key_data))
    } else {
        // If there is no prefix then we Default to ED25519.
        Ok((KeyType::ED25519, value))
    }
}

impl FromStr for PublicKey {
    type Err = ParseKeyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (key_type, key_data) = split_key_type_data(value)?;
        Ok(match key_type {
            KeyType::ED25519 => Self::Ed25519(decode_bs58(key_data)?),
            KeyType::SECP256K1 => Self::Secp256k1(decode_bs58(key_data)?),
        })
    }
}
#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseKeyTypeError {
    #[error("unknown key type '{unknown_key_type}'")]
    UnknownKeyType { unknown_key_type: String },
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseKeyError {
    #[error("unknown key type '{unknown_key_type}'")]
    UnknownKeyType { unknown_key_type: String },
    #[error("invalid key length: expected the input of {expected_length} bytes, but {received_length} was given")]
    InvalidLength {
        expected_length: usize,
        received_length: usize,
    },
    #[error("invalid key data: {error_message}")]
    InvalidData { error_message: String },
}

impl From<ParseKeyTypeError> for ParseKeyError {
    fn from(err: ParseKeyTypeError) -> Self {
        match err {
            ParseKeyTypeError::UnknownKeyType { unknown_key_type } => {
                Self::UnknownKeyType { unknown_key_type }
            }
        }
    }
}

/// Helper struct which provides Display implementation for bytes slice
/// encoding them using base58.
// TODO(mina86): Get rid of it once bs58 has this feature.  There’s currently PR
// for that: https://github.com/Nullus157/bs58-rs/pull/97
struct Bs58<'a>(&'a [u8]);

impl<'a> core::fmt::Display for Bs58<'a> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        debug_assert!(self.0.len() <= 65);
        // The largest buffer we’re ever encoding is 65-byte long.  Base58
        // increases size of the value by less than 40%.  96-byte buffer is
        // therefore enough to fit the largest value we’re ever encoding.
        let mut buf = [0u8; 96];
        let len = bs58::encode(self.0).into(&mut buf[..]).unwrap();
        let output = &buf[..len];
        // SAFETY: we know that alphabet can only include ASCII characters
        // thus our result is an ASCII string.
        fmt.write_str(unsafe { std::str::from_utf8_unchecked(output) })
    }
}
/// Helper which decodes fixed-length base58-encoded data.
///
/// If the encoded string decodes into a buffer of different length than `N`,
/// returns error.  Similarly returns error if decoding fails.
fn decode_bs58<const N: usize>(encoded: &str) -> Result<[u8; N], DecodeBs58Error> {
    let mut buffer = [0u8; N];
    decode_bs58_impl(&mut buffer[..], encoded)?;
    Ok(buffer)
}

fn decode_bs58_impl(dst: &mut [u8], encoded: &str) -> Result<(), DecodeBs58Error> {
    let expected = dst.len();
    match bs58::decode(encoded).into(dst) {
        Ok(received) if received == expected => Ok(()),
        Ok(received) => Err(DecodeBs58Error::BadLength { expected, received }),
        Err(bs58::decode::Error::BufferTooSmall) => Err(DecodeBs58Error::BadLength {
            expected,
            received: expected.saturating_add(1),
        }),
        Err(err) => Err(DecodeBs58Error::BadData(err.to_string())),
    }
}

enum DecodeBs58Error {
    BadLength { expected: usize, received: usize },
    BadData(String),
}

impl std::convert::From<DecodeBs58Error> for ParseKeyError {
    fn from(err: DecodeBs58Error) -> Self {
        match err {
            DecodeBs58Error::BadLength { expected, received } => ParseKeyError::InvalidLength {
                expected_length: expected,
                received_length: received,
            },
            DecodeBs58Error::BadData(error_message) => Self::InvalidData { error_message },
        }
    }
}

impl core::fmt::Display for PublicKey {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> std::fmt::Result {
        let (key_type, key_data) = match self {
            PublicKey::Ed25519(public_key) => (KeyType::ED25519, public_key.as_slice()),
            PublicKey::Secp256k1(public_key) => (KeyType::SECP256K1, public_key.as_slice()),
        };
        write!(fmt, "{}:{}", key_type, Bs58(key_data))
    }
}

impl serde::Serialize for PublicKey {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as serde::Deserializer<'de>>::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as serde::Deserialize>::deserialize(deserializer)?;
        s.parse()
            .map_err(|err: ParseKeyError| serde::de::Error::custom(err.to_string()))
    }
}

#[derive(Debug, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum KeyType {
    ED25519 = 0,
    SECP256K1 = 1,
}

impl core::fmt::Display for KeyType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(match self {
            KeyType::ED25519 => "ed25519",
            KeyType::SECP256K1 => "secp256k1",
        })
    }
}
impl TryFrom<u8> for KeyType {
    type Error = ParseKeyTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyType::ED25519),
            1 => Ok(KeyType::SECP256K1),
            unknown_key_type => Err(ParseKeyTypeError::UnknownKeyType {
                unknown_key_type: unknown_key_type.to_string(),
            }),
        }
    }
}

impl BorshSerialize for PublicKey {
    // TODO(aeryz): this should have a custom error type
    fn serialize<W: borsh::io::Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        match self {
            PublicKey::Ed25519(public_key) => {
                BorshSerialize::serialize(&0u8, writer)?;
                writer.write_all(public_key.as_slice())?;
            }
            PublicKey::Secp256k1(public_key) => {
                BorshSerialize::serialize(&1u8, writer)?;
                writer.write_all(public_key.as_slice())?;
            }
        }
        Ok(())
    }
}

impl BorshDeserialize for PublicKey {
    fn deserialize_reader<R: borsh::io::Read>(rd: &mut R) -> std::io::Result<Self> {
        let key_type = KeyType::try_from(u8::deserialize_reader(rd)?).unwrap();
        match key_type {
            KeyType::ED25519 => Ok(PublicKey::Ed25519(BorshDeserialize::deserialize_reader(
                rd,
            )?)),
            KeyType::SECP256K1 => Ok(PublicKey::Secp256k1(BorshDeserialize::deserialize_reader(
                rd,
            )?)),
        }
    }
}

// TODO(aeryz): make this the proper type
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    BorshSerialize,
    BorshDeserialize,
)]
pub enum Signature {
    Ed25519(Vec<u8>),
    Secp256k1(Vec<u8>),
}

/// The part of the block approval that is different for endorsements and skips
#[derive(BorshSerialize, BorshDeserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApprovalInner {
    Endorsement(CryptoHash),
    Skip(BlockHeight),
}

#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct UpdateState {}
