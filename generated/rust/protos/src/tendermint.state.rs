// @generated
/// LegacyABCIResponses retains the responses
/// of the legacy ABCI calls during block processing.
/// Note ReponseDeliverTx is renamed to ExecTxResult but they are semantically the same
/// Kept for backwards compatibility for versions prior to v0.38
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LegacyAbciResponses {
    #[prost(message, repeated, tag = "1")]
    pub deliver_txs: ::prost::alloc::vec::Vec<super::abci::ExecTxResult>,
    #[prost(message, optional, tag = "2")]
    pub end_block: ::core::option::Option<ResponseEndBlock>,
    #[prost(message, optional, tag = "3")]
    pub begin_block: ::core::option::Option<ResponseBeginBlock>,
}
impl ::prost::Name for LegacyAbciResponses {
    const NAME: &'static str = "LegacyABCIResponses";
    const PACKAGE: &'static str = "tendermint.state";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.state.{}", Self::NAME)
    }
}
/// ResponseBeginBlock is kept for backwards compatibility for versions prior to v0.38
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBeginBlock {
    #[prost(message, repeated, tag = "1")]
    pub events: ::prost::alloc::vec::Vec<super::abci::Event>,
}
impl ::prost::Name for ResponseBeginBlock {
    const NAME: &'static str = "ResponseBeginBlock";
    const PACKAGE: &'static str = "tendermint.state";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.state.{}", Self::NAME)
    }
}
/// ResponseEndBlock is kept for backwards compatibility for versions prior to v0.38
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEndBlock {
    #[prost(message, repeated, tag = "1")]
    pub validator_updates: ::prost::alloc::vec::Vec<super::abci::ValidatorUpdate>,
    #[prost(message, optional, tag = "2")]
    pub consensus_param_updates: ::core::option::Option<super::types::ConsensusParams>,
    #[prost(message, repeated, tag = "3")]
    pub events: ::prost::alloc::vec::Vec<super::abci::Event>,
}
impl ::prost::Name for ResponseEndBlock {
    const NAME: &'static str = "ResponseEndBlock";
    const PACKAGE: &'static str = "tendermint.state";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.state.{}", Self::NAME)
    }
}
/// ValidatorsInfo represents the latest validator set, or the last height it changed
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidatorsInfo {
    #[prost(message, optional, tag = "1")]
    pub validator_set: ::core::option::Option<super::types::ValidatorSet>,
    #[prost(int64, tag = "2")]
    pub last_height_changed: i64,
}
impl ::prost::Name for ValidatorsInfo {
    const NAME: &'static str = "ValidatorsInfo";
    const PACKAGE: &'static str = "tendermint.state";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.state.{}", Self::NAME)
    }
}
/// ConsensusParamsInfo represents the latest consensus params, or the last height it changed
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusParamsInfo {
    #[prost(message, optional, tag = "1")]
    pub consensus_params: ::core::option::Option<super::types::ConsensusParams>,
    #[prost(int64, tag = "2")]
    pub last_height_changed: i64,
}
impl ::prost::Name for ConsensusParamsInfo {
    const NAME: &'static str = "ConsensusParamsInfo";
    const PACKAGE: &'static str = "tendermint.state";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.state.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AbciResponsesInfo {
    #[prost(message, optional, tag = "1")]
    pub legacy_abci_responses: ::core::option::Option<LegacyAbciResponses>,
    #[prost(int64, tag = "2")]
    pub height: i64,
    #[prost(message, optional, tag = "3")]
    pub response_finalize_block: ::core::option::Option<super::abci::ResponseFinalizeBlock>,
}
impl ::prost::Name for AbciResponsesInfo {
    const NAME: &'static str = "ABCIResponsesInfo";
    const PACKAGE: &'static str = "tendermint.state";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.state.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Version {
    #[prost(message, optional, tag = "1")]
    pub consensus: ::core::option::Option<super::version::Consensus>,
    #[prost(string, tag = "2")]
    pub software: ::prost::alloc::string::String,
}
impl ::prost::Name for Version {
    const NAME: &'static str = "Version";
    const PACKAGE: &'static str = "tendermint.state";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.state.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct State {
    #[prost(message, optional, tag = "1")]
    pub version: ::core::option::Option<Version>,
    /// immutable
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "14")]
    pub initial_height: i64,
    /// LastBlockHeight=0 at genesis (ie. block(H=0) does not exist)
    #[prost(int64, tag = "3")]
    pub last_block_height: i64,
    #[prost(message, optional, tag = "4")]
    pub last_block_id: ::core::option::Option<super::types::BlockId>,
    #[prost(message, optional, tag = "5")]
    pub last_block_time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// LastValidators is used to validate block.LastCommit.
    /// Validators are persisted to the database separately every time they change,
    /// so we can query for historical validator sets.
    /// Note that if s.LastBlockHeight causes a valset change,
    /// we set s.LastHeightValidatorsChanged = s.LastBlockHeight + 1 + 1
    /// Extra +1 due to nextValSet delay.
    #[prost(message, optional, tag = "6")]
    pub next_validators: ::core::option::Option<super::types::ValidatorSet>,
    #[prost(message, optional, tag = "7")]
    pub validators: ::core::option::Option<super::types::ValidatorSet>,
    #[prost(message, optional, tag = "8")]
    pub last_validators: ::core::option::Option<super::types::ValidatorSet>,
    #[prost(int64, tag = "9")]
    pub last_height_validators_changed: i64,
    /// Consensus parameters used for validating blocks.
    /// Changes returned by EndBlock and updated after Commit.
    #[prost(message, optional, tag = "10")]
    pub consensus_params: ::core::option::Option<super::types::ConsensusParams>,
    #[prost(int64, tag = "11")]
    pub last_height_consensus_params_changed: i64,
    /// Merkle root of the results from executing prev block
    #[prost(bytes = "vec", tag = "12")]
    pub last_results_hash: ::prost::alloc::vec::Vec<u8>,
    /// the latest AppHash we've received from calling abci.Commit()
    #[prost(bytes = "vec", tag = "13")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for State {
    const NAME: &'static str = "State";
    const PACKAGE: &'static str = "tendermint.state";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("tendermint.state.{}", Self::NAME)
    }
}
// @@protoc_insertion_point(module)
