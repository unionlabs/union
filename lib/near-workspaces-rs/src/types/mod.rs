//! Types used in the workspaces crate. A lot of these are types are copied over from near_primitives
//! since those APIs are not yet stable. Once they are, we can directly reference them here, so no
//! changes on the library consumer side is needed. Just keep using these types defined here as-is.

pub(crate) mod account;
pub(crate) mod block;
pub(crate) mod chunk;
pub(crate) mod gas_meter;

#[cfg(feature = "interop_sdk")]
mod sdk;

use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::io;
use std::path::Path;
use std::str::FromStr;

pub use near_account_id::AccountId;
use near_primitives::borsh::{BorshDeserialize, BorshSerialize};

use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::error::{Error, ErrorKind};
use crate::result::Result;

pub use self::account::{AccountDetails, AccountDetailsPatch};
pub use self::chunk::{Chunk, ChunkHeader};
pub use self::gas_meter::GasMeter;

/// Nonce is a unit used to determine the order of transactions in the pool.
pub type Nonce = u64;

/// Gas units used in the execution of transactions. For a more in depth description of
/// how and where it can be used, visit [Gas](https://docs.near.org/docs/concepts/gas).
pub use near_gas::NearGas as Gas;

pub use near_token::NearToken;

/// Height of a specific block
pub type BlockHeight = u64;

/// Shard index, from 0 to NUM_SHARDS - 1.
pub type ShardId = u64;

fn from_base58(s: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    bs58::decode(s).into_vec().map_err(|err| err.into())
}

/// Key types supported for either a [`SecretKey`] or [`PublicKey`]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum KeyType {
    ED25519 = 0,
    SECP256K1 = 1,
}

impl KeyType {
    const fn into_near_keytype(self) -> near_crypto::KeyType {
        match self {
            Self::ED25519 => near_crypto::KeyType::ED25519,
            Self::SECP256K1 => near_crypto::KeyType::SECP256K1,
        }
    }

    const fn from_near_keytype(key_type: near_crypto::KeyType) -> Self {
        match key_type {
            near_crypto::KeyType::ED25519 => Self::ED25519,
            near_crypto::KeyType::SECP256K1 => Self::SECP256K1,
        }
    }

    /// Length of the bytes of the public key associated with this key type.
    pub const fn data_len(&self) -> usize {
        match self {
            Self::ED25519 => 32,
            Self::SECP256K1 => 64,
        }
    }
}

impl Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.into_near_keytype())
    }
}

impl FromStr for KeyType {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let key_type = near_crypto::KeyType::from_str(value)
            .map_err(|e| ErrorKind::DataConversion.custom(e))?;

        Ok(Self::from_near_keytype(key_type))
    }
}

impl TryFrom<u8> for KeyType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::ED25519),
            1 => Ok(Self::SECP256K1),
            unknown_key_type => Err(ErrorKind::DataConversion
                .custom(format!("Unknown key type provided: {unknown_key_type}"))),
        }
    }
}

impl From<PublicKey> for near_crypto::PublicKey {
    fn from(pk: PublicKey) -> Self {
        pk.0
    }
}

/// Public key of an account on chain. Usually created along with a [`SecretKey`]
/// to form a keypair associated to the account.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PublicKey(pub(crate) near_crypto::PublicKey);

#[allow(clippy::len_without_is_empty)] // PublicKey is guaranteed to never be empty due to KeyType restrictions.
impl PublicKey {
    /// Create an empty `PublicKey` with the given [`KeyType`]. This is a zero-ed out public key with the
    /// length of the bytes determined by the associated key type.
    pub fn empty(key_type: KeyType) -> Self {
        Self(near_crypto::PublicKey::empty(key_type.into_near_keytype()))
    }

