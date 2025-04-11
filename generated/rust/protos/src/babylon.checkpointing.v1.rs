// @generated
/// BlsKey wraps BLS public key with PoP
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlsKey {
    /// pubkey is the BLS public key of a validator
    #[prost(bytes = "vec", tag = "1")]
    pub pubkey: ::prost::alloc::vec::Vec<u8>,
    /// pop is the proof-of-possession of the BLS key
    #[prost(message, optional, tag = "2")]
    pub pop: ::core::option::Option<ProofOfPossession>,
}
impl ::prost::Name for BlsKey {
    const NAME: &'static str = "BlsKey";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// ProofOfPossession defines proof for the ownership of Ed25519 and BLS private
/// keys
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProofOfPossession {
    /// ed25519_sig is used for verification, ed25519_sig = sign(key = Ed25519_sk,
    /// data = BLS_pk)
    #[prost(bytes = "vec", tag = "1")]
    pub ed25519_sig: ::prost::alloc::vec::Vec<u8>,
    /// bls_sig is the result of PoP, bls_sig = sign(key = BLS_sk, data =
    /// ed25519_sig)
    #[prost(bytes = "vec", tag = "2")]
    pub bls_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ProofOfPossession {
    const NAME: &'static str = "ProofOfPossession";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// ValidatorWithBLSSet defines a set of validators with their BLS public keys
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorWithBlsKeySet {
    #[prost(message, repeated, tag = "1")]
    pub val_set: ::prost::alloc::vec::Vec<ValidatorWithBlsKey>,
}
impl ::prost::Name for ValidatorWithBlsKeySet {
    const NAME: &'static str = "ValidatorWithBlsKeySet";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// ValidatorWithBlsKey couples validator address, voting power, and its bls
/// public key
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorWithBlsKey {
    /// validator_address is the address of the validator
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// bls_pub_key is the BLS public key of the validator
    #[prost(bytes = "vec", tag = "2")]
    pub bls_pub_key: ::prost::alloc::vec::Vec<u8>,
    /// voting_power is the voting power of the validator at the given epoch
    #[prost(uint64, tag = "3")]
    pub voting_power: u64,
}
impl ::prost::Name for ValidatorWithBlsKey {
    const NAME: &'static str = "ValidatorWithBlsKey";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// VoteExtension defines the structure used to create a BLS vote extension.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VoteExtension {
    /// signer is the address of the vote extension signer
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// validator_address is the address of the validator
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// block_hash is the hash of the block that the vote extension is signed over
    #[prost(bytes = "vec", tag = "3")]
    pub block_hash: ::prost::alloc::vec::Vec<u8>,
    /// epoch_num is the epoch number of the vote extension
    #[prost(uint64, tag = "4")]
    pub epoch_num: u64,
    /// height is the height of the vote extension
    #[prost(uint64, tag = "5")]
    pub height: u64,
    /// bls_sig is the BLS signature
    #[prost(bytes = "vec", tag = "6")]
    pub bls_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for VoteExtension {
    const NAME: &'static str = "VoteExtension";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// RawCheckpoint wraps the BLS multi sig with metadata
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawCheckpoint {
    /// epoch_num defines the epoch number the raw checkpoint is for
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
    /// block_hash defines the 'BlockID.Hash', which is the hash of
    /// the block that individual BLS sigs are signed on
    #[prost(bytes = "vec", tag = "2")]
    pub block_hash: ::prost::alloc::vec::Vec<u8>,
    /// bitmap defines the bitmap that indicates the signers of the BLS multi sig
    #[prost(bytes = "vec", tag = "3")]
    pub bitmap: ::prost::alloc::vec::Vec<u8>,
    /// bls_multi_sig defines the multi sig that is aggregated from individual BLS
    /// sigs
    #[prost(bytes = "vec", tag = "4")]
    pub bls_multi_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for RawCheckpoint {
    const NAME: &'static str = "RawCheckpoint";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// RawCheckpointWithMeta wraps the raw checkpoint with metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawCheckpointWithMeta {
    #[prost(message, optional, tag = "1")]
    pub ckpt: ::core::option::Option<RawCheckpoint>,
    /// status defines the status of the checkpoint
    #[prost(enumeration = "CheckpointStatus", tag = "2")]
    pub status: i32,
    /// bls_aggr_pk defines the aggregated BLS public key
    #[prost(bytes = "vec", tag = "3")]
    pub bls_aggr_pk: ::prost::alloc::vec::Vec<u8>,
    /// power_sum defines the accumulated voting power for the checkpoint
    #[prost(uint64, tag = "4")]
    pub power_sum: u64,
    /// lifecycle defines the lifecycle of this checkpoint, i.e., each state
    /// transition and the time (in both timestamp and block height) of this
    /// transition.
    #[prost(message, repeated, tag = "5")]
    pub lifecycle: ::prost::alloc::vec::Vec<CheckpointStateUpdate>,
}
impl ::prost::Name for RawCheckpointWithMeta {
    const NAME: &'static str = "RawCheckpointWithMeta";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// MsgInjectedCheckpoint wraps the checkpoint and the extended votes
/// Note: this is a special message type that is only for internal ABCI++ usage
/// for inserting checkpoint into the block
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgInjectedCheckpoint {
    #[prost(message, optional, tag = "1")]
    pub ckpt: ::core::option::Option<RawCheckpointWithMeta>,
    /// extended_commit_info is the commit info including the vote extensions
    /// from the previous proposal
    #[prost(message, optional, tag = "2")]
    pub extended_commit_info:
        ::core::option::Option<super::super::super::tendermint::abci::ExtendedCommitInfo>,
}
impl ::prost::Name for MsgInjectedCheckpoint {
    const NAME: &'static str = "MsgInjectedCheckpoint";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// CheckpointStateUpdate defines a state transition on the checkpoint.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CheckpointStateUpdate {
    /// state defines the event of a state transition towards this state
    #[prost(enumeration = "CheckpointStatus", tag = "1")]
    pub state: i32,
    /// block_height is the height of the Babylon block that triggers the state
    /// update
    #[prost(uint64, tag = "2")]
    pub block_height: u64,
    /// block_time is the timestamp in the Babylon block that triggers the state
    /// update
    #[prost(message, optional, tag = "3")]
    pub block_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for CheckpointStateUpdate {
    const NAME: &'static str = "CheckpointStateUpdate";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// BlsSig wraps the BLS sig with metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlsSig {
    /// epoch_num defines the epoch number that the BLS sig is signed on
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
    /// block_hash defines the 'BlockID.Hash', which is the hash of
    /// the block that individual BLS sigs are signed on
    #[prost(bytes = "vec", tag = "2")]
    pub block_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "3")]
    pub bls_sig: ::prost::alloc::vec::Vec<u8>,
    /// can't find cosmos_proto.scalar when compiling due to cosmos v0.45.4 does
    /// not support scalar string signer_address = 4 [(cosmos_proto.scalar) =
    /// "cosmos.AddressString"]
    /// the signer_address defines the address of the
    /// signer
    #[prost(string, tag = "4")]
    pub signer_address: ::prost::alloc::string::String,
    /// validator_address defines the validator's consensus address
    #[prost(string, tag = "5")]
    pub validator_address: ::prost::alloc::string::String,
}
impl ::prost::Name for BlsSig {
    const NAME: &'static str = "BlsSig";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// CheckpointStatus is the status of a checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CheckpointStatus {
    /// ACCUMULATING defines a checkpoint that is awaiting for BLS signatures.
    CkptStatusAccumulating = 0,
    /// SEALED defines a checkpoint that has accumulated sufficient BLS signatures.
    CkptStatusSealed = 1,
    /// SUBMITTED defines a checkpoint that is included on BTC.
    CkptStatusSubmitted = 2,
    /// CONFIRMED defines a checkpoint that is k-deep on BTC.
    CkptStatusConfirmed = 3,
    /// FINALIZED defines a checkpoint that is w-deep on BTC.
    CkptStatusFinalized = 4,
}
impl CheckpointStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CheckpointStatus::CkptStatusAccumulating => "CKPT_STATUS_ACCUMULATING",
            CheckpointStatus::CkptStatusSealed => "CKPT_STATUS_SEALED",
            CheckpointStatus::CkptStatusSubmitted => "CKPT_STATUS_SUBMITTED",
            CheckpointStatus::CkptStatusConfirmed => "CKPT_STATUS_CONFIRMED",
            CheckpointStatus::CkptStatusFinalized => "CKPT_STATUS_FINALIZED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CKPT_STATUS_ACCUMULATING" => Some(Self::CkptStatusAccumulating),
            "CKPT_STATUS_SEALED" => Some(Self::CkptStatusSealed),
            "CKPT_STATUS_SUBMITTED" => Some(Self::CkptStatusSubmitted),
            "CKPT_STATUS_CONFIRMED" => Some(Self::CkptStatusConfirmed),
            "CKPT_STATUS_FINALIZED" => Some(Self::CkptStatusFinalized),
            _ => None,
        }
    }
}
/// EventCheckpointAccumulating is emitted when a checkpoint reaches the
/// `Accumulating` state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCheckpointAccumulating {
    #[prost(message, optional, tag = "1")]
    pub checkpoint: ::core::option::Option<RawCheckpointWithMeta>,
}
impl ::prost::Name for EventCheckpointAccumulating {
    const NAME: &'static str = "EventCheckpointAccumulating";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// EventCheckpointSealed is emitted when a checkpoint reaches the `Sealed`
/// state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCheckpointSealed {
    #[prost(message, optional, tag = "1")]
    pub checkpoint: ::core::option::Option<RawCheckpointWithMeta>,
}
impl ::prost::Name for EventCheckpointSealed {
    const NAME: &'static str = "EventCheckpointSealed";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// EventCheckpointSubmitted is emitted when a checkpoint reaches the `Submitted`
/// state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCheckpointSubmitted {
    #[prost(message, optional, tag = "1")]
    pub checkpoint: ::core::option::Option<RawCheckpointWithMeta>,
}
impl ::prost::Name for EventCheckpointSubmitted {
    const NAME: &'static str = "EventCheckpointSubmitted";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// EventCheckpointConfirmed is emitted when a checkpoint reaches the `Confirmed`
/// state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCheckpointConfirmed {
    #[prost(message, optional, tag = "1")]
    pub checkpoint: ::core::option::Option<RawCheckpointWithMeta>,
}
impl ::prost::Name for EventCheckpointConfirmed {
    const NAME: &'static str = "EventCheckpointConfirmed";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// EventCheckpointFinalized is emitted when a checkpoint reaches the `Finalized`
/// state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCheckpointFinalized {
    #[prost(message, optional, tag = "1")]
    pub checkpoint: ::core::option::Option<RawCheckpointWithMeta>,
}
impl ::prost::Name for EventCheckpointFinalized {
    const NAME: &'static str = "EventCheckpointFinalized";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// EventCheckpointForgotten is emitted when a checkpoint switches to a
/// `Forgotten` state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventCheckpointForgotten {
    #[prost(message, optional, tag = "1")]
    pub checkpoint: ::core::option::Option<RawCheckpointWithMeta>,
}
impl ::prost::Name for EventCheckpointForgotten {
    const NAME: &'static str = "EventCheckpointForgotten";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// EventConflictingCheckpoint is emitted when two conflicting checkpoints are
/// found.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventConflictingCheckpoint {
    #[prost(message, optional, tag = "1")]
    pub conflicting_checkpoint: ::core::option::Option<RawCheckpoint>,
    #[prost(message, optional, tag = "2")]
    pub local_checkpoint: ::core::option::Option<RawCheckpointWithMeta>,
}
impl ::prost::Name for EventConflictingCheckpoint {
    const NAME: &'static str = "EventConflictingCheckpoint";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the checkpointing module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// genesis_keys defines the public keys for the genesis validators
    #[prost(message, repeated, tag = "1")]
    pub genesis_keys: ::prost::alloc::vec::Vec<GenesisKey>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// GenesisKey defines public key information about the genesis validators
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisKey {
    /// validator_address is the address corresponding to a validator
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// bls_key defines the BLS key of the validator at genesis
    #[prost(message, optional, tag = "2")]
    pub bls_key: ::core::option::Option<BlsKey>,
    /// val_pubkey defines the ed25519 public key of the validator at genesis
    #[prost(message, optional, tag = "3")]
    pub val_pubkey: ::core::option::Option<super::super::super::cosmos::crypto::ed25519::PubKey>,
}
impl ::prost::Name for GenesisKey {
    const NAME: &'static str = "GenesisKey";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryRawCheckpointListRequest is the request type for the
/// Query/RawCheckpoints RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRawCheckpointListRequest {
    /// status defines the status of the raw checkpoints of the query
    #[prost(enumeration = "CheckpointStatus", tag = "1")]
    pub status: i32,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryRawCheckpointListRequest {
    const NAME: &'static str = "QueryRawCheckpointListRequest";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryRawCheckpointListResponse is the response type for the
/// Query/RawCheckpoints RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRawCheckpointListResponse {
    /// the order is going from the newest to oldest based on the epoch number
    #[prost(message, repeated, tag = "1")]
    pub raw_checkpoints: ::prost::alloc::vec::Vec<RawCheckpointWithMetaResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryRawCheckpointListResponse {
    const NAME: &'static str = "QueryRawCheckpointListResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryRawCheckpointRequest is the request type for the Query/RawCheckpoint
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRawCheckpointRequest {
    /// epoch_num defines the epoch for the queried checkpoint
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
}
impl ::prost::Name for QueryRawCheckpointRequest {
    const NAME: &'static str = "QueryRawCheckpointRequest";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryRawCheckpointResponse is the response type for the Query/RawCheckpoint
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRawCheckpointResponse {
    #[prost(message, optional, tag = "1")]
    pub raw_checkpoint: ::core::option::Option<RawCheckpointWithMetaResponse>,
}
impl ::prost::Name for QueryRawCheckpointResponse {
    const NAME: &'static str = "QueryRawCheckpointResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryRawCheckpointsRequest is the request type for the Query/RawCheckpoints
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRawCheckpointsRequest {
    /// pagination defines whether to have the pagination in the request
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryRawCheckpointsRequest {
    const NAME: &'static str = "QueryRawCheckpointsRequest";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryRawCheckpointsResponse is the response type for the Query/RawCheckpoints
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRawCheckpointsResponse {
    /// the order is going from the newest to oldest based on the epoch number
    #[prost(message, repeated, tag = "1")]
    pub raw_checkpoints: ::prost::alloc::vec::Vec<RawCheckpointWithMetaResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryRawCheckpointsResponse {
    const NAME: &'static str = "QueryRawCheckpointsResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryBlsPublicKeyListRequest is the request type for the Query/BlsPublicKeys
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBlsPublicKeyListRequest {
    /// epoch_num defines the epoch for the queried bls public keys
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryBlsPublicKeyListRequest {
    const NAME: &'static str = "QueryBlsPublicKeyListRequest";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// BlsPublicKeyListResponse couples validator address, voting power, and its bls
/// public key
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlsPublicKeyListResponse {
    /// validator_address is the address of the validator
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    /// bls_pub_key is the BLS public key of the validator
    #[prost(string, tag = "2")]
    pub bls_pub_key_hex: ::prost::alloc::string::String,
    /// voting_power is the voting power of the validator at the given epoch
    #[prost(uint64, tag = "3")]
    pub voting_power: u64,
}
impl ::prost::Name for BlsPublicKeyListResponse {
    const NAME: &'static str = "BlsPublicKeyListResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryBlsPublicKeyListResponse is the response type for the
/// Query/BlsPublicKeys RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryBlsPublicKeyListResponse {
    #[prost(message, repeated, tag = "1")]
    pub validator_with_bls_keys: ::prost::alloc::vec::Vec<BlsPublicKeyListResponse>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryBlsPublicKeyListResponse {
    const NAME: &'static str = "QueryBlsPublicKeyListResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryEpochStatusRequest is the request type for the Query/EpochStatus
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochStatusRequest {
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
}
impl ::prost::Name for QueryEpochStatusRequest {
    const NAME: &'static str = "QueryEpochStatusRequest";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryEpochStatusResponse is the response type for the Query/EpochStatus
/// RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryEpochStatusResponse {
    #[prost(enumeration = "CheckpointStatus", tag = "1")]
    pub status: i32,
}
impl ::prost::Name for QueryEpochStatusResponse {
    const NAME: &'static str = "QueryEpochStatusResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryRecentEpochStatusCountRequest is the request type for the
/// Query/EpochStatusCount RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRecentEpochStatusCountRequest {
    /// epoch_count is the number of the most recent epochs to include in the
    /// aggregation
    #[prost(uint64, tag = "1")]
    pub epoch_count: u64,
}
impl ::prost::Name for QueryRecentEpochStatusCountRequest {
    const NAME: &'static str = "QueryRecentEpochStatusCountRequest";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryRecentEpochStatusCountResponse is the response type for the
/// Query/EpochStatusCount RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRecentEpochStatusCountResponse {
    #[prost(uint64, tag = "1")]
    pub tip_epoch: u64,
    #[prost(uint64, tag = "2")]
    pub epoch_count: u64,
    #[prost(map = "string, uint64", tag = "3")]
    pub status_count: ::std::collections::HashMap<::prost::alloc::string::String, u64>,
}
impl ::prost::Name for QueryRecentEpochStatusCountResponse {
    const NAME: &'static str = "QueryRecentEpochStatusCountResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryLastCheckpointWithStatusRequest is the request type for the
/// Query/LastCheckpointWithStatus RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLastCheckpointWithStatusRequest {
    #[prost(enumeration = "CheckpointStatus", tag = "1")]
    pub status: i32,
}
impl ::prost::Name for QueryLastCheckpointWithStatusRequest {
    const NAME: &'static str = "QueryLastCheckpointWithStatusRequest";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// QueryLastCheckpointWithStatusResponse is the response type for the
/// Query/LastCheckpointWithStatus RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryLastCheckpointWithStatusResponse {
    #[prost(message, optional, tag = "1")]
    pub raw_checkpoint: ::core::option::Option<RawCheckpointResponse>,
}
impl ::prost::Name for QueryLastCheckpointWithStatusResponse {
    const NAME: &'static str = "QueryLastCheckpointWithStatusResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// RawCheckpointResponse wraps the BLS multi sig with metadata
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawCheckpointResponse {
    /// epoch_num defines the epoch number the raw checkpoint is for
    #[prost(uint64, tag = "1")]
    pub epoch_num: u64,
    /// block_hash_hex defines the 'BlockID.Hash', which is the hash of
    /// the block that individual BLS sigs are signed on as hex string
    #[prost(string, tag = "2")]
    pub block_hash_hex: ::prost::alloc::string::String,
    /// bitmap defines the bitmap that indicates the signers of the BLS multi sig
    #[prost(bytes = "vec", tag = "3")]
    pub bitmap: ::prost::alloc::vec::Vec<u8>,
    /// bls_multi_sig defines the multi sig that is aggregated from individual BLS
    /// sigs
    #[prost(bytes = "vec", tag = "4")]
    pub bls_multi_sig: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for RawCheckpointResponse {
    const NAME: &'static str = "RawCheckpointResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// CheckpointStateUpdateResponse defines a state transition on the checkpoint.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CheckpointStateUpdateResponse {
    /// state defines the event of a state transition towards this state
    #[prost(enumeration = "CheckpointStatus", tag = "1")]
    pub state: i32,
    /// status_desc represents the description of status enum.
    #[prost(string, tag = "2")]
    pub status_desc: ::prost::alloc::string::String,
    /// block_height is the height of the Babylon block that triggers the state
    /// update
    #[prost(uint64, tag = "3")]
    pub block_height: u64,
    /// block_time is the timestamp in the Babylon block that triggers the state
    /// update
    #[prost(message, optional, tag = "4")]
    pub block_time: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for CheckpointStateUpdateResponse {
    const NAME: &'static str = "CheckpointStateUpdateResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// RawCheckpointWithMetaResponse wraps the raw checkpoint with metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RawCheckpointWithMetaResponse {
    #[prost(message, optional, tag = "1")]
    pub ckpt: ::core::option::Option<RawCheckpointResponse>,
    /// status defines the status of the checkpoint
    #[prost(enumeration = "CheckpointStatus", tag = "2")]
    pub status: i32,
    /// status_desc represents the description of status enum.
    #[prost(string, tag = "3")]
    pub status_desc: ::prost::alloc::string::String,
    /// bls_aggr_pk defines the aggregated BLS public key
    #[prost(bytes = "vec", tag = "4")]
    pub bls_aggr_pk: ::prost::alloc::vec::Vec<u8>,
    /// power_sum defines the accumulated voting power for the checkpoint
    #[prost(uint64, tag = "5")]
    pub power_sum: u64,
    /// lifecycle defines the lifecycle of this checkpoint, i.e., each state
    /// transition and the time (in both timestamp and block height) of this
    /// transition.
    #[prost(message, repeated, tag = "6")]
    pub lifecycle: ::prost::alloc::vec::Vec<CheckpointStateUpdateResponse>,
}
impl ::prost::Name for RawCheckpointWithMetaResponse {
    const NAME: &'static str = "RawCheckpointWithMetaResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// MsgWrappedCreateValidator defines a wrapped message to create a validator
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedCreateValidator {
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<BlsKey>,
    #[prost(message, optional, tag = "2")]
    pub msg_create_validator:
        ::core::option::Option<super::super::super::cosmos::staking::v1beta1::MsgCreateValidator>,
}
impl ::prost::Name for MsgWrappedCreateValidator {
    const NAME: &'static str = "MsgWrappedCreateValidator";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
/// MsgWrappedCreateValidatorResponse defines the MsgWrappedCreateValidator
/// response type
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWrappedCreateValidatorResponse {}
impl ::prost::Name for MsgWrappedCreateValidatorResponse {
    const NAME: &'static str = "MsgWrappedCreateValidatorResponse";
    const PACKAGE: &'static str = "babylon.checkpointing.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("babylon.checkpointing.v1.{}", Self::NAME)
    }
}
include!("babylon.checkpointing.v1.tonic.rs");
// @@protoc_insertion_point(module)
