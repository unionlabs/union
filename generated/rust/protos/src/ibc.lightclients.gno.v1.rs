#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Block {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<GnoHeader>,
    #[prost(message, optional, tag = "2")]
    pub data: ::core::option::Option<Data>,
    #[prost(message, optional, tag = "3")]
    pub last_commit: ::core::option::Option<Commit>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct BlockId {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub parts_header: ::core::option::Option<PartSetHeader>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Commit {
    #[prost(message, optional, tag = "1")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, repeated, tag = "2")]
    pub precommits: ::prost::alloc::vec::Vec<CommitSig>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct CommitSig {
    #[prost(uint32, tag = "1")]
    pub r#type: u32,
    #[prost(sint64, tag = "2")]
    pub height: i64,
    #[prost(sint64, tag = "3")]
    pub round: i64,
    #[prost(message, optional, tag = "4")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<super::super::super::super::google::protobuf::Timestamp>,
    #[prost(string, tag = "6")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(sint64, tag = "7")]
    pub validator_index: i64,
    #[prost(bytes = "vec", tag = "8")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Data {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GnoHeader {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(sint64, tag = "3")]
    pub height: i64,
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<super::super::super::super::google::protobuf::Timestamp>,
    #[prost(sint64, tag = "5")]
    pub num_txs: i64,
    #[prost(sint64, tag = "6")]
    pub total_txs: i64,
    #[prost(string, tag = "7")]
    pub app_version: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "8")]
    pub last_block_id: ::core::option::Option<BlockId>,
    #[prost(bytes = "vec", tag = "9")]
    pub last_commit_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "10")]
    pub data_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "11")]
    pub validators_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "12")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "13")]
    pub consensus_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "14")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "15")]
    pub last_results_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag = "16")]
    pub proposer_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct LightBlock {
    #[prost(message, optional, tag = "1")]
    pub signed_header: ::core::option::Option<SignedHeader>,
    #[prost(message, optional, tag = "2")]
    pub validator_set: ::core::option::Option<ValidatorSet>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct PartSet {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct PartSetHeader {
    #[prost(sint64, tag = "1")]
    pub total: i64,
    #[prost(bytes = "vec", tag = "2")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct SignedHeader {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<GnoHeader>,
    #[prost(message, optional, tag = "2")]
    pub commit: ::core::option::Option<Commit>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Validator {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub pub_key: ::core::option::Option<super::super::super::super::tendermint::crypto::PublicKey>,
    #[prost(sint64, tag = "3")]
    pub voting_power: i64,
    #[prost(sint64, tag = "4")]
    pub proposer_priority: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct ValidatorSet {
    #[prost(message, repeated, tag = "1")]
    pub validators: ::prost::alloc::vec::Vec<Validator>,
    #[prost(message, optional, tag = "2")]
    pub proposer: ::core::option::Option<Validator>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Vote {
    #[prost(uint32, tag = "1")]
    pub r#type: u32,
    #[prost(sint64, tag = "2")]
    pub height: i64,
    #[prost(sint64, tag = "3")]
    pub round: i64,
    #[prost(message, optional, tag = "4")]
    pub block_id: ::core::option::Option<BlockId>,
    #[prost(message, optional, tag = "5")]
    pub timestamp: ::core::option::Option<super::super::super::super::google::protobuf::Timestamp>,
    #[prost(string, tag = "6")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(sint64, tag = "7")]
    pub validator_index: i64,
    #[prost(bytes = "vec", tag = "8")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
/// ClientState from Gno tracks the current validator set, latest height,
/// and a possible frozen height.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct ClientState {
    #[prost(string, tag = "1")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub trust_level: ::core::option::Option<Fraction>,
    /// duration of the period since the LatestTimestamp during which the
    /// submitted headers are valid for upgrade
    #[prost(message, optional, tag = "3")]
    pub trusting_period:
        ::core::option::Option<super::super::super::super::google::protobuf::Duration>,
    /// duration of the staking unbonding period
    #[prost(message, optional, tag = "4")]
    pub unbonding_period:
        ::core::option::Option<super::super::super::super::google::protobuf::Duration>,
    /// defines how much new (untrusted) header's Time can drift into the future.
    #[prost(message, optional, tag = "5")]
    pub max_clock_drift:
        ::core::option::Option<super::super::super::super::google::protobuf::Duration>,
    /// Block height when the client was frozen due to a misbehaviour
    #[prost(message, optional, tag = "6")]
    pub frozen_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
    /// Latest height the client was updated to
    #[prost(message, optional, tag = "7")]
    pub latest_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
    /// Proof specifications used in verifying counterparty state
    #[prost(message, repeated, tag = "8")]
    pub proof_specs:
        ::prost::alloc::vec::Vec<super::super::super::super::cosmos::ics23::v1::ProofSpec>,
    /// Path at which next upgraded client will be committed.
    /// Each element corresponds to the key for a single CommitmentProof in the
    /// chained proof. NOTE: ClientState must stored under
    /// `{upgradePath}/{upgradeHeight}/clientState` ConsensusState must be stored
    /// under `{upgradepath}/{upgradeHeight}/consensusState` For SDK chains using
    /// the default upgrade module, upgrade_path should be \[\]string{"upgrade",
    /// "upgradedIBCState"}`
    #[prost(string, repeated, tag = "9")]
    pub upgrade_path: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// allow_update_after_expiry is deprecated
    #[deprecated]
    #[prost(bool, tag = "10")]
    pub allow_update_after_expiry: bool,
    /// allow_update_after_misbehaviour is deprecated
    #[deprecated]
    #[prost(bool, tag = "11")]
    pub allow_update_after_misbehaviour: bool,
    /// In order to distinguish between Gno and Tendermint light clients
    /// we add a client type field. This is useful for clients that
    /// may support multiple light client types.
    #[prost(string, tag = "12")]
    pub lc_type: ::prost::alloc::string::String,
}
/// ConsensusState defines the consensus state from Gno.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct ConsensusState {
    /// timestamp that corresponds to the block height in which the ConsensusState
    /// was stored.
    #[prost(message, optional, tag = "1")]
    pub timestamp: ::core::option::Option<super::super::super::super::google::protobuf::Timestamp>,
    /// commitment root (i.e app hash)
    #[prost(message, optional, tag = "2")]
    pub root: ::core::option::Option<super::super::super::core::commitment::v1::MerkleRoot>,
    #[prost(bytes = "vec", tag = "3")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// In order to distinguish between Gno and Tendermint light clients
    /// we add a client type field. This is useful for clients that
    /// may support multiple light client types.
    #[prost(string, tag = "4")]
    pub lc_type: ::prost::alloc::string::String,
}
/// Fraction defines the protobuf message type for tmmath.Fraction that only
/// supports positive values.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Fraction {
    #[prost(uint64, tag = "1")]
    pub numerator: u64,
    #[prost(uint64, tag = "2")]
    pub denominator: u64,
}
/// Header defines the Tendermint client consensus Header.
/// It encapsulates all the information necessary to update from a trusted
/// Tendermint ConsensusState. The inclusion of TrustedHeight and
/// TrustedValidators allows this update to process correctly, so long as the
/// ConsensusState for the TrustedHeight exists, this removes race conditions
/// among relayers The SignedHeader and ValidatorSet are the new untrusted update
/// fields for the client. The TrustedHeight is the height of a stored
/// ConsensusState on the client that will be used to verify the new untrusted
/// header. The Trusted ConsensusState must be within the unbonding period of
/// current time in order to correctly verify, and the TrustedValidators must
/// hash to TrustedConsensusState.NextValidatorsHash since that is the last
/// trusted validator set at the TrustedHeight.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Header {
    #[prost(message, optional, tag = "1")]
    pub signed_header: ::core::option::Option<SignedHeader>,
    #[prost(message, optional, tag = "2")]
    pub validator_set: ::core::option::Option<ValidatorSet>,
    #[prost(message, optional, tag = "3")]
    pub trusted_height: ::core::option::Option<super::super::super::core::client::v1::Height>,
    #[prost(message, optional, tag = "4")]
    pub trusted_validators: ::core::option::Option<ValidatorSet>,
}
/// Misbehaviour is a wrapper over two conflicting Headers
/// that implements Misbehaviour interface expected by ICS-02
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Misbehaviour {
    /// ClientID is deprecated
    #[deprecated]
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub header_1: ::core::option::Option<Header>,
    #[prost(message, optional, tag = "3")]
    pub header_2: ::core::option::Option<Header>,
}
impl ::prost::Name for Block {
    const NAME: &'static str = "Block";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for BlockId {
    const NAME: &'static str = "BlockID";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for ClientState {
    const NAME: &'static str = "ClientState";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Commit {
    const NAME: &'static str = "Commit";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for CommitSig {
    const NAME: &'static str = "CommitSig";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for ConsensusState {
    const NAME: &'static str = "ConsensusState";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Data {
    const NAME: &'static str = "Data";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Fraction {
    const NAME: &'static str = "Fraction";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GnoHeader {
    const NAME: &'static str = "GnoHeader";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Header {
    const NAME: &'static str = "Header";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for LightBlock {
    const NAME: &'static str = "LightBlock";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Misbehaviour {
    const NAME: &'static str = "Misbehaviour";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for PartSet {
    const NAME: &'static str = "PartSet";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for PartSetHeader {
    const NAME: &'static str = "PartSetHeader";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for SignedHeader {
    const NAME: &'static str = "SignedHeader";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Validator {
    const NAME: &'static str = "Validator";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for ValidatorSet {
    const NAME: &'static str = "ValidatorSet";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Vote {
    const NAME: &'static str = "Vote";
    const PACKAGE: &'static str = "ibc.lightclients.gno.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.lightclients.gno.v1.{}", Self::NAME)
    }
}