    /// Create a new [`PublicKey`] from the given bytes. This will return an error if the bytes are not in the
    /// correct format. Expected to have key type be the first byte encoded, with the remaining bytes being the
    /// key data.
    pub fn try_from_parts(key_type: KeyType, bytes: &[u8]) -> Result<Self> {
        let mut buf = Vec::new();
        buf.push(key_type as u8);
        buf.extend(bytes);
        Ok(Self(near_crypto::PublicKey::try_from_slice(&buf).map_err(
            |e| {
                ErrorKind::DataConversion
                    .full(format!("Invalid key data for key type: {key_type}"), e)
            },
        )?))
    }

    #[cfg(feature = "interop_sdk")]
    fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        let key_type = KeyType::try_from(bytes[0])?;
        Ok(Self(
            near_crypto::PublicKey::try_from_slice(bytes).map_err(|e| {
                ErrorKind::DataConversion
                    .full(format!("Invalid key data for key type: {key_type}"), e)
            })?,
        ))
    }

    /// Get the number of bytes this key uses. This will differ depending on the [`KeyType`]. i.e. for
    /// ED25519 keys, this will return 32 + 1, while for SECP256K1 keys, this will return 64 + 1. The +1
    /// is used to store the key type, and will appear at the start of the serialized key.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get the [`KeyType`] of the public key.
    pub fn key_type(&self) -> KeyType {
        KeyType::from_near_keytype(self.0.key_type())
    }

    /// Get the key data of the public key. This is serialized bytes of the public key. This will not
    /// include the key type, and will only contain the raw key data.
    pub fn key_data(&self) -> &[u8] {
        self.0.key_data()
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PublicKey {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let pk = near_crypto::PublicKey::from_str(value)
            .map_err(|e| ErrorKind::DataConversion.custom(e))?;

        Ok(Self(pk))
    }
}

impl BorshSerialize for PublicKey {
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        // NOTE: sdk::PublicKey requires that we serialize the length of the key first, then the key itself.
        // Casted usize to u32 since the length in WASM is only 4 bytes long.
        BorshSerialize::serialize(&(self.len() as u32), writer)?;
        // Serialize key type and key data:
        BorshSerialize::serialize(&self.0, writer)
    }
}

impl BorshDeserialize for PublicKey {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let len: u32 = BorshDeserialize::deserialize_reader(reader)?;
        let pk: near_crypto::PublicKey = BorshDeserialize::deserialize_reader(reader)?;

        // Check that the length of the key matches the length we read from the buffer:
        if pk.len() != len as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Key length of {} does not match length of {} read from buffer",
                    pk.len(),
                    len
                ),
            ));
        }
        Ok(Self(pk))
    }
}

/// Secret key of an account on chain. Usually created along with a [`PublicKey`]
/// to form a keypair associated to the account. To generate a new keypair, use
/// one of the creation methods found here, such as [`SecretKey::from_seed`]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretKey(pub(crate) near_crypto::SecretKey);

impl SecretKey {
    /// Get the [`KeyType`] of the secret key.
    pub fn key_type(&self) -> KeyType {
        KeyType::from_near_keytype(self.0.key_type())
    }

    /// Get the [`PublicKey`] associated to this secret key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey(self.0.public_key())
    }

    /// Generate a new secret key provided the [`KeyType`] and seed.
    pub fn from_seed(key_type: KeyType, seed: &str) -> Self {
        let key_type = key_type.into_near_keytype();
        Self(near_crypto::SecretKey::from_seed(key_type, seed))
    }

    /// Generate a new secret key provided the [`KeyType`]. This will use OS provided entropy
    /// to generate the key.
    pub fn from_random(key_type: KeyType) -> Self {
        let key_type = key_type.into_near_keytype();
        Self(near_crypto::SecretKey::from_random(key_type))
    }
}

impl Display for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SecretKey {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let sk = near_crypto::SecretKey::from_str(value)
            .map_err(|e| ErrorKind::DataConversion.custom(e))?;

        Ok(Self(sk))
    }
}

