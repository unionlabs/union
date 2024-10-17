// @generated
/// Request represents a request to the ABCI application.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    /// Sum of all possible messages.
    #[prost(
        oneof = "request::Value",
        tags = "1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17"
    )]
    pub value: ::core::option::Option<request::Value>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    /// Sum of all possible messages.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Echo(super::super::v1beta1::RequestEcho),
        #[prost(message, tag = "2")]
        Flush(super::super::v1beta1::RequestFlush),
        #[prost(message, tag = "3")]
        Info(super::RequestInfo),
        #[prost(message, tag = "5")]
        InitChain(super::RequestInitChain),
        #[prost(message, tag = "6")]
        Query(super::super::v1beta1::RequestQuery),
        #[prost(message, tag = "7")]
        BeginBlock(super::RequestBeginBlock),
        #[prost(message, tag = "8")]
        CheckTx(super::super::v1beta1::RequestCheckTx),
        #[prost(message, tag = "9")]
        DeliverTx(super::super::v1beta1::RequestDeliverTx),
        #[prost(message, tag = "10")]
        EndBlock(super::super::v1beta1::RequestEndBlock),
        #[prost(message, tag = "11")]
        Commit(super::super::v1beta1::RequestCommit),
        #[prost(message, tag = "12")]
        ListSnapshots(super::super::v1beta1::RequestListSnapshots),
        #[prost(message, tag = "13")]
        OfferSnapshot(super::super::v1beta1::RequestOfferSnapshot),
        #[prost(message, tag = "14")]
        LoadSnapshotChunk(super::super::v1beta1::RequestLoadSnapshotChunk),
        #[prost(message, tag = "15")]
        ApplySnapshotChunk(super::super::v1beta1::RequestApplySnapshotChunk),
        #[prost(message, tag = "16")]
        PrepareProposal(super::RequestPrepareProposal),
        #[prost(message, tag = "17")]
        ProcessProposal(super::RequestProcessProposal),
    }
}
impl ::prost::Name for Request {
    const NAME: &'static str = "Request";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// RequestInfo is a request for the ABCI application version.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInfo {
    #[prost(string, tag = "1")]
    pub version: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub block_version: u64,
    #[prost(uint64, tag = "3")]
    pub p2p_version: u64,
    #[prost(string, tag = "4")]
    pub abci_version: ::prost::alloc::string::String,
}
impl ::prost::Name for RequestInfo {
    const NAME: &'static str = "RequestInfo";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// RequestInitChain is a request to initialize the blockchain.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestInitChain {
    #[prost(message, optional, tag = "1")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(string, tag = "2")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub consensus_params: ::core::option::Option<super::super::types::v1beta2::ConsensusParams>,
    #[prost(message, repeated, tag = "4")]
    pub validators: ::prost::alloc::vec::Vec<super::v1beta1::ValidatorUpdate>,
    #[prost(bytes = "vec", tag = "5")]
    pub app_state_bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "6")]
    pub initial_height: i64,
}
impl ::prost::Name for RequestInitChain {
    const NAME: &'static str = "RequestInitChain";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// RequestBeginBlock indicates the beginning of committing the block.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestBeginBlock {
    #[prost(bytes = "vec", tag = "1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub header: ::core::option::Option<super::super::types::v1beta1::Header>,
    #[prost(message, optional, tag = "3")]
    pub last_commit_info: ::core::option::Option<CommitInfo>,
    #[prost(message, repeated, tag = "4")]
    pub byzantine_validators: ::prost::alloc::vec::Vec<Misbehavior>,
}
impl ::prost::Name for RequestBeginBlock {
    const NAME: &'static str = "RequestBeginBlock";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// RequestPrepareProposal is a request for the ABCI application to prepare a new
/// block proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestPrepareProposal {
    /// the modified transactions cannot exceed this size.
    #[prost(int64, tag = "1")]
    pub max_tx_bytes: i64,
    /// txs is an array of transactions that will be included in a block,
    /// sent to the app for possible modifications.
    #[prost(bytes = "vec", repeated, tag = "2")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "3")]
    pub local_last_commit: ::core::option::Option<ExtendedCommitInfo>,
    #[prost(message, repeated, tag = "4")]
    pub misbehavior: ::prost::alloc::vec::Vec<Misbehavior>,
    #[prost(int64, tag = "5")]
    pub height: i64,
    #[prost(message, optional, tag = "6")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(bytes = "vec", tag = "7")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// address of the public key of the validator proposing the block.
    #[prost(bytes = "vec", tag = "8")]
    pub proposer_address: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for RequestPrepareProposal {
    const NAME: &'static str = "RequestPrepareProposal";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// RequestProcessProposal is a request for the ABCI application to process proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestProcessProposal {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "2")]
    pub proposed_last_commit: ::core::option::Option<CommitInfo>,
    #[prost(message, repeated, tag = "3")]
    pub misbehavior: ::prost::alloc::vec::Vec<Misbehavior>,
    /// hash is the merkle root hash of the fields of the proposed block.
    #[prost(bytes = "vec", tag = "4")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag = "5")]
    pub height: i64,
    #[prost(message, optional, tag = "6")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    #[prost(bytes = "vec", tag = "7")]
    pub next_validators_hash: ::prost::alloc::vec::Vec<u8>,
    /// address of the public key of the original proposer of the block.
    #[prost(bytes = "vec", tag = "8")]
    pub proposer_address: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for RequestProcessProposal {
    const NAME: &'static str = "RequestProcessProposal";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// Response represents a response from the ABCI application.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    /// Sum of all possible messages.
    #[prost(
        oneof = "response::Value",
        tags = "1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18"
    )]
    pub value: ::core::option::Option<response::Value>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    /// Sum of all possible messages.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag = "1")]
        Exception(super::super::v1beta1::ResponseException),
        #[prost(message, tag = "2")]
        Echo(super::super::v1beta1::ResponseEcho),
        #[prost(message, tag = "3")]
        Flush(super::super::v1beta1::ResponseFlush),
        #[prost(message, tag = "4")]
        Info(super::super::v1beta1::ResponseInfo),
        #[prost(message, tag = "6")]
        InitChain(super::ResponseInitChain),
        #[prost(message, tag = "7")]
        Query(super::super::v1beta1::ResponseQuery),
        #[prost(message, tag = "8")]
        BeginBlock(super::ResponseBeginBlock),
        #[prost(message, tag = "9")]
        CheckTx(super::ResponseCheckTx),
        #[prost(message, tag = "10")]
        DeliverTx(super::ResponseDeliverTx),
        #[prost(message, tag = "11")]
        EndBlock(super::ResponseEndBlock),
        #[prost(message, tag = "12")]
        Commit(super::super::v1beta1::ResponseCommit),
        #[prost(message, tag = "13")]
        ListSnapshots(super::super::v1beta1::ResponseListSnapshots),
        #[prost(message, tag = "14")]
        OfferSnapshot(super::super::v1beta1::ResponseOfferSnapshot),
        #[prost(message, tag = "15")]
        LoadSnapshotChunk(super::super::v1beta1::ResponseLoadSnapshotChunk),
        #[prost(message, tag = "16")]
        ApplySnapshotChunk(super::super::v1beta1::ResponseApplySnapshotChunk),
        #[prost(message, tag = "17")]
        PrepareProposal(super::ResponsePrepareProposal),
        #[prost(message, tag = "18")]
        ProcessProposal(super::ResponseProcessProposal),
    }
}
impl ::prost::Name for Response {
    const NAME: &'static str = "Response";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ResponseInitChain contains the ABCI application's hash and updates to the
/// validator set and/or the consensus params, if any.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseInitChain {
    #[prost(message, optional, tag = "1")]
    pub consensus_params: ::core::option::Option<super::super::types::v1beta2::ConsensusParams>,
    #[prost(message, repeated, tag = "2")]
    pub validators: ::prost::alloc::vec::Vec<super::v1beta1::ValidatorUpdate>,
    #[prost(bytes = "vec", tag = "3")]
    pub app_hash: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ResponseInitChain {
    const NAME: &'static str = "ResponseInitChain";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ResponseBeginBlock contains a list of block-level events.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBeginBlock {
    #[prost(message, repeated, tag = "1")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
impl ::prost::Name for ResponseBeginBlock {
    const NAME: &'static str = "ResponseBeginBlock";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ResponseCheckTx shows if the transaction was deemed valid by the ABCI
/// application.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseCheckTx {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "6")]
    pub gas_used: i64,
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    #[prost(string, tag = "8")]
    pub codespace: ::prost::alloc::string::String,
    #[prost(string, tag = "9")]
    pub sender: ::prost::alloc::string::String,
    #[prost(int64, tag = "10")]
    pub priority: i64,
    /// mempool_error is set by CometBFT.
    /// ABCI applications creating a ResponseCheckTX should not set mempool_error.
    #[prost(string, tag = "11")]
    pub mempool_error: ::prost::alloc::string::String,
}
impl ::prost::Name for ResponseCheckTx {
    const NAME: &'static str = "ResponseCheckTx";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ResponseDeliverTx contains a result of committing the given transaction and a
/// list of events.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseDeliverTx {
    #[prost(uint32, tag = "1")]
    pub code: u32,
    #[prost(bytes = "vec", tag = "2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    /// nondeterministic
    #[prost(string, tag = "3")]
    pub log: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(string, tag = "4")]
    pub info: ::prost::alloc::string::String,
    #[prost(int64, tag = "5")]
    pub gas_wanted: i64,
    #[prost(int64, tag = "6")]
    pub gas_used: i64,
    /// nondeterministic
    #[prost(message, repeated, tag = "7")]
    pub events: ::prost::alloc::vec::Vec<Event>,
    #[prost(string, tag = "8")]
    pub codespace: ::prost::alloc::string::String,
}
impl ::prost::Name for ResponseDeliverTx {
    const NAME: &'static str = "ResponseDeliverTx";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ResponseEndBlock contains updates to consensus params and/or validator set changes, if any.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseEndBlock {
    #[prost(message, repeated, tag = "1")]
    pub validator_updates: ::prost::alloc::vec::Vec<super::v1beta1::ValidatorUpdate>,
    #[prost(message, optional, tag = "2")]
    pub consensus_param_updates:
        ::core::option::Option<super::super::types::v1beta2::ConsensusParams>,
    #[prost(message, repeated, tag = "3")]
    pub events: ::prost::alloc::vec::Vec<Event>,
}
impl ::prost::Name for ResponseEndBlock {
    const NAME: &'static str = "ResponseEndBlock";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ResponsePrepareProposal contains the list of transactions that will be included in the proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponsePrepareProposal {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub txs: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for ResponsePrepareProposal {
    const NAME: &'static str = "ResponsePrepareProposal";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ResponseProcessProposal contains the result of processing a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseProcessProposal {
    #[prost(enumeration = "response_process_proposal::ProposalStatus", tag = "1")]
    pub status: i32,
}
/// Nested message and enum types in `ResponseProcessProposal`.
pub mod response_process_proposal {
    /// The status.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum ProposalStatus {
        /// Unknown
        Unknown = 0,
        /// Accepted
        Accept = 1,
        /// Rejected
        Reject = 2,
    }
    impl ProposalStatus {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                ProposalStatus::Unknown => "UNKNOWN",
                ProposalStatus::Accept => "ACCEPT",
                ProposalStatus::Reject => "REJECT",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "UNKNOWN" => Some(Self::Unknown),
                "ACCEPT" => Some(Self::Accept),
                "REJECT" => Some(Self::Reject),
                _ => None,
            }
        }
    }
}
impl ::prost::Name for ResponseProcessProposal {
    const NAME: &'static str = "ResponseProcessProposal";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// CommitInfo contains votes for the particular round.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommitInfo {
    #[prost(int32, tag = "1")]
    pub round: i32,
    #[prost(message, repeated, tag = "2")]
    pub votes: ::prost::alloc::vec::Vec<super::v1beta1::VoteInfo>,
}
impl ::prost::Name for CommitInfo {
    const NAME: &'static str = "CommitInfo";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ExtendedCommitInfo is similar to CommitInfo except that it is only used in
/// the PrepareProposal request such that Tendermint can provide vote extensions
/// to the application.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedCommitInfo {
    /// The round at which the block proposer decided in the previous height.
    #[prost(int32, tag = "1")]
    pub round: i32,
    /// List of validators' addresses in the last validator set with their voting
    /// information, including vote extensions.
    #[prost(message, repeated, tag = "2")]
    pub votes: ::prost::alloc::vec::Vec<ExtendedVoteInfo>,
}
impl ::prost::Name for ExtendedCommitInfo {
    const NAME: &'static str = "ExtendedCommitInfo";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// Event allows application developers to attach additional information to
/// ResponseFinalizeBlock (defined in .v1beta3) and ResponseCheckTx.
/// Up to 0.37, this could also be used in ResponseBeginBlock, ResponseEndBlock,
/// and ResponseDeliverTx.
/// Later, transactions may be queried using these events.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Event {
    #[prost(string, tag = "1")]
    pub r#type: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub attributes: ::prost::alloc::vec::Vec<EventAttribute>,
}
impl ::prost::Name for Event {
    const NAME: &'static str = "Event";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// EventAttribute is a single key-value pair, associated with an event.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventAttribute {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
    /// nondeterministic
    #[prost(bool, tag = "3")]
    pub index: bool,
}
impl ::prost::Name for EventAttribute {
    const NAME: &'static str = "EventAttribute";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// ExtendedVoteInfo extends VoteInfo with the vote extensions (non-deterministic).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExtendedVoteInfo {
    /// The validator that sent the vote.
    #[prost(message, optional, tag = "1")]
    pub validator: ::core::option::Option<super::v1beta1::Validator>,
    /// Indicates whether the validator signed the last block, allowing for rewards based on validator availability.
    #[prost(bool, tag = "2")]
    pub signed_last_block: bool,
    /// Non-deterministic extension provided by the sending validator's application.
    #[prost(bytes = "vec", tag = "3")]
    pub vote_extension: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for ExtendedVoteInfo {
    const NAME: &'static str = "ExtendedVoteInfo";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// Misbehavior is a type of misbehavior committed by a validator.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Misbehavior {
    #[prost(enumeration = "MisbehaviorType", tag = "1")]
    pub r#type: i32,
    /// The offending validator
    #[prost(message, optional, tag = "2")]
    pub validator: ::core::option::Option<super::v1beta1::Validator>,
    /// The height when the offense occurred
    #[prost(int64, tag = "3")]
    pub height: i64,
    /// The corresponding time where the offense occurred
    #[prost(message, optional, tag = "4")]
    pub time: ::core::option::Option<::pbjson_types::Timestamp>,
    /// Total voting power of the validator set in case the ABCI application does
    /// not store historical validators.
    /// <https://github.com/tendermint/tendermint/issues/4581>
    #[prost(int64, tag = "5")]
    pub total_voting_power: i64,
}
impl ::prost::Name for Misbehavior {
    const NAME: &'static str = "Misbehavior";
    const PACKAGE: &'static str = "cometbft.abci.v1beta2";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cometbft.abci.v1beta2.{}", Self::NAME)
    }
}
/// The type of misbehavior committed by a validator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MisbehaviorType {
    /// Unknown
    Unknown = 0,
    /// Duplicate vote
    DuplicateVote = 1,
    /// Light client attack
    LightClientAttack = 2,
}
impl MisbehaviorType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MisbehaviorType::Unknown => "UNKNOWN",
            MisbehaviorType::DuplicateVote => "DUPLICATE_VOTE",
            MisbehaviorType::LightClientAttack => "LIGHT_CLIENT_ATTACK",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "UNKNOWN" => Some(Self::Unknown),
            "DUPLICATE_VOTE" => Some(Self::DuplicateVote),
            "LIGHT_CLIENT_ATTACK" => Some(Self::LightClientAttack),
            _ => None,
        }
    }
}
include!("cometbft.abci.v1beta2.tonic.rs");
// @@protoc_insertion_point(module)
