/// WeightedVoteOption defines a unit of vote for vote split.
///
/// Since: cosmos-sdk 0.43
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WeightedVoteOption {
    /// option defines the valid vote options, it must not contain duplicate vote options.
    #[prost(enumeration = "VoteOption", tag = "1")]
    pub option: i32,
    /// weight is the vote weight associated with the vote option.
    #[prost(string, tag = "2")]
    pub weight: ::prost::alloc::string::String,
}
/// TextProposal defines a standard text proposal whose changes need to be
/// manually updated in case of approval.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TextProposal {
    /// title of the proposal.
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// description associated with the proposal.
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
}
/// Deposit defines an amount deposited by an account address to an active
/// proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Deposit {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// depositor defines the deposit addresses from the proposals.
    #[prost(string, tag = "2")]
    pub depositor: ::prost::alloc::string::String,
    /// amount to be deposited by depositor.
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
/// Proposal defines the core field members of a governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// content is the proposal's content.
    #[prost(message, optional, tag = "2")]
    pub content: ::core::option::Option<super::super::super::google::protobuf::Any>,
    /// status defines the proposal status.
    #[prost(enumeration = "ProposalStatus", tag = "3")]
    pub status: i32,
    /// final_tally_result is the final tally result of the proposal. When
    /// querying a proposal via gRPC, this field is not populated until the
    /// proposal's voting period has ended.
    #[prost(message, optional, tag = "4")]
    pub final_tally_result: ::core::option::Option<TallyResult>,
    /// submit_time is the time of proposal submission.
    #[prost(message, optional, tag = "5")]
    pub submit_time: ::core::option::Option<
        super::super::super::google::protobuf::Timestamp,
    >,
    /// deposit_end_time is the end time for deposition.
    #[prost(message, optional, tag = "6")]
    pub deposit_end_time: ::core::option::Option<
        super::super::super::google::protobuf::Timestamp,
    >,
    /// total_deposit is the total deposit on the proposal.
    #[prost(message, repeated, tag = "7")]
    pub total_deposit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// voting_start_time is the starting time to vote on a proposal.
    #[prost(message, optional, tag = "8")]
    pub voting_start_time: ::core::option::Option<
        super::super::super::google::protobuf::Timestamp,
    >,
    /// voting_end_time is the end time of voting on a proposal.
    #[prost(message, optional, tag = "9")]
    pub voting_end_time: ::core::option::Option<
        super::super::super::google::protobuf::Timestamp,
    >,
}
/// TallyResult defines a standard tally for a governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TallyResult {
    /// yes is the number of yes votes on a proposal.
    #[prost(string, tag = "1")]
    pub yes: ::prost::alloc::string::String,
    /// abstain is the number of abstain votes on a proposal.
    #[prost(string, tag = "2")]
    pub abstain: ::prost::alloc::string::String,
    /// no is the number of no votes on a proposal.
    #[prost(string, tag = "3")]
    pub no: ::prost::alloc::string::String,
    /// no_with_veto is the number of no with veto votes on a proposal.
    #[prost(string, tag = "4")]
    pub no_with_veto: ::prost::alloc::string::String,
}
/// Vote defines a vote on a governance proposal.
/// A Vote consists of a proposal ID, the voter, and the vote option.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vote {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// voter is the voter address of the proposal.
    #[prost(string, tag = "2")]
    pub voter: ::prost::alloc::string::String,
    /// Deprecated: Prefer to use `options` instead. This field is set in queries
    /// if and only if `len(options) == 1` and that option has weight 1. In all
    /// other cases, this field will default to VOTE_OPTION_UNSPECIFIED.
    #[deprecated]
    #[prost(enumeration = "VoteOption", tag = "3")]
    pub option: i32,
    /// options is the weighted vote options.
    ///
    /// Since: cosmos-sdk 0.43
    #[prost(message, repeated, tag = "4")]
    pub options: ::prost::alloc::vec::Vec<WeightedVoteOption>,
}
/// DepositParams defines the params for deposits on governance proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DepositParams {
    /// Minimum deposit for a proposal to enter voting period.
    #[prost(message, repeated, tag = "1")]
    pub min_deposit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// Maximum period for Atom holders to deposit on a proposal. Initial value: 2
    /// months.
    #[prost(message, optional, tag = "2")]
    pub max_deposit_period: ::core::option::Option<
        super::super::super::google::protobuf::Duration,
    >,
}
/// VotingParams defines the params for voting on governance proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VotingParams {
    /// Duration of the voting period.
    #[prost(message, optional, tag = "1")]
    pub voting_period: ::core::option::Option<
        super::super::super::google::protobuf::Duration,
    >,
}
/// TallyParams defines the params for tallying votes on governance proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TallyParams {
    /// Minimum percentage of total stake needed to vote for a result to be
    /// considered valid.
    #[prost(bytes = "vec", tag = "1")]
    pub quorum: ::prost::alloc::vec::Vec<u8>,
    /// Minimum proportion of Yes votes for proposal to pass. Default value: 0.5.
    #[prost(bytes = "vec", tag = "2")]
    pub threshold: ::prost::alloc::vec::Vec<u8>,
    /// Minimum value of Veto votes to Total votes ratio for proposal to be
    /// vetoed. Default value: 1/3.
    #[prost(bytes = "vec", tag = "3")]
    pub veto_threshold: ::prost::alloc::vec::Vec<u8>,
}
/// VoteOption enumerates the valid vote options for a given governance proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum VoteOption {
    /// VOTE_OPTION_UNSPECIFIED defines a no-op vote option.
    Unspecified = 0,
    /// VOTE_OPTION_YES defines a yes vote option.
    Yes = 1,
    /// VOTE_OPTION_ABSTAIN defines an abstain vote option.
    Abstain = 2,
    /// VOTE_OPTION_NO defines a no vote option.
    No = 3,
    /// VOTE_OPTION_NO_WITH_VETO defines a no with veto vote option.
    NoWithVeto = 4,
}
impl VoteOption {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            VoteOption::Unspecified => "VOTE_OPTION_UNSPECIFIED",
            VoteOption::Yes => "VOTE_OPTION_YES",
            VoteOption::Abstain => "VOTE_OPTION_ABSTAIN",
            VoteOption::No => "VOTE_OPTION_NO",
            VoteOption::NoWithVeto => "VOTE_OPTION_NO_WITH_VETO",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "VOTE_OPTION_UNSPECIFIED" => Some(Self::Unspecified),
            "VOTE_OPTION_YES" => Some(Self::Yes),
            "VOTE_OPTION_ABSTAIN" => Some(Self::Abstain),
            "VOTE_OPTION_NO" => Some(Self::No),
            "VOTE_OPTION_NO_WITH_VETO" => Some(Self::NoWithVeto),
            _ => None,
        }
    }
}
/// ProposalStatus enumerates the valid statuses of a proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ProposalStatus {
    /// PROPOSAL_STATUS_UNSPECIFIED defines the default proposal status.
    Unspecified = 0,
    /// PROPOSAL_STATUS_DEPOSIT_PERIOD defines a proposal status during the deposit
    /// period.
    DepositPeriod = 1,
    /// PROPOSAL_STATUS_VOTING_PERIOD defines a proposal status during the voting
    /// period.
    VotingPeriod = 2,
    /// PROPOSAL_STATUS_PASSED defines a proposal status of a proposal that has
    /// passed.
    Passed = 3,
    /// PROPOSAL_STATUS_REJECTED defines a proposal status of a proposal that has
    /// been rejected.
    Rejected = 4,
    /// PROPOSAL_STATUS_FAILED defines a proposal status of a proposal that has
    /// failed.
    Failed = 5,
}
impl ProposalStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ProposalStatus::Unspecified => "PROPOSAL_STATUS_UNSPECIFIED",
            ProposalStatus::DepositPeriod => "PROPOSAL_STATUS_DEPOSIT_PERIOD",
            ProposalStatus::VotingPeriod => "PROPOSAL_STATUS_VOTING_PERIOD",
            ProposalStatus::Passed => "PROPOSAL_STATUS_PASSED",
            ProposalStatus::Rejected => "PROPOSAL_STATUS_REJECTED",
            ProposalStatus::Failed => "PROPOSAL_STATUS_FAILED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PROPOSAL_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "PROPOSAL_STATUS_DEPOSIT_PERIOD" => Some(Self::DepositPeriod),
            "PROPOSAL_STATUS_VOTING_PERIOD" => Some(Self::VotingPeriod),
            "PROPOSAL_STATUS_PASSED" => Some(Self::Passed),
            "PROPOSAL_STATUS_REJECTED" => Some(Self::Rejected),
            "PROPOSAL_STATUS_FAILED" => Some(Self::Failed),
            _ => None,
        }
    }
}
/// QueryProposalRequest is the request type for the Query/Proposal RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryProposalRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
}
/// QueryProposalResponse is the response type for the Query/Proposal RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryProposalResponse {
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<Proposal>,
}
/// QueryProposalsRequest is the request type for the Query/Proposals RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryProposalsRequest {
    /// proposal_status defines the status of the proposals.
    #[prost(enumeration = "ProposalStatus", tag = "1")]
    pub proposal_status: i32,
    /// voter defines the voter address for the proposals.
    #[prost(string, tag = "2")]
    pub voter: ::prost::alloc::string::String,
    /// depositor defines the deposit addresses from the proposals.
    #[prost(string, tag = "3")]
    pub depositor: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "4")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageRequest,
    >,
}
/// QueryProposalsResponse is the response type for the Query/Proposals RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryProposalsResponse {
    /// proposals defines all the requested governance proposals.
    #[prost(message, repeated, tag = "1")]
    pub proposals: ::prost::alloc::vec::Vec<Proposal>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageResponse,
    >,
}
/// QueryVoteRequest is the request type for the Query/Vote RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVoteRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// voter defines the voter address for the proposals.
    #[prost(string, tag = "2")]
    pub voter: ::prost::alloc::string::String,
}
/// QueryVoteResponse is the response type for the Query/Vote RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVoteResponse {
    /// vote defines the queried vote.
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<Vote>,
}
/// QueryVotesRequest is the request type for the Query/Votes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVotesRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageRequest,
    >,
}
/// QueryVotesResponse is the response type for the Query/Votes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVotesResponse {
    /// votes defines the queried votes.
    #[prost(message, repeated, tag = "1")]
    pub votes: ::prost::alloc::vec::Vec<Vote>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageResponse,
    >,
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsRequest {
    /// params_type defines which parameters to query for, can be one of "voting",
    /// "tallying" or "deposit".
    #[prost(string, tag = "1")]
    pub params_type: ::prost::alloc::string::String,
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// voting_params defines the parameters related to voting.
    #[prost(message, optional, tag = "1")]
    pub voting_params: ::core::option::Option<VotingParams>,
    /// deposit_params defines the parameters related to deposit.
    #[prost(message, optional, tag = "2")]
    pub deposit_params: ::core::option::Option<DepositParams>,
    /// tally_params defines the parameters related to tally.
    #[prost(message, optional, tag = "3")]
    pub tally_params: ::core::option::Option<TallyParams>,
}
/// QueryDepositRequest is the request type for the Query/Deposit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDepositRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// depositor defines the deposit addresses from the proposals.
    #[prost(string, tag = "2")]
    pub depositor: ::prost::alloc::string::String,
}
/// QueryDepositResponse is the response type for the Query/Deposit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDepositResponse {
    /// deposit defines the requested deposit.
    #[prost(message, optional, tag = "1")]
    pub deposit: ::core::option::Option<Deposit>,
}
/// QueryDepositsRequest is the request type for the Query/Deposits RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDepositsRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageRequest,
    >,
}
/// QueryDepositsResponse is the response type for the Query/Deposits RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDepositsResponse {
    /// deposits defines the requested deposits.
    #[prost(message, repeated, tag = "1")]
    pub deposits: ::prost::alloc::vec::Vec<Deposit>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::base::query::v1beta1::PageResponse,
    >,
}
/// QueryTallyResultRequest is the request type for the Query/Tally RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTallyResultRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
}
/// QueryTallyResultResponse is the response type for the Query/Tally RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTallyResultResponse {
    /// tally defines the requested tally.
    #[prost(message, optional, tag = "1")]
    pub tally: ::core::option::Option<TallyResult>,
}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query defines the gRPC querier service for gov module
    #[derive(Debug, Clone)]
    pub struct QueryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            QueryClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Proposal queries proposal details based on ProposalID.
        pub async fn proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryProposalRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryProposalResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Query/Proposal",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Query", "Proposal"));
            self.inner.unary(req, path, codec).await
        }
        /// Proposals queries all proposals based on given status.
        pub async fn proposals(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryProposalsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryProposalsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Query/Proposals",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Query", "Proposals"));
            self.inner.unary(req, path, codec).await
        }
        /// Vote queries voted information based on proposalID, voterAddr.
        pub async fn vote(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryVoteRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryVoteResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Query/Vote",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Query", "Vote"));
            self.inner.unary(req, path, codec).await
        }
        /// Votes queries votes of a given proposal.
        pub async fn votes(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryVotesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryVotesResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Query/Votes",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Query", "Votes"));
            self.inner.unary(req, path, codec).await
        }
        /// Params queries all parameters of the gov module.
        pub async fn params(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryParamsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryParamsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Query/Params",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
        /// Deposit queries single deposit information based proposalID, depositAddr.
        pub async fn deposit(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDepositRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDepositResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Query/Deposit",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Query", "Deposit"));
            self.inner.unary(req, path, codec).await
        }
        /// Deposits queries all deposits of a single proposal.
        pub async fn deposits(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryDepositsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDepositsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Query/Deposits",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Query", "Deposits"));
            self.inner.unary(req, path, codec).await
        }
        /// TallyResult queries the tally of a proposal vote.
        pub async fn tally_result(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryTallyResultRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTallyResultResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Query/TallyResult",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Query", "TallyResult"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod query_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServer.
    #[async_trait]
    pub trait Query: Send + Sync + 'static {
        /// Proposal queries proposal details based on ProposalID.
        async fn proposal(
            &self,
            request: tonic::Request<super::QueryProposalRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryProposalResponse>,
            tonic::Status,
        >;
        /// Proposals queries all proposals based on given status.
        async fn proposals(
            &self,
            request: tonic::Request<super::QueryProposalsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryProposalsResponse>,
            tonic::Status,
        >;
        /// Vote queries voted information based on proposalID, voterAddr.
        async fn vote(
            &self,
            request: tonic::Request<super::QueryVoteRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryVoteResponse>,
            tonic::Status,
        >;
        /// Votes queries votes of a given proposal.
        async fn votes(
            &self,
            request: tonic::Request<super::QueryVotesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryVotesResponse>,
            tonic::Status,
        >;
        /// Params queries all parameters of the gov module.
        async fn params(
            &self,
            request: tonic::Request<super::QueryParamsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryParamsResponse>,
            tonic::Status,
        >;
        /// Deposit queries single deposit information based proposalID, depositAddr.
        async fn deposit(
            &self,
            request: tonic::Request<super::QueryDepositRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDepositResponse>,
            tonic::Status,
        >;
        /// Deposits queries all deposits of a single proposal.
        async fn deposits(
            &self,
            request: tonic::Request<super::QueryDepositsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryDepositsResponse>,
            tonic::Status,
        >;
        /// TallyResult queries the tally of a proposal vote.
        async fn tally_result(
            &self,
            request: tonic::Request<super::QueryTallyResultRequest>,
        ) -> std::result::Result<
            tonic::Response<super::QueryTallyResultResponse>,
            tonic::Status,
        >;
    }
    /// Query defines the gRPC querier service for gov module
    #[derive(Debug)]
    pub struct QueryServer<T: Query> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Query> QueryServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServer<T>
    where
        T: Query,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/cosmos.gov.v1beta1.Query/Proposal" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryProposalRequest>
                    for ProposalSvc<T> {
                        type Response = super::QueryProposalResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryProposalRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).proposal(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Query/Proposals" => {
                    #[allow(non_camel_case_types)]
                    struct ProposalsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryProposalsRequest>
                    for ProposalsSvc<T> {
                        type Response = super::QueryProposalsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryProposalsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).proposals(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProposalsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Query/Vote" => {
                    #[allow(non_camel_case_types)]
                    struct VoteSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryVoteRequest>
                    for VoteSvc<T> {
                        type Response = super::QueryVoteResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryVoteRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).vote(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VoteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Query/Votes" => {
                    #[allow(non_camel_case_types)]
                    struct VotesSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryVotesRequest>
                    for VotesSvc<T> {
                        type Response = super::QueryVotesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryVotesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).votes(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VotesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Query/Params" => {
                    #[allow(non_camel_case_types)]
                    struct ParamsSvc<T: Query>(pub Arc<T>);
                    impl<T: Query> tonic::server::UnaryService<super::QueryParamsRequest>
                    for ParamsSvc<T> {
                        type Response = super::QueryParamsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryParamsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).params(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ParamsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Query/Deposit" => {
                    #[allow(non_camel_case_types)]
                    struct DepositSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryDepositRequest>
                    for DepositSvc<T> {
                        type Response = super::QueryDepositResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryDepositRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).deposit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DepositSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Query/Deposits" => {
                    #[allow(non_camel_case_types)]
                    struct DepositsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryDepositsRequest>
                    for DepositsSvc<T> {
                        type Response = super::QueryDepositsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryDepositsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).deposits(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DepositsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Query/TallyResult" => {
                    #[allow(non_camel_case_types)]
                    struct TallyResultSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<super::QueryTallyResultRequest>
                    for TallyResultSvc<T> {
                        type Response = super::QueryTallyResultResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::QueryTallyResultRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).tally_result(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TallyResultSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Query> Clone for QueryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Query> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Query> tonic::server::NamedService for QueryServer<T> {
        const NAME: &'static str = "cosmos.gov.v1beta1.Query";
    }
}
/// MsgSubmitProposal defines an sdk.Msg type that supports submitting arbitrary
/// proposal Content.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitProposal {
    /// content is the proposal's content.
    #[prost(message, optional, tag = "1")]
    pub content: ::core::option::Option<super::super::super::google::protobuf::Any>,
    /// initial_deposit is the deposit value that must be paid at proposal submission.
    #[prost(message, repeated, tag = "2")]
    pub initial_deposit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// proposer is the account address of the proposer.
    #[prost(string, tag = "3")]
    pub proposer: ::prost::alloc::string::String,
}
/// MsgSubmitProposalResponse defines the Msg/SubmitProposal response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitProposalResponse {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
}
/// MsgVote defines a message to cast a vote.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgVote {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// voter is the voter address for the proposal.
    #[prost(string, tag = "2")]
    pub voter: ::prost::alloc::string::String,
    /// option defines the vote option.
    #[prost(enumeration = "VoteOption", tag = "3")]
    pub option: i32,
}
/// MsgVoteResponse defines the Msg/Vote response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgVoteResponse {}
/// MsgVoteWeighted defines a message to cast a vote.
///
/// Since: cosmos-sdk 0.43
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgVoteWeighted {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// voter is the voter address for the proposal.
    #[prost(string, tag = "2")]
    pub voter: ::prost::alloc::string::String,
    /// options defines the weighted vote options.
    #[prost(message, repeated, tag = "3")]
    pub options: ::prost::alloc::vec::Vec<WeightedVoteOption>,
}
/// MsgVoteWeightedResponse defines the Msg/VoteWeighted response type.
///
/// Since: cosmos-sdk 0.43
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgVoteWeightedResponse {}
/// MsgDeposit defines a message to submit a deposit to an existing proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDeposit {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// depositor defines the deposit addresses from the proposals.
    #[prost(string, tag = "2")]
    pub depositor: ::prost::alloc::string::String,
    /// amount to be deposited by depositor.
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
}
/// MsgDepositResponse defines the Msg/Deposit response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDepositResponse {}
/// Generated client implementations.
#[cfg(feature = "client")]
pub mod msg_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Msg defines the bank Msg service.
    #[derive(Debug, Clone)]
    pub struct MsgClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MsgClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> MsgClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MsgClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            MsgClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// SubmitProposal defines a method to create new proposal given a content.
        pub async fn submit_proposal(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgSubmitProposal>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSubmitProposalResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Msg/SubmitProposal",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Msg", "SubmitProposal"));
            self.inner.unary(req, path, codec).await
        }
        /// Vote defines a method to add a vote on a specific proposal.
        pub async fn vote(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgVote>,
        ) -> std::result::Result<
            tonic::Response<super::MsgVoteResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Msg/Vote",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Msg", "Vote"));
            self.inner.unary(req, path, codec).await
        }
        /// VoteWeighted defines a method to add a weighted vote on a specific proposal.
        ///
        /// Since: cosmos-sdk 0.43
        pub async fn vote_weighted(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgVoteWeighted>,
        ) -> std::result::Result<
            tonic::Response<super::MsgVoteWeightedResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Msg/VoteWeighted",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Msg", "VoteWeighted"));
            self.inner.unary(req, path, codec).await
        }
        /// Deposit defines a method to add deposit on a specific proposal.
        pub async fn deposit(
            &mut self,
            request: impl tonic::IntoRequest<super::MsgDeposit>,
        ) -> std::result::Result<
            tonic::Response<super::MsgDepositResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/cosmos.gov.v1beta1.Msg/Deposit",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("cosmos.gov.v1beta1.Msg", "Deposit"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "server")]
pub mod msg_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MsgServer.
    #[async_trait]
    pub trait Msg: Send + Sync + 'static {
        /// SubmitProposal defines a method to create new proposal given a content.
        async fn submit_proposal(
            &self,
            request: tonic::Request<super::MsgSubmitProposal>,
        ) -> std::result::Result<
            tonic::Response<super::MsgSubmitProposalResponse>,
            tonic::Status,
        >;
        /// Vote defines a method to add a vote on a specific proposal.
        async fn vote(
            &self,
            request: tonic::Request<super::MsgVote>,
        ) -> std::result::Result<tonic::Response<super::MsgVoteResponse>, tonic::Status>;
        /// VoteWeighted defines a method to add a weighted vote on a specific proposal.
        ///
        /// Since: cosmos-sdk 0.43
        async fn vote_weighted(
            &self,
            request: tonic::Request<super::MsgVoteWeighted>,
        ) -> std::result::Result<
            tonic::Response<super::MsgVoteWeightedResponse>,
            tonic::Status,
        >;
        /// Deposit defines a method to add deposit on a specific proposal.
        async fn deposit(
            &self,
            request: tonic::Request<super::MsgDeposit>,
        ) -> std::result::Result<
            tonic::Response<super::MsgDepositResponse>,
            tonic::Status,
        >;
    }
    /// Msg defines the bank Msg service.
    #[derive(Debug)]
    pub struct MsgServer<T: Msg> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Msg> MsgServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for MsgServer<T>
    where
        T: Msg,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/cosmos.gov.v1beta1.Msg/SubmitProposal" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitProposalSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgSubmitProposal>
                    for SubmitProposalSvc<T> {
                        type Response = super::MsgSubmitProposalResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgSubmitProposal>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).submit_proposal(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubmitProposalSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Msg/Vote" => {
                    #[allow(non_camel_case_types)]
                    struct VoteSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgVote>
                    for VoteSvc<T> {
                        type Response = super::MsgVoteResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgVote>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).vote(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VoteSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Msg/VoteWeighted" => {
                    #[allow(non_camel_case_types)]
                    struct VoteWeightedSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgVoteWeighted>
                    for VoteWeightedSvc<T> {
                        type Response = super::MsgVoteWeightedResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgVoteWeighted>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                (*inner).vote_weighted(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = VoteWeightedSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/cosmos.gov.v1beta1.Msg/Deposit" => {
                    #[allow(non_camel_case_types)]
                    struct DepositSvc<T: Msg>(pub Arc<T>);
                    impl<T: Msg> tonic::server::UnaryService<super::MsgDeposit>
                    for DepositSvc<T> {
                        type Response = super::MsgDepositResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MsgDeposit>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { (*inner).deposit(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DepositSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Msg> Clone for MsgServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Msg> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Msg> tonic::server::NamedService for MsgServer<T> {
        const NAME: &'static str = "cosmos.gov.v1beta1.Msg";
    }
}
/// GenesisState defines the gov module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// starting_proposal_id is the ID of the starting proposal.
    #[prost(uint64, tag = "1")]
    pub starting_proposal_id: u64,
    /// deposits defines all the deposits present at genesis.
    #[prost(message, repeated, tag = "2")]
    pub deposits: ::prost::alloc::vec::Vec<Deposit>,
    /// votes defines all the votes present at genesis.
    #[prost(message, repeated, tag = "3")]
    pub votes: ::prost::alloc::vec::Vec<Vote>,
    /// proposals defines all the proposals present at genesis.
    #[prost(message, repeated, tag = "4")]
    pub proposals: ::prost::alloc::vec::Vec<Proposal>,
    /// params defines all the parameters of related to deposit.
    #[prost(message, optional, tag = "5")]
    pub deposit_params: ::core::option::Option<DepositParams>,
    /// params defines all the parameters of related to voting.
    #[prost(message, optional, tag = "6")]
    pub voting_params: ::core::option::Option<VotingParams>,
    /// params defines all the parameters of related to tally.
    #[prost(message, optional, tag = "7")]
    pub tally_params: ::core::option::Option<TallyParams>,
}