#[derive(Clone)]
pub struct InMemorySigner {
    pub(crate) account_id: AccountId,
    pub(crate) secret_key: SecretKey,
}

impl InMemorySigner {
    pub fn from_secret_key(account_id: AccountId, secret_key: SecretKey) -> Self {
        Self {
            account_id,
            secret_key,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let signer = near_crypto::InMemorySigner::from_file(path)
            .map_err(|err| ErrorKind::Io.custom(err))?;
        Ok(Self::from_secret_key(
            signer.account_id,
            SecretKey(signer.secret_key),
        ))
    }

    pub(crate) fn inner(&self) -> near_crypto::InMemorySigner {
        near_crypto::InMemorySigner::from_secret_key(
            self.account_id.clone(),
            self.secret_key.0.clone(),
        )
    }
}

// type taken from near_primitives::hash::CryptoHash.
/// CryptoHash is type for storing the hash of a specific block.
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct CryptoHash(pub [u8; 32]);

impl CryptoHash {
    pub(crate) fn hash_bytes(bytes: &[u8]) -> Self {
        let hash = sha2::Sha256::digest(bytes).into();
        Self(hash)
    }
}

impl FromStr for CryptoHash {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = from_base58(s).map_err(|e| ErrorKind::DataConversion.custom(e))?;
        Self::try_from(bytes)
    }
}

impl TryFrom<&[u8]> for CryptoHash {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != 32 {
            return Err(Error::message(
                ErrorKind::DataConversion,
                format!(
                    "incorrect hash length (expected 32, but {} was given)",
                    bytes.len()
                ),
            ));
        }
        let mut buf = [0; 32];
        buf.copy_from_slice(bytes);
        Ok(Self(buf))
    }
}

impl TryFrom<Vec<u8>> for CryptoHash {
    type Error = Error;

    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        <Self as TryFrom<&[u8]>>::try_from(v.as_ref())
    }
}

impl Debug for CryptoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for CryptoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&bs58::encode(self.0).into_string(), f)
    }
}

impl From<near_primitives::hash::CryptoHash> for CryptoHash {
    fn from(hash: near_primitives::hash::CryptoHash) -> Self {
        Self(hash.0)
    }
}

/// Access key provides limited access to an account. Each access key belongs to some account and
/// is identified by a unique (within the account) public key. One account may have large number of
/// access keys. Access keys allow to act on behalf of the account by restricting transactions
/// that can be issued.
#[derive(Clone, Debug)]
pub struct AccessKey {
    /// The nonce for this access key.
    /// NOTE: In some cases the access key needs to be recreated. If the new access key reuses the
    /// same public key, the nonce of the new access key should be equal to the nonce of the old
    /// access key. It's required to avoid replaying old transactions again.
    pub nonce: Nonce,

    /// Defines permissions for this access key.
    pub permission: AccessKeyPermission,
}

impl AccessKey {
    pub fn full_access() -> Self {
        Self {
            nonce: 0,
            permission: AccessKeyPermission::FullAccess,
        }
    }

    pub fn function_call_access(
        receiver_id: &AccountId,
        method_names: &[&str],
        allowance: Option<NearToken>,
    ) -> Self {
        Self {
            nonce: 0,
            permission: AccessKeyPermission::FunctionCall(FunctionCallPermission {
                receiver_id: receiver_id.clone().into(),
                method_names: method_names.iter().map(|s| s.to_string()).collect(),
                allowance,
            }),
        }
    }
}

/// Similar to an [`AccessKey`], but also has the [`PublicKey`] associated with it.
#[derive(Clone, Debug)]
pub struct AccessKeyInfo {
    pub public_key: PublicKey,
    pub access_key: AccessKey,
}

impl From<near_primitives::views::AccessKeyInfoView> for AccessKeyInfo {
    fn from(view: near_primitives::views::AccessKeyInfoView) -> Self {
        Self {
            public_key: PublicKey(view.public_key),
            access_key: view.access_key.into(),
        }
    }
}

