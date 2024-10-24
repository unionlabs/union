// @generated
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
    #[prost(message, optional, tag = "5")]
    pub abci: ::core::option::Option<AbciParams>,
}
impl ::prost::Name for ConsensusParams {
    const NAME: &'static str = "ConsensusParams";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// BlockParams contains limits on the block size.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockParams {
    /// Max block size, in bytes.
    /// Note: must be greater than 0
    #[prost(int64, tag = "1")]
    pub max_bytes: i64,
    /// Max gas per block.
    /// Note: must be greater or equal to -1
    #[prost(int64, tag = "2")]
    pub max_gas: i64,
}
impl ::prost::Name for BlockParams {
    const NAME: &'static str = "BlockParams";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// EvidenceParams determine how we handle evidence of malfeasance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceParams {
    /// Max age of evidence, in blocks.
    ///
    /// The basic formula for calculating this is: MaxAgeDuration / {average block
    /// time}.
    #[prost(int64, tag = "1")]
    pub max_age_num_blocks: i64,
    /// Max age of evidence, in time.
    ///
    /// It should correspond with an app's "unbonding period" or other similar
    /// mechanism for handling [Nothing-At-Stake
    /// attacks](<https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed>).
    #[prost(message, optional, tag = "2")]
    pub max_age_duration: ::core::option::Option<::pbjson_types::Duration>,
    /// This sets the maximum size of total evidence in bytes that can be committed in a single block.
    /// and should fall comfortably under the max block bytes.
    /// Default is 1048576 or 1MB
    #[prost(int64, tag = "3")]
    pub max_bytes: i64,
}
impl ::prost::Name for EvidenceParams {
    const NAME: &'static str = "EvidenceParams";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// ValidatorParams restrict the public key types validators can use.
/// NOTE: uses ABCI pubkey naming, not Amino names.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorParams {
    #[prost(string, repeated, tag = "1")]
    pub pub_key_types: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for ValidatorParams {
    const NAME: &'static str = "ValidatorParams";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// VersionParams contains the ABCI application version.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VersionParams {
    #[prost(uint64, tag = "1")]
    pub app: u64,
}
impl ::prost::Name for VersionParams {
    const NAME: &'static str = "VersionParams";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// ABCIParams configure functionality specific to the Application Blockchain Interface.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciParams {
    /// vote_extensions_enable_height configures the first height during which
    /// vote extensions will be enabled. During this specified height, and for all
    /// subsequent heights, precommit messages that do not contain valid extension data
    /// will be considered invalid. Prior to this height, vote extensions will not
    /// be used or accepted by validators on the network.
    ///
    /// Once enabled, vote extensions will be created by the application in ExtendVote,
    /// passed to the application for validation in VerifyVoteExtension and given
    /// to the application to use when proposing a block during PrepareProposal.
    #[prost(int64, tag = "1")]
    pub vote_extensions_enable_height: i64,
}
impl ::prost::Name for AbciParams {
    const NAME: &'static str = "ABCIParams";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Validator {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag = "3")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub voting_power: i64,
    #[prost(int64, tag = "4")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub proposer_priority: i64,
}
impl ::prost::Name for Validator {
    const NAME: &'static str = "Validator";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleValidator {
    #[prost(message, optional, tag = "1")]
    pub pub_key: ::core::option::Option<super::crypto::PublicKey>,
    #[prost(int64, tag = "2")]
    pub voting_power: i64,
}
impl ::prost::Name for SimpleValidator {
    const NAME: &'static str = "SimpleValidator";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// BlockIdFlag indicates which BlockID the signature is for
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BlockIdFlag {
    /// indicates an error condition
    Unknown = 0,
    /// the vote was not received
    Absent = 1,
    /// voted for the block that received the majority
    Commit = 2,
    /// voted for nil
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
/// PartsetHeader
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PartSetHeader {
    #[prost(uint32, tag = "1")]
    pub total: u32,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for PartSetHeader {
    const NAME: &'static str = "PartSetHeader";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Part {
    #[prost(uint32, tag = "1")]
    pub index: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<super::crypto::Proof>,
}
impl ::prost::Name for Part {
    const NAME: &'static str = "Part";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// BlockID
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockId {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    #[serde(alias = "parts")]
    pub part_set_header: ::core::option::Option<PartSetHeader>,
}
impl ::prost::Name for BlockId {
    const NAME: &'static str = "BlockID";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// Header defines the structure of a block header.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Header {
    /// basic block info
    #[prost(message, optional, tag = "1")]
    pub version: ::core::option::Option<super::version::Consensus>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub height: i64,
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// prev block info
    #[prost(message, optional, tag = "5")]
    pub last_block_id: ::core::option::Option<BlockId>,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    #[prost(bytes = "vec", tag = "6")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub last_commit_hash: ::prost::alloc::vec::Vec<u8>,
    /// transactions
    #[prost(bytes = "vec", tag = "7")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub data_hash: ::prost::alloc::vec::Vec<u8>,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    #[prost(bytes = "vec", tag = "8")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// validators for the next block
    #[prost(bytes = "vec", tag = "9")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus params for current block
    #[prost(bytes = "vec", tag = "10")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub consensus_hash: ::prost::alloc::vec::Vec<u8>,
    /// state after txs from the previous block
    #[prost(bytes = "vec", tag = "11")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    /// root hash of all results from the txs from the previous block
    #[prost(bytes = "vec", tag = "12")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub last_results_hash: ::prost::alloc::vec::Vec<u8>,
    /// consensus info
    ///
    /// evidence included in the block
    #[prost(bytes = "vec", tag = "13")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub evidence_hash: ::prost::alloc::vec::Vec<u8>,
    /// original proposer of the block
    #[prost(bytes = "vec", tag = "14")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub proposer_address: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// Data contains the set of transactions included in the block
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Data {
    /// Txs that will be applied by state @ block.Height+1.
    /// NOTE: not all txs here are valid.  We're just agreeing on the order first.
    /// This means that block.AppHash does not include these txs.
    #[prost(bytes = "vec", repeated, tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::inner_base64"))]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for Data {
    const NAME: &'static str = "Data";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// Vote represents a prevote or precommit vote from validators for
/// consensus.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    #[prost(enumeration = "SignedMsgType", tag = "1")]
    pub r#type: i32,
    #[prost(int64, tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub height: i64,
    #[prost(int32, tag = "3")]
    pub round: i32,
    /// zero if vote is nil.
    #[prost(message, optional, tag = "4")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(bytes = "vec", tag = "6")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub validator_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(int32, tag = "7")]
    pub validator_index: i32,
    /// Vote signature by the validator if they participated in consensus for the
    /// associated block.
    #[prost(bytes = "vec", tag = "8")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]
    pub signature: ::prost::alloc::vec::Vec<u8>,
    /// Vote extension provided by the application. Only valid for precommit
    /// messages.
    #[prost(bytes = "vec", tag = "9")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64_opt_default"))]
    pub extension: ::prost::alloc::vec::Vec<u8>,
    /// Vote extension signature by the validator if they participated in
    /// consensus for the associated block.
    /// Only valid for precommit messages.
    #[prost(bytes = "vec", tag = "10")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64_opt_default"))]
    pub extension_signature: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Vote {
    const NAME: &'static str = "Vote";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// Commit contains the evidence that a block was committed by a set of validators.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Commit {
    #[prost(int64, tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// CommitSig is a part of the Vote included in a Commit.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitSig {
    #[prost(enumeration = "BlockIdFlag", tag = "1")]
    pub block_id_flag: i32,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub validator_address: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    #[cfg_attr(
        feature = "serde",
        serde(with = "::serde_utils::parse_from_rfc3339_string_but_0001_01_01T00_00_00Z_is_none")
    )]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(bytes = "vec", tag = "4")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64_opt_default"))]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for CommitSig {
    const NAME: &'static str = "CommitSig";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
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
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
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
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(bytes = "vec", tag = "7")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Proposal {
    const NAME: &'static str = "Proposal";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// TxProof represents a Merkle proof of the presence of a transaction in the Merkle tree.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TxProof {
    #[prost(bytes = "vec", tag = "1")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_upper_unprefixed"))]
    pub root_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::base64"))]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "3")]
    pub proof: ::core::option::Option<super::crypto::Proof>,
}
impl ::prost::Name for TxProof {
    const NAME: &'static str = "TxProof";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// SignedMsgType is a type of signed message in the consensus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SignedMsgType {
    Unknown = 0,
    /// Votes
    Prevote = 1,
    Precommit = 2,
    /// Proposals
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
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Evidence {
    #[prost(oneof = "evidence::Sum", tags = "1, 2")]
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub sum: ::core::option::Option<evidence::Sum>,
}
/// Nested message and enum types in `Evidence`.
pub mod evidence {
    #[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    #[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
    pub enum Sum {
        #[prost(message, tag = "1")]
        #[serde(rename = "tendermint/DuplicateVoteEvidence")]
        DuplicateVoteEvidence(super::DuplicateVoteEvidence),
        #[prost(message, tag = "2")]
        #[serde(rename = "tendermint/DuplicateVoteEvidence")]
        LightClientAttackEvidence(super::LightClientAttackEvidence),
    }
}
impl ::prost::Name for Evidence {
    const NAME: &'static str = "Evidence";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// DuplicateVoteEvidence contains evidence of a validator signed two conflicting votes.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DuplicateVoteEvidence {
    #[prost(message, optional, tag = "1")]
    pub vote_a: ::core::option::Option<Vote>,
    #[prost(message, optional, tag = "2")]
    pub vote_b: ::core::option::Option<Vote>,
    #[prost(int64, tag = "3")]
    #[serde(alias = "TotalVotingPower")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub total_voting_power: i64,
    #[prost(int64, tag = "4")]
    #[serde(alias = "ValidatorPower")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub validator_power: i64,
    #[prost(message, optional, tag = "5")]
    #[serde(alias = "Timestamp")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for DuplicateVoteEvidence {
    const NAME: &'static str = "DuplicateVoteEvidence";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
/// LightClientAttackEvidence contains evidence of a set of validators attempting to mislead a light client.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LightClientAttackEvidence {
    #[prost(message, optional, tag = "1")]
    pub conflicting_block: ::core::option::Option<LightBlock>,
    #[prost(int64, tag = "2")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub common_height: i64,
    #[prost(message, repeated, tag = "3")]
    pub byzantine_validators: ::prost::alloc::vec::Vec<Validator>,
    #[prost(int64, tag = "4")]
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::string"))]
    pub total_voting_power: i64,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
}
impl ::prost::Name for LightClientAttackEvidence {
    const NAME: &'static str = "LightClientAttackEvidence";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EvidenceList {
    #[prost(message, repeated, tag = "1")]
    pub evidence: ::prost::alloc::vec::Vec<Evidence>,
}
impl ::prost::Name for EvidenceList {
    const NAME: &'static str = "EvidenceList";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
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
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag = "7")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for CanonicalProposal {
    const NAME: &'static str = "CanonicalProposal";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LegacyCanonicalVote {
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
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag = "6")]
    pub chain_id: ::prost::alloc::string::String,
}
impl ::prost::Name for LegacyCanonicalVote {
    const NAME: &'static str = "LegacyCanonicalVote";
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
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
    const PACKAGE: &'static str = "tendermint.types";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.types.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
