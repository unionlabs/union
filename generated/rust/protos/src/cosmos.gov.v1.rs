// @generated
/// WeightedVoteOption defines a unit of vote for vote split.
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
impl ::prost::Name for WeightedVoteOption {
    const NAME: &'static str = "WeightedVoteOption";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
impl ::prost::Name for Deposit {
    const NAME: &'static str = "Deposit";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// Proposal defines the core field members of a governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Proposal {
    /// id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub id: u64,
    /// messages are the arbitrary messages to be executed if the proposal passes.
    #[prost(message, repeated, tag = "2")]
    pub messages: ::prost::alloc::vec::Vec<super::super::super::google::protobuf::Any>,
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
    pub submit_time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// deposit_end_time is the end time for deposition.
    #[prost(message, optional, tag = "6")]
    pub deposit_end_time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// total_deposit is the total deposit on the proposal.
    #[prost(message, repeated, tag = "7")]
    pub total_deposit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// voting_start_time is the starting time to vote on a proposal.
    #[prost(message, optional, tag = "8")]
    pub voting_start_time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// voting_end_time is the end time of voting on a proposal.
    #[prost(message, optional, tag = "9")]
    pub voting_end_time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// metadata is any arbitrary metadata attached to the proposal.
    /// the recommended format of the metadata is to be found here:
    /// <https://docs.cosmos.network/v0.47/modules/gov#proposal-3>
    #[prost(string, tag = "10")]
    pub metadata: ::prost::alloc::string::String,
    /// title is the title of the proposal
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(string, tag = "11")]
    pub title: ::prost::alloc::string::String,
    /// summary is a short summary of the proposal
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(string, tag = "12")]
    pub summary: ::prost::alloc::string::String,
    /// proposer is the address of the proposal sumbitter
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(string, tag = "13")]
    pub proposer: ::prost::alloc::string::String,
    /// expedited defines if the proposal is expedited
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(bool, tag = "14")]
    pub expedited: bool,
    /// failed_reason defines the reason why the proposal failed
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(string, tag = "15")]
    pub failed_reason: ::prost::alloc::string::String,
}
impl ::prost::Name for Proposal {
    const NAME: &'static str = "Proposal";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// TallyResult defines a standard tally for a governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TallyResult {
    /// yes_count is the number of yes votes on a proposal.
    #[prost(string, tag = "1")]
    pub yes_count: ::prost::alloc::string::String,
    /// abstain_count is the number of abstain votes on a proposal.
    #[prost(string, tag = "2")]
    pub abstain_count: ::prost::alloc::string::String,
    /// no_count is the number of no votes on a proposal.
    #[prost(string, tag = "3")]
    pub no_count: ::prost::alloc::string::String,
    /// no_with_veto_count is the number of no with veto votes on a proposal.
    #[prost(string, tag = "4")]
    pub no_with_veto_count: ::prost::alloc::string::String,
}
impl ::prost::Name for TallyResult {
    const NAME: &'static str = "TallyResult";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    /// options is the weighted vote options.
    #[prost(message, repeated, tag = "4")]
    pub options: ::prost::alloc::vec::Vec<WeightedVoteOption>,
    /// metadata is any arbitrary metadata attached to the vote.
    /// the recommended format of the metadata is to be found here: <https://docs.cosmos.network/v0.47/modules/gov#vote-5>
    #[prost(string, tag = "5")]
    pub metadata: ::prost::alloc::string::String,
}
impl ::prost::Name for Vote {
    const NAME: &'static str = "Vote";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    pub max_deposit_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
}
impl ::prost::Name for DepositParams {
    const NAME: &'static str = "DepositParams";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// VotingParams defines the params for voting on governance proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VotingParams {
    /// Duration of the voting period.
    #[prost(message, optional, tag = "1")]
    pub voting_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
}
impl ::prost::Name for VotingParams {
    const NAME: &'static str = "VotingParams";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// TallyParams defines the params for tallying votes on governance proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TallyParams {
    /// Minimum percentage of total stake needed to vote for a result to be
    /// considered valid.
    #[prost(string, tag = "1")]
    pub quorum: ::prost::alloc::string::String,
    /// Minimum proportion of Yes votes for proposal to pass. Default value: 0.5.
    #[prost(string, tag = "2")]
    pub threshold: ::prost::alloc::string::String,
    /// Minimum value of Veto votes to Total votes ratio for proposal to be
    /// vetoed. Default value: 1/3.
    #[prost(string, tag = "3")]
    pub veto_threshold: ::prost::alloc::string::String,
}
impl ::prost::Name for TallyParams {
    const NAME: &'static str = "TallyParams";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// Params defines the parameters for the x/gov module.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// Minimum deposit for a proposal to enter voting period.
    #[prost(message, repeated, tag = "1")]
    pub min_deposit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// Maximum period for Atom holders to deposit on a proposal. Initial value: 2
    /// months.
    #[prost(message, optional, tag = "2")]
    pub max_deposit_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// Duration of the voting period.
    #[prost(message, optional, tag = "3")]
    pub voting_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    ///   Minimum percentage of total stake needed to vote for a result to be
    ///   considered valid.
    #[prost(string, tag = "4")]
    pub quorum: ::prost::alloc::string::String,
    ///   Minimum proportion of Yes votes for proposal to pass. Default value: 0.5.
    #[prost(string, tag = "5")]
    pub threshold: ::prost::alloc::string::String,
    ///   Minimum value of Veto votes to Total votes ratio for proposal to be
    ///   vetoed. Default value: 1/3.
    #[prost(string, tag = "6")]
    pub veto_threshold: ::prost::alloc::string::String,
    ///   The ratio representing the proportion of the deposit value that must be paid at proposal submission.
    #[prost(string, tag = "7")]
    pub min_initial_deposit_ratio: ::prost::alloc::string::String,
    /// The cancel ratio which will not be returned back to the depositors when a proposal is cancelled.
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(string, tag = "8")]
    pub proposal_cancel_ratio: ::prost::alloc::string::String,
    /// The address which will receive (proposal_cancel_ratio * deposit) proposal deposits.
    /// If empty, the (proposal_cancel_ratio * deposit) proposal deposits will be burned.
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(string, tag = "9")]
    pub proposal_cancel_dest: ::prost::alloc::string::String,
    /// Duration of the voting period of an expedited proposal.
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(message, optional, tag = "10")]
    pub expedited_voting_period:
        ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// Minimum proportion of Yes votes for proposal to pass. Default value: 0.67.
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(string, tag = "11")]
    pub expedited_threshold: ::prost::alloc::string::String,
    ///   Minimum expedited deposit for a proposal to enter voting period.
    #[prost(message, repeated, tag = "12")]
    pub expedited_min_deposit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// burn deposits if a proposal does not meet quorum
    #[prost(bool, tag = "13")]
    pub burn_vote_quorum: bool,
    /// burn deposits if the proposal does not enter voting period
    #[prost(bool, tag = "14")]
    pub burn_proposal_deposit_prevote: bool,
    /// burn deposits if quorum with vote type no_veto is met
    #[prost(bool, tag = "15")]
    pub burn_vote_veto: bool,
    /// The ratio representing the proportion of the deposit value minimum that must be met when making a deposit.
    /// Default value: 0.01. Meaning that for a chain with a min_deposit of 100stake, a deposit of 1stake would be
    /// required.
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(string, tag = "16")]
    pub min_deposit_ratio: ::prost::alloc::string::String,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    /// Deprecated: Prefer to use `params` instead.
    /// deposit_params defines all the paramaters of related to deposit.
    #[deprecated]
    #[prost(message, optional, tag = "5")]
    pub deposit_params: ::core::option::Option<DepositParams>,
    /// Deprecated: Prefer to use `params` instead.
    /// voting_params defines all the paramaters of related to voting.
    #[deprecated]
    #[prost(message, optional, tag = "6")]
    pub voting_params: ::core::option::Option<VotingParams>,
    /// Deprecated: Prefer to use `params` instead.
    /// tally_params defines all the paramaters of related to tally.
    #[deprecated]
    #[prost(message, optional, tag = "7")]
    pub tally_params: ::core::option::Option<TallyParams>,
    /// params defines all the paramaters of x/gov module.
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(message, optional, tag = "8")]
    pub params: ::core::option::Option<Params>,
    /// The constitution allows builders to lay a foundation and define purpose.
    /// This is an immutable string set in genesis.
    /// There are no amendments, to go outside of scope, just fork.
    /// constitution is an immutable string in genesis for a chain builder to lay out their vision, ideas and ideals.
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(string, tag = "9")]
    pub constitution: ::prost::alloc::string::String,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// QueryConstitutionRequest is the request type for the Query/Constitution RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConstitutionRequest {}
