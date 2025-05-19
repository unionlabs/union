use checkpoint_summary::CheckpointSummary;
use crypto::AuthorityStrongQuorumSignInfo;
use digest::Digest;
use unionlabs_primitives::{encoding::HexPrefixed, FixedBytes};

pub mod checkpoint_summary;
pub mod client_state;
pub mod committee;
pub mod consensus_state;
pub mod crypto;
pub mod digest;
pub mod header;
pub mod object;
pub mod storage_proof;
pub mod transaction;
pub mod transaction_effects;

pub type ObjectID = FixedBytes<32, HexPrefixed>;

pub type AccountAddress = FixedBytes<32, HexPrefixed>;

pub type SuiAddress = FixedBytes<32, HexPrefixed>;

pub type ObjectRef = (ObjectID, u64, Digest);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum Owner {
    /// Object is exclusively owned by a single address, and is mutable.
    AddressOwner(SuiAddress),
    /// Object is exclusively owned by a single object, and is mutable.
    /// The object ID is converted to SuiAddress as SuiAddress is universal.
    ObjectOwner(SuiAddress),
    /// Object is shared, can be used by any address, and is mutable.
    Shared {
        /// The version at which the object became shared
        initial_shared_version: u64,
    },
    /// Object is immutable, and hence ownership doesn't matter.
    Immutable,
    /// Object is sequenced via consensus. Ownership is managed by the configured authenticator.
    ///
    /// Note: wondering what happened to `V1`? `Shared` above was the V1 of consensus objects.
    ConsensusV2 {
        /// The version at which the object most recently became a consensus object.
        /// This serves the same function as `initial_shared_version`, except it may change
        /// if the object's Owner type changes.
        start_version: u64,
        /// The authentication mode of the object
        authenticator: Box<Authenticator>,
    },
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum Authenticator {
    /// The contained SuiAddress exclusively has all permissions: read, write, delete, transfer
    SingleOwner(SuiAddress),
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Intent {
    pub scope: IntentScope,
    pub version: IntentVersion,
    pub app_id: AppId,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(u8)]
pub enum IntentVersion {
    V0 = 0,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(u8)]
pub enum AppId {
    Sui = 0,
    Narwhal = 1,
    Consensus = 2,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(u8)]
pub enum IntentScope {
    TransactionData = 0,         // Used for a user signature on a transaction data.
    TransactionEffects = 1,      // Used for an authority signature on transaction effects.
    CheckpointSummary = 2,       // Used for an authority signature on a checkpoint summary.
    PersonalMessage = 3,         // Used for a user signature on a personal message.
    SenderSignedTransaction = 4, // Used for an authority signature on a user signed transaction.
    ProofOfPossession = 5, // Used as a signature representing an authority's proof of possession of its authority protocol key.
    HeaderDigest = 6,      // Used for narwhal authority signature on header digest.
    BridgeEventUnused = 7, // for bridge purposes but it's currently not included in messages.
    ConsensusBlock = 8,    // Used for consensus authority signature on block's digest.
    DiscoveryPeers = 9,    // Used for reporting peer addresses in discovery.
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntentMessage<T> {
    pub intent: Intent,
    pub value: T,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct U64(pub u64);

#[cfg(feature = "serde")]
impl serde::Serialize for U64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.0.to_string().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for U64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            Ok(U64(String::deserialize(deserializer)?
                .parse()
                .map_err(serde::de::Error::custom)?))
        } else {
            Ok(U64(u64::deserialize(deserializer)?))
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CertifiedCheckpointSummary {
    pub data: CheckpointSummary,
    pub auth_signature: AuthorityStrongQuorumSignInfo,
}