/// Defines permissions for AccessKey
#[derive(Clone, Debug)]
pub enum AccessKeyPermission {
    FunctionCall(FunctionCallPermission),

    /// Grants full access to the account.
    /// NOTE: It's used to replace account-level public keys.
    FullAccess,
}

/// Grants limited permission to make transactions with FunctionCallActions
/// The permission can limit the allowed balance to be spent on the prepaid gas.
/// It also restrict the account ID of the receiver for this function call.
/// It also can restrict the method name for the allowed function calls.
#[derive(Clone, Debug)]
pub struct FunctionCallPermission {
    /// Allowance is a balance limit to use by this access key to pay for function call gas and
    /// transaction fees. When this access key is used, both account balance and the allowance is
    /// decreased by the same value.
    /// `None` means unlimited allowance.
    /// NOTE: To change or increase the allowance, the old access key needs to be deleted and a new
    /// access key should be created.
    pub allowance: Option<NearToken>,

    // This isn't an AccountId because already existing records in testnet genesis have invalid
    // values for this field (see: https://github.com/near/nearcore/pull/4621#issuecomment-892099860)
    // we accommodate those by using a string, allowing us to read and parse genesis.
    /// The access key only allows transactions with the given receiver's account id.
    pub receiver_id: String,

    /// A list of method names that can be used. The access key only allows transactions with the
    /// function call of one of the given method names.
    /// Empty list means any method name can be used.
    pub method_names: Vec<String>,
}

impl From<AccessKey> for near_primitives::account::AccessKey {
    fn from(access_key: AccessKey) -> Self {
        Self {
            nonce: access_key.nonce,
            permission: match access_key.permission {
                AccessKeyPermission::FunctionCall(function_call_permission) => {
                    near_primitives::account::AccessKeyPermission::FunctionCall(
                        near_primitives::account::FunctionCallPermission {
                            allowance: function_call_permission.allowance.map(|a| a.as_yoctonear()),
                            receiver_id: function_call_permission.receiver_id,
                            method_names: function_call_permission.method_names,
                        },
                    )
                }
                AccessKeyPermission::FullAccess => {
                    near_primitives::account::AccessKeyPermission::FullAccess
                }
            },
        }
    }
}

impl From<near_primitives::views::AccessKeyView> for AccessKey {
    fn from(access_key: near_primitives::views::AccessKeyView) -> Self {
        Self {
            nonce: access_key.nonce,
            permission: match access_key.permission {
                near_primitives::views::AccessKeyPermissionView::FunctionCall {
                    allowance,
                    receiver_id,
                    method_names,
                } => AccessKeyPermission::FunctionCall(FunctionCallPermission {
                    allowance: allowance.map(NearToken::from_yoctonear),
                    receiver_id,
                    method_names,
                }),
                near_primitives::views::AccessKeyPermissionView::FullAccess => {
                    AccessKeyPermission::FullAccess
                }
            },
        }
    }
}

/// Finality of a transaction or block in which transaction is included in. For more info
/// go to the [NEAR finality](https://docs.near.org/docs/concepts/transaction#finality) docs.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Finality {
    /// Optimistic finality. The latest block recorded on the node that responded to our query
    /// (<1 second delay after the transaction is submitted).
    Optimistic,
    /// Near-final finality. Similarly to `Final` finality, but delay should be roughly 1 second.
    DoomSlug,
    /// Final finality. The block that has been validated on at least 66% of the nodes in the
    /// network. (At max, should be 2 second delay after the transaction is submitted.)
    Final,
}

impl From<Finality> for near_primitives::types::BlockReference {
    fn from(value: Finality) -> Self {
        let value = match value {
            Finality::Optimistic => near_primitives::types::Finality::None,
            Finality::DoomSlug => near_primitives::types::Finality::DoomSlug,
            Finality::Final => near_primitives::types::Finality::Final,
        };
        value.into()
    }
}