impl ::prost::Name for QueryConstitutionRequest {
    const NAME: &'static str = "QueryConstitutionRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// QueryConstitutionResponse is the response type for the Query/Constitution RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConstitutionResponse {
    #[prost(string, tag = "1")]
    pub constitution: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryConstitutionResponse {
    const NAME: &'static str = "QueryConstitutionResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
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
impl ::prost::Name for QueryProposalRequest {
    const NAME: &'static str = "QueryProposalRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// QueryProposalResponse is the response type for the Query/Proposal RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryProposalResponse {
    /// proposal is the requested governance proposal.
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<Proposal>,
}
impl ::prost::Name for QueryProposalResponse {
    const NAME: &'static str = "QueryProposalResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryProposalsRequest {
    const NAME: &'static str = "QueryProposalsRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryProposalsResponse {
    const NAME: &'static str = "QueryProposalsResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
impl ::prost::Name for QueryVoteRequest {
    const NAME: &'static str = "QueryVoteRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// QueryVoteResponse is the response type for the Query/Vote RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVoteResponse {
    /// vote defines the queried vote.
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<Vote>,
}
impl ::prost::Name for QueryVoteResponse {
    const NAME: &'static str = "QueryVoteResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryVotesRequest {
    const NAME: &'static str = "QueryVotesRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryVotesResponse {
    const NAME: &'static str = "QueryVotesResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryParamsResponse {
    /// Deprecated: Prefer to use `params` instead.
    /// voting_params defines the parameters related to voting.
    #[deprecated]
    #[prost(message, optional, tag = "1")]
    pub voting_params: ::core::option::Option<VotingParams>,
    /// Deprecated: Prefer to use `params` instead.
    /// deposit_params defines the parameters related to deposit.
    #[deprecated]
    #[prost(message, optional, tag = "2")]
    pub deposit_params: ::core::option::Option<DepositParams>,
    /// Deprecated: Prefer to use `params` instead.
    /// tally_params defines the parameters related to tally.
    #[deprecated]
    #[prost(message, optional, tag = "3")]
    pub tally_params: ::core::option::Option<TallyParams>,
    /// params defines all the paramaters of x/gov module.
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
impl ::prost::Name for QueryDepositRequest {
    const NAME: &'static str = "QueryDepositRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// QueryDepositResponse is the response type for the Query/Deposit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryDepositResponse {
    /// deposit defines the requested deposit.
    #[prost(message, optional, tag = "1")]
    pub deposit: ::core::option::Option<Deposit>,
}
impl ::prost::Name for QueryDepositResponse {
    const NAME: &'static str = "QueryDepositResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
impl ::prost::Name for QueryDepositsRequest {
    const NAME: &'static str = "QueryDepositsRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
impl ::prost::Name for QueryDepositsResponse {
    const NAME: &'static str = "QueryDepositsResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// QueryTallyResultRequest is the request type for the Query/Tally RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTallyResultRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
}
impl ::prost::Name for QueryTallyResultRequest {
    const NAME: &'static str = "QueryTallyResultRequest";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// QueryTallyResultResponse is the response type for the Query/Tally RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryTallyResultResponse {
    /// tally defines the requested tally.
    #[prost(message, optional, tag = "1")]
    pub tally: ::core::option::Option<TallyResult>,
}
impl ::prost::Name for QueryTallyResultResponse {
    const NAME: &'static str = "QueryTallyResultResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgSubmitProposal defines an sdk.Msg type that supports submitting arbitrary
/// proposal Content.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitProposal {
    /// messages are the arbitrary messages to be executed if proposal passes.
    #[prost(message, repeated, tag = "1")]
    pub messages: ::prost::alloc::vec::Vec<super::super::super::google::protobuf::Any>,
    /// initial_deposit is the deposit value that must be paid at proposal submission.
    #[prost(message, repeated, tag = "2")]
    pub initial_deposit: ::prost::alloc::vec::Vec<super::super::base::v1beta1::Coin>,
    /// proposer is the account address of the proposer.
    #[prost(string, tag = "3")]
    pub proposer: ::prost::alloc::string::String,
    /// metadata is any arbitrary metadata attached to the proposal.
    #[prost(string, tag = "4")]
    pub metadata: ::prost::alloc::string::String,
    /// title is the title of the proposal.
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(string, tag = "5")]
    pub title: ::prost::alloc::string::String,
    /// summary is the summary of the proposal
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(string, tag = "6")]
    pub summary: ::prost::alloc::string::String,
    /// expedited defines if the proposal is expedited or not
    ///
    /// Since: cosmos-sdk 0.50
    #[prost(bool, tag = "7")]
    pub expedited: bool,
}
impl ::prost::Name for MsgSubmitProposal {
    const NAME: &'static str = "MsgSubmitProposal";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgSubmitProposalResponse defines the Msg/SubmitProposal response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitProposalResponse {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
}
impl ::prost::Name for MsgSubmitProposalResponse {
    const NAME: &'static str = "MsgSubmitProposalResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgExecLegacyContent is used to wrap the legacy content field into a message.
/// This ensures backwards compatibility with v1beta1.MsgSubmitProposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExecLegacyContent {
    /// content is the proposal's content.
    #[prost(message, optional, tag = "1")]
    pub content: ::core::option::Option<super::super::super::google::protobuf::Any>,
    /// authority must be the gov module address.
    #[prost(string, tag = "2")]
    pub authority: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgExecLegacyContent {
    const NAME: &'static str = "MsgExecLegacyContent";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgExecLegacyContentResponse defines the Msg/ExecLegacyContent response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExecLegacyContentResponse {}
impl ::prost::Name for MsgExecLegacyContentResponse {
    const NAME: &'static str = "MsgExecLegacyContentResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
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
    /// metadata is any arbitrary metadata attached to the Vote.
    #[prost(string, tag = "4")]
    pub metadata: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgVote {
    const NAME: &'static str = "MsgVote";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgVoteResponse defines the Msg/Vote response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgVoteResponse {}
impl ::prost::Name for MsgVoteResponse {
    const NAME: &'static str = "MsgVoteResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgVoteWeighted defines a message to cast a vote.
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
    /// metadata is any arbitrary metadata attached to the VoteWeighted.
    #[prost(string, tag = "4")]
    pub metadata: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgVoteWeighted {
    const NAME: &'static str = "MsgVoteWeighted";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgVoteWeightedResponse defines the Msg/VoteWeighted response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgVoteWeightedResponse {}
impl ::prost::Name for MsgVoteWeightedResponse {
    const NAME: &'static str = "MsgVoteWeightedResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
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
impl ::prost::Name for MsgDeposit {
    const NAME: &'static str = "MsgDeposit";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgDepositResponse defines the Msg/Deposit response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDepositResponse {}
impl ::prost::Name for MsgDepositResponse {
    const NAME: &'static str = "MsgDepositResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams is the Msg/UpdateParams request type.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/gov parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgCancelProposal is the Msg/CancelProposal request type.
///
/// Since: cosmos-sdk 0.50
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCancelProposal {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// proposer is the account address of the proposer.
    #[prost(string, tag = "2")]
    pub proposer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCancelProposal {
    const NAME: &'static str = "MsgCancelProposal";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
/// MsgCancelProposalResponse defines the response structure for executing a
/// MsgCancelProposal message.
///
/// Since: cosmos-sdk 0.50
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCancelProposalResponse {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// canceled_time is the time when proposal is canceled.
    #[prost(message, optional, tag = "2")]
    pub canceled_time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// canceled_height defines the block height at which the proposal is canceled.
    #[prost(uint64, tag = "3")]
    pub canceled_height: u64,
}
impl ::prost::Name for MsgCancelProposalResponse {
    const NAME: &'static str = "MsgCancelProposalResponse";
    const PACKAGE: &'static str = "cosmos.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("cosmos.gov.v1.{}", Self::NAME)
    }
}
include!("cosmos.gov.v1.tonic.rs");
// @@protoc_insertion_point(module)
