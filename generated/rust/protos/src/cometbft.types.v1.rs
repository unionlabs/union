// @generated
/// ValidatorSet defines a set of validators.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorSet {
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    #[prost(message, optional, tag = "2")]
    pub proposer: ::core::option::Option<Validator>,
    #[prost(int64, tag = "3")]
    pub total_voting_power: i64,
}
impl ::prost::Name for ValidatorSet {
    const NAME: &'static str = "ValidatorSet";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Validator represents a node participating in the consensus protocol.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    #[prost(bytes = "vec", tag = "1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub pub_key: ::core::option::Option<super::super::crypto::v1::PublicKey>,
    #[prost(int64, tag = "3")]
    pub voting_power: i64,
    #[prost(int64, tag = "4")]
    pub proposer_priority: i64,
    #[prost(bytes = "vec", tag = "5")]
    pub pub_key_bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "6")]
    pub pub_key_type: ::prost::alloc::string::String,
}
impl ::prost::Name for Validator {
    const NAME: &'static str = "Validator";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// SimpleValidator is a Validator, which is serialized and hashed in consensus.
/// Address is removed because it's redundant with the pubkey.
/// Proposer priority is removed because it changes every round.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleValidator {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::super::crypto::v1::PublicKey>,
    #[prost(int64, tag = "2")]
    pub voting_power: i64,
}
impl ::prost::Name for SimpleValidator {
    const NAME: &'static str = "SimpleValidator";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// BlockIdFlag indicates which BlockID the signature is for
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BlockIdFlag {
    /// Indicates an error condition
    Unknown = 0,
    /// The vote was not received
    Absent = 1,
    /// Voted for the block that received the majority
    Commit = 2,
    /// Voted for nil
    Nil = 3,
}
impl BlockIdFlag {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BlockIdFlag::Unknown => "BLOCK_ID_FLAG_UNKNOWN",
            BlockIdFlag::Absent => "BLOCK_ID_FLAG_ABSENT",
            BlockIdFlag::Commit => "BLOCK_ID_FLAG_COMMIT",
            BlockIdFlag::Nil => "BLOCK_ID_FLAG_NIL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "BLOCK_ID_FLAG_UNKNOWN" => Some(Self::Unknown),
            "BLOCK_ID_FLAG_ABSENT" => Some(Self::Absent),
            "BLOCK_ID_FLAG_COMMIT" => Some(Self::Commit),
            "BLOCK_ID_FLAG_NIL" => Some(Self::Nil),
            _ => None,
        }
    }
}
/// Header of the parts set for a block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartSetHeader {
    #[prost(uint32, tag = "1")]
    pub total: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PartSetHeader {
    const NAME: &'static str = "PartSetHeader";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Part of the block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Part {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<super::super::crypto::v1::Proof>,
}
impl ::prost::Name for Part {
    const NAME: &'static str = "Part";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// BlockID defines the unique ID of a block as its hash and its `PartSetHeader`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockId {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub part_set_header: ::core::option::Option<PartSetHeader>,
}
impl ::prost::Name for BlockId {
    const NAME: &'static str = "BlockID";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Header defines the structure of a block header.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// basic block info
    #[prost(message, optional, tag = "1")]
    pub version: ::core::option::Option<super::super::version::v1::Consensus>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub height: i64,
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// prev block info
    #[prost(message, optional, tag = "5")]
    pub last_block_id: ::core::option::Option<BlockId>,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    #[prost(bytes = "vec", tag = "6")]
    pub last_commit_hash: ::prost::alloc::vec::Vec<u8>,
    /// transactions
    #[prost(bytes = "vec", tag = "7")]
    pub data_hash: ::prost::alloc::vec::Vec<u8>,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    #[prost(bytes = "vec", tag = "8")]
    pub validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// validators for the next block
    #[prost(bytes = "vec", tag = "9")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus params for current block
    #[prost(bytes = "vec", tag = "10")]
    pub consensus_hash: ::prost::alloc::vec::Vec<u8>,
    /// state after txs from the previous block
    #[prost(bytes = "vec", tag = "11")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// root hash of all results from the txs from the previous block
    #[prost(bytes = "vec", tag = "12")]
    pub last_results_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus info
    ///
    /// evidence included in the block
    #[prost(bytes = "vec", tag = "13")]
    pub evidence_hash: ::prost::alloc::vec::Vec<u8>,
    /// original proposer of the block
    #[prost(bytes = "vec", tag = "14")]
    pub proposer_address: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Data contains the set of transactions included in the block
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    /// Txs that will be applied by state @ block.Height+1.
    /// NOTE: not all txs here are valid.  We're just agreeing on the order first.
    /// This means that block.AppHash does not include these txs.
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for Data {
    const NAME: &'static str = "Data";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Vote represents a prevote or precommit vote from validators for
/// consensus.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(int32, tag = "3")]
    pub round: i32,
    /// zero if vote is nil.
    #[prost(message, optional, tag = "4")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    #[prost(bytes = "vec", tag = "6")]
    pub validator_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int32, tag = "7")]
    pub validator_index: i32,
    /// Vote signature by the validator if they participated in consensus for the
    /// associated block.
    #[prost(bytes = "vec", tag = "8")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    /// Vote extension provided by the application. Only valid for precommit
    /// messages.
    #[prost(bytes = "vec", tag = "9")]
    pub extension: ::prost::alloc::vec::Vec<u8>,
    /// Vote extension signature by the validator if they participated in
    /// consensus for the associated block.
    /// Only valid for precommit messages.
    #[prost(bytes = "vec", tag = "10")]
    pub extension_signature: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Vote {
    const NAME: &'static str = "Vote";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Commit contains the evidence that a block was committed by a set of validators.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Commit {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(message, optional, tag = "3")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, repeated, tag = "4")]
    pub signatures: ::prost::alloc::vec::Vec<CommitSig>,
}
impl ::prost::Name for Commit {
    const NAME: &'static str = "Commit";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// CommitSig is a part of the Vote included in a Commit.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitSig {
    #[prost(enumeration = "BlockIdFlag", tag = "1")]
    pub block_id_flag: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub validator_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub timestamp: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    #[prost(bytes = "vec", tag = "4")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CommitSig {
    const NAME: &'static str = "CommitSig";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// ExtendedCommit is a Commit with ExtendedCommitSig.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedCommit {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(message, optional, tag = "3")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, repeated, tag = "4")]
    pub extended_signatures: ::prost::alloc::vec::Vec<ExtendedCommitSig>,
}
impl ::prost::Name for ExtendedCommit {
    const NAME: &'static str = "ExtendedCommit";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// ExtendedCommitSig retains all the same fields as CommitSig but adds vote
/// extension-related fields. We use two signatures to ensure backwards compatibility.
/// That is the digest of the original signature is still the same in prior versions
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedCommitSig {
    #[prost(enumeration = "BlockIdFlag", tag = "1")]
    pub block_id_flag: i32,
    #[prost(bytes = "vec", tag = "2")]
    pub validator_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub timestamp: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    #[prost(bytes = "vec", tag = "4")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    /// Vote extension data
    #[prost(bytes = "vec", tag = "5")]
    pub extension: ::prost::alloc::vec::Vec<u8>,
    /// Vote extension signature
    #[prost(bytes = "vec", tag = "6")]
    pub extension_signature: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ExtendedCommitSig {
    const NAME: &'static str = "ExtendedCommitSig";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Block proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(int32, tag = "3")]
    pub round: i32,
    #[prost(int32, tag = "4")]
    pub pol_round: i32,
    #[prost(message, optional, tag = "5")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, optional, tag = "6")]
    pub timestamp: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    #[prost(bytes = "vec", tag = "7")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Proposal {
    const NAME: &'static str = "Proposal";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// SignedHeader contains a Header(H) and Commit(H+1) with signatures of validators who signed it.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SignedHeader {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub commit: ::core::option::Option<Commit>,
}
impl ::prost::Name for SignedHeader {
    const NAME: &'static str = "SignedHeader";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// LightBlock is a combination of SignedHeader and ValidatorSet. It is used by light clients.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightBlock {
    #[prost(message, optional, tag = "1")]
    pub signed_header: ::core::option::Option<SignedHeader>,
    #[prost(message, optional, tag = "2")]
    pub validator_set: ::core::option::Option<ValidatorSet>,
}
impl ::prost::Name for LightBlock {
    const NAME: &'static str = "LightBlock";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// BlockMeta contains meta information about a block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockMeta {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(int64, tag = "2")]
    pub block_size: i64,
    #[prost(message, optional, tag = "3")]
    pub header: ::core::option::Option<Header>,
    #[prost(int64, tag = "4")]
    pub num_txs: i64,
}
impl ::prost::Name for BlockMeta {
    const NAME: &'static str = "BlockMeta";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// TxProof represents a Merkle proof of the presence of a transaction in the Merkle tree.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxProof {
    #[prost(bytes = "vec", tag = "1")]
    pub root_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<super::super::crypto::v1::Proof>,
}
impl ::prost::Name for TxProof {
    const NAME: &'static str = "TxProof";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// SignedMsgType is a type of signed message in the consensus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SignedMsgType {
    /// Unknown
    Unknown = 0,
    /// Prevote
    Prevote = 1,
    /// Precommit
    Precommit = 2,
    /// Proposal
    Proposal = 32,
}
impl SignedMsgType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SignedMsgType::Unknown => "SIGNED_MSG_TYPE_UNKNOWN",
            SignedMsgType::Prevote => "SIGNED_MSG_TYPE_PREVOTE",
            SignedMsgType::Precommit => "SIGNED_MSG_TYPE_PRECOMMIT",
            SignedMsgType::Proposal => "SIGNED_MSG_TYPE_PROPOSAL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SIGNED_MSG_TYPE_UNKNOWN" => Some(Self::Unknown),
            "SIGNED_MSG_TYPE_PREVOTE" => Some(Self::Prevote),
            "SIGNED_MSG_TYPE_PRECOMMIT" => Some(Self::Precommit),
            "SIGNED_MSG_TYPE_PROPOSAL" => Some(Self::Proposal),
            _ => None,
        }
    }
}
/// ConsensusParams contains consensus critical parameters that determine the
/// validity of blocks.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusParams {
    #[prost(message, optional, tag = "1")]
    pub block: ::core::option::Option<BlockParams>,
    #[prost(message, optional, tag = "2")]
    pub evidence: ::core::option::Option<EvidenceParams>,
    #[prost(message, optional, tag = "3")]
    pub validator: ::core::option::Option<ValidatorParams>,
    #[prost(message, optional, tag = "4")]
    pub version: ::core::option::Option<VersionParams>,
    /// Use FeatureParams.vote_extensions_enable_height instead
    #[deprecated]
    #[prost(message, optional, tag = "5")]
    pub abci: ::core::option::Option<AbciParams>,
    #[prost(message, optional, tag = "6")]
    pub synchrony: ::core::option::Option<SynchronyParams>,
    #[prost(message, optional, tag = "7")]
    pub feature: ::core::option::Option<FeatureParams>,
}
impl ::prost::Name for ConsensusParams {
    const NAME: &'static str = "ConsensusParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// BlockParams define limits on the block size and gas.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockParams {
    /// Maximum size of a block, in bytes.
    ///
    /// Must be greater or equal to -1 and cannot be greater than the hard-coded
    /// maximum block size, which is 100MB.
    ///
    /// If set to -1, the limit is the hard-coded maximum block size.
    #[prost(int64, tag = "1")]
    pub max_bytes: i64,
    /// Maximum gas wanted by transactions included in a block.
    ///
    /// Must be greater or equal to -1. If set to -1, no limit is enforced.
    #[prost(int64, tag = "2")]
    pub max_gas: i64,
}
impl ::prost::Name for BlockParams {
    const NAME: &'static str = "BlockParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// EvidenceParams determine the validity of evidences of Byzantine behavior.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceParams {
    /// Maximum age of evidence, in blocks.
    ///
    /// The recommended formula for calculating it is max_age_duration / {average
    /// block time}.
    #[prost(int64, tag = "1")]
    pub max_age_num_blocks: i64,
    /// Maximum age of evidence, in time.
    ///
    /// The recommended value of is should correspond to the application's
    /// "unbonding period" or other similar mechanism for handling
    /// Nothing-At-Stake attacks.
    /// See: <https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed.>
    #[prost(message, optional, tag = "2")]
    pub max_age_duration: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// Maximum size in bytes of evidence allowed to be included in a block.
    ///
    /// It should fall comfortably under the maximum size of a block.
    #[prost(int64, tag = "3")]
    pub max_bytes: i64,
}
impl ::prost::Name for EvidenceParams {
    const NAME: &'static str = "EvidenceParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// ValidatorParams restrict the public key types validators can use.
///
/// NOTE: uses ABCI public keys naming, not Amino names.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorParams {
    #[prost(string, repeated, tag = "1")]
    pub pub_key_types: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ValidatorParams {
    const NAME: &'static str = "ValidatorParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// VersionParams contain the version of specific components of CometBFT.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionParams {
    /// The ABCI application version.
    ///
    /// It was named app_version in CometBFT 0.34.
    #[prost(uint64, tag = "1")]
    pub app: u64,
}
impl ::prost::Name for VersionParams {
    const NAME: &'static str = "VersionParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// HashedParams is a subset of ConsensusParams.
///
/// It is hashed into the Header.ConsensusHash.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HashedParams {
    #[prost(int64, tag = "1")]
    pub block_max_bytes: i64,
    #[prost(int64, tag = "2")]
    pub block_max_gas: i64,
}
impl ::prost::Name for HashedParams {
    const NAME: &'static str = "HashedParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// SynchronyParams determine the validity of block timestamps.
///
/// These parameters are part of the Proposer-Based Timestamps (PBTS) algorithm.
/// For more information on the relationship of the synchrony parameters to
/// block timestamps validity, refer to the PBTS specification:
/// <https://github.com/tendermint/spec/blob/master/spec/consensus/proposer-based-timestamp/README.md>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SynchronyParams {
    /// Bound for how skewed a proposer's clock may be from any validator on the
    /// network while still producing valid proposals.
    #[prost(message, optional, tag = "1")]
    pub precision: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// Bound for how long a proposal message may take to reach all validators on
    /// a network and still be considered valid.
    #[prost(message, optional, tag = "2")]
    pub message_delay: ::core::option::Option<super::super::super::google::protobuf::Duration>,
}
impl ::prost::Name for SynchronyParams {
    const NAME: &'static str = "SynchronyParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// FeatureParams configure the height from which features of CometBFT are enabled.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FeatureParams {
    /// Height during which vote extensions will be enabled.
    ///
    /// A value of 0 means vote extensions are disabled. A value > 0 denotes
    /// the height at which vote extensions will be (or have been) enabled.
    ///
    /// During the specified height, and for all subsequent heights, precommit
    /// messages that do not contain valid extension data will be considered
    /// invalid. Prior to this height, or when this height is set to 0, vote
    /// extensions will not be used or accepted by validators on the network.
    ///
    /// Once enabled, vote extensions will be created by the application in
    /// ExtendVote, validated by the application in VerifyVoteExtension, and
    /// used by the application in PrepareProposal, when proposing the next block.
    ///
    /// Cannot be set to heights lower or equal to the current blockchain height.
    #[prost(message, optional, tag = "1")]
    pub vote_extensions_enable_height:
        ::core::option::Option<super::super::super::google::protobuf::Int64Value>,
    /// Height at which Proposer-Based Timestamps (PBTS) will be enabled.
    ///
    /// A value of 0 means PBTS is disabled. A value > 0 denotes the height at
    /// which PBTS will be (or has been) enabled.
    ///
    /// From the specified height, and for all subsequent heights, the PBTS
    /// algorithm will be used to produce and validate block timestamps. Prior to
    /// this height, or when this height is set to 0, the legacy BFT Time
    /// algorithm is used to produce and validate timestamps.
    ///
    /// Cannot be set to heights lower or equal to the current blockchain height.
    #[prost(message, optional, tag = "2")]
    pub pbts_enable_height:
        ::core::option::Option<super::super::super::google::protobuf::Int64Value>,
}
impl ::prost::Name for FeatureParams {
    const NAME: &'static str = "FeatureParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// ABCIParams is deprecated and its contents moved to FeatureParams
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciParams {
    /// vote_extensions_enable_height has been deprecated.
    /// Instead, use FeatureParams.vote_extensions_enable_height.
    #[prost(int64, tag = "1")]
    pub vote_extensions_enable_height: i64,
}
impl ::prost::Name for AbciParams {
    const NAME: &'static str = "ABCIParams";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Evidence is a generic type for wrapping evidence of misbehavior by a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Evidence {
    /// The type of evidence.
    #[prost(oneof = "evidence::Sum", tags = "1, 2")]
    pub sum: ::core::option::Option<evidence::Sum>,
}
/// Nested message and enum types in `Evidence`.
pub mod evidence {
    /// The type of evidence.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Sum {
        #[prost(message, tag = "1")]
        DuplicateVoteEvidence(super::DuplicateVoteEvidence),
        #[prost(message, tag = "2")]
        LightClientAttackEvidence(super::LightClientAttackEvidence),
    }
}
impl ::prost::Name for Evidence {
    const NAME: &'static str = "Evidence";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// DuplicateVoteEvidence contains evidence of a validator signed two conflicting votes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DuplicateVoteEvidence {
    #[prost(message, optional, tag = "1")]
    pub vote_a: ::core::option::Option<Vote>,
    #[prost(message, optional, tag = "2")]
    pub vote_b: ::core::option::Option<Vote>,
    #[prost(int64, tag = "3")]
    pub total_voting_power: i64,
    #[prost(int64, tag = "4")]
    pub validator_power: i64,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
}
impl ::prost::Name for DuplicateVoteEvidence {
    const NAME: &'static str = "DuplicateVoteEvidence";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// LightClientAttackEvidence contains evidence of a set of validators attempting to mislead a light client.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightClientAttackEvidence {
    #[prost(message, optional, tag = "1")]
    pub conflicting_block: ::core::option::Option<LightBlock>,
    #[prost(int64, tag = "2")]
    pub common_height: i64,
    #[prost(message, repeated, tag = "3")]
    pub byzantine_validators: ::prost::alloc::vec::Vec<Validator>,
    #[prost(int64, tag = "4")]
    pub total_voting_power: i64,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
}
impl ::prost::Name for LightClientAttackEvidence {
    const NAME: &'static str = "LightClientAttackEvidence";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// EvidenceList is a list of evidence.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceList {
    #[prost(message, repeated, tag = "1")]
    pub evidence: ::prost::alloc::vec::Vec<Evidence>,
}
impl ::prost::Name for EvidenceList {
    const NAME: &'static str = "EvidenceList";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// Block defines the structure of a block in the CometBFT blockchain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Block {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<Header>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<Data>,
    #[prost(message, optional, tag = "3")]
    pub evidence: ::core::option::Option<EvidenceList>,
    #[prost(message, optional, tag = "4")]
    pub last_commit: ::core::option::Option<Commit>,
}
impl ::prost::Name for Block {
    const NAME: &'static str = "Block";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// EventDataRoundState is emitted with each new round step.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventDataRoundState {
    #[prost(int64, tag = "1")]
    pub height: i64,
    #[prost(int32, tag = "2")]
    pub round: i32,
    #[prost(string, tag = "3")]
    pub step: ::prost::alloc::string::String,
}
impl ::prost::Name for EventDataRoundState {
    const NAME: &'static str = "EventDataRoundState";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// CanonicalBlockID is a canonical representation of a BlockID, which gets
/// serialized and signed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalBlockId {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub part_set_header: ::core::option::Option<CanonicalPartSetHeader>,
}
impl ::prost::Name for CanonicalBlockId {
    const NAME: &'static str = "CanonicalBlockID";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// CanonicalPartSetHeader is a canonical representation of a PartSetHeader,
/// which gets serialized and signed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalPartSetHeader {
    #[prost(uint32, tag = "1")]
    pub total: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CanonicalPartSetHeader {
    const NAME: &'static str = "CanonicalPartSetHeader";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// CanonicalProposal is a canonical representation of a Proposal, which gets
/// serialized and signed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalProposal {
    /// type alias for byte
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "2")]
    pub height: i64,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "3")]
    pub round: i64,
    #[prost(int64, tag = "4")]
    pub pol_round: i64,
    #[prost(message, optional, tag = "5")]
    pub block_id: ::core::option::Option<CanonicalBlockId>,
    #[prost(message, optional, tag = "6")]
    pub timestamp: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    #[prost(string, tag = "7")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for CanonicalProposal {
    const NAME: &'static str = "CanonicalProposal";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// CanonicalVote is a canonical representation of a Vote, which gets
/// serialized and signed.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalVote {
    /// type alias for byte
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "2")]
    pub height: i64,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "3")]
    pub round: i64,
    #[prost(message, optional, tag = "4")]
    pub block_id: ::core::option::Option<CanonicalBlockId>,
    #[prost(string, tag = "6")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for CanonicalVote {
    const NAME: &'static str = "CanonicalVote";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
/// CanonicalVoteExtension provides us a way to serialize a vote extension from
/// a particular validator such that we can sign over those serialized bytes.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CanonicalVoteExtension {
    #[prost(bytes = "vec", tag = "1")]
    pub extension: ::prost::alloc::vec::Vec<u8>,
    #[prost(sfixed64, tag = "2")]
    pub height: i64,
    #[prost(sfixed64, tag = "3")]
    pub round: i64,
    #[prost(string, tag = "4")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for CanonicalVoteExtension {
    const NAME: &'static str = "CanonicalVoteExtension";
    const PACKAGE: &'static str = "cometbft.types.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.types.v1.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
