#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MinDepositThrottler {
    /// Floor value for the minimum deposit required for a proposal to enter the voting period.
    #[prost(message, repeated, tag = "1")]
    pub floor_value: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Duration that dictates after how long the dynamic minimum deposit should be recalculated
    /// for time-based decreases.
    #[prost(message, optional, tag = "2")]
    pub update_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// The number of active proposals the dynamic minimum deposit should target.
    #[prost(uint64, tag = "3")]
    pub target_active_proposals: u64,
    /// The ratio of increase for the minimum deposit when the number of active proposals
    /// is at or above the target.
    #[prost(string, tag = "4")]
    pub increase_ratio: ::prost::alloc::string::String,
    /// The ratio of decrease for the minimum deposit when the number of active proposals
    /// is 1 less than the target.
    #[prost(string, tag = "5")]
    pub decrease_ratio: ::prost::alloc::string::String,
    /// A positive integer representing the sensitivity of dynamic minimum deposit
    /// decreases to the distance from the target number of active proposals.
    /// The higher the number, the lower the sensitivity. A value of 1 represents the
    /// highest sensitivity.
    #[prost(uint64, tag = "6")]
    pub decrease_sensitivity_target_distance: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MinInitialDepositThrottler {
    /// Floor value for the minimum initial deposit required for a proposal to enter the deposit period.
    #[prost(message, repeated, tag = "1")]
    pub floor_value: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Duration that dictates after how long the dynamic minimum deposit should be recalculated
    /// for time-based decreases.
    #[prost(message, optional, tag = "2")]
    pub update_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// The number of proposals in deposit period the dynamic minimum initial deposit should target.
    #[prost(uint64, tag = "3")]
    pub target_proposals: u64,
    /// The ratio of increase for the minimum initial deposit when the number of proposals
    /// in deposit period is at or above the target.
    #[prost(string, tag = "4")]
    pub increase_ratio: ::prost::alloc::string::String,
    /// The ratio of decrease for the minimum initial deposit when the number of proposals
    /// in deposit period is 1 less than the target.
    #[prost(string, tag = "5")]
    pub decrease_ratio: ::prost::alloc::string::String,
    /// A positive integer representing the sensitivity of dynamic minimum initial
    /// deposit decreases to the distance from the target number of proposals
    /// in deposit period. The higher the number, the lower the sensitivity. A value
    /// of 1 represents the highest sensitivity.
    #[prost(uint64, tag = "6")]
    pub decrease_sensitivity_target_distance: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QuorumRange {
    /// Maximum achievable quorum
    #[prost(string, tag = "1")]
    pub max: ::prost::alloc::string::String,
    /// Minimum achievable quorum
    #[prost(string, tag = "2")]
    pub min: ::prost::alloc::string::String,
}
/// Deposit defines an amount deposited by an account address to an active
/// proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Deposit {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// depositor defines the deposit addresses from the proposals.
    #[prost(string, tag = "2")]
    pub depositor: ::prost::alloc::string::String,
    /// amount to be deposited by depositor.
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// DepositParams defines the params for deposits on governance proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct DepositParams {
    /// Minimum deposit for a proposal to enter voting period.
    #[prost(message, repeated, tag = "1")]
    pub min_deposit: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Maximum period for Atom holders to deposit on a proposal. Initial value: 2
    /// months.
    #[prost(message, optional, tag = "2")]
    pub max_deposit_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
}
/// Description defines a governor description.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GovernorDescription {
    /// moniker defines a human-readable name for the governor.
    #[prost(string, tag = "1")]
    pub moniker: ::prost::alloc::string::String,
    /// identity defines an optional identity signature (ex. UPort or Keybase).
    #[prost(string, tag = "2")]
    pub identity: ::prost::alloc::string::String,
    /// website defines an optional website link.
    #[prost(string, tag = "3")]
    pub website: ::prost::alloc::string::String,
    /// security_contact defines an optional email for security contact.
    #[prost(string, tag = "4")]
    pub security_contact: ::prost::alloc::string::String,
    /// details define other optional details.
    #[prost(string, tag = "5")]
    pub details: ::prost::alloc::string::String,
}
/// GenesisState defines the gov module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    ///
    /// Since: cosmos-sdk 0.48
    #[prost(string, tag = "9")]
    pub constitution: ::prost::alloc::string::String,
    /// last updated value for the dynamic min deposit
    #[prost(message, optional, tag = "10")]
    pub last_min_deposit: ::core::option::Option<LastMinDeposit>,
    /// last updated value for the dynamic min initial deposit
    #[prost(message, optional, tag = "11")]
    pub last_min_initial_deposit: ::core::option::Option<LastMinDeposit>,
    /// governance participation EMA
    /// If unset or set to 0, the quorum for the next proposal will be set to the
    /// params.MinQuorum value.
    #[prost(string, tag = "12")]
    pub participation_ema: ::prost::alloc::string::String,
    /// governance participation EMA for constitution amendment proposals.
    /// If unset or set to 0, the quorum for the next constitution amendment
    /// proposal will be set to the params.MinConstitutionAmendmentQuorum value.
    #[prost(string, tag = "13")]
    pub constitution_amendment_participation_ema: ::prost::alloc::string::String,
    /// governance participation EMA for law proposals.
    /// If unset or set to 0, the quorum for the next law proposal will be set to
    /// the params.LawMinQuorum value.
    #[prost(string, tag = "14")]
    pub law_participation_ema: ::prost::alloc::string::String,
    /// governors defines all the governors present at genesis.
    #[prost(message, repeated, tag = "15")]
    pub governors: ::prost::alloc::vec::Vec<Governor>,
    /// governance_delegations defines all the governance delegations present at genesis.
    #[prost(message, repeated, tag = "16")]
    pub governance_delegations: ::prost::alloc::vec::Vec<GovernanceDelegation>,
}
/// GovernanceDelegation defines a delegation of governance voting power from a
/// delegator to a governor.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GovernanceDelegation {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub governor_address: ::prost::alloc::string::String,
}
/// Governor defines a governor, together with the total amount of delegated
/// validator's bond shares for a set amount of validators. When a delegator
/// delegates a percentage of its x/gov power to a governor, the resulting
/// shares from each delegators delegations in x/staking are added to the
/// governor's total shares.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Governor {
    /// governor_address defines the address of the governor; bech32-encoded.
    #[prost(string, tag = "1")]
    pub governor_address: ::prost::alloc::string::String,
    /// status is the status of the governor (active/inactive).
    #[prost(enumeration = "GovernorStatus", tag = "2")]
    pub status: i32,
    /// description defines the description terms for the governor.
    #[prost(message, optional, tag = "3")]
    pub description: ::core::option::Option<GovernorDescription>,
    /// last_status_change_time is the time when the governor's status was last changed.
    #[prost(message, optional, tag = "4")]
    pub last_status_change_time:
        ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
}
/// GovernorStatus is the status of a governor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, :: prost :: Enumeration)]
#[repr(i32)]
pub enum GovernorStatus {
    /// UNSPECIFIED defines an invalid governor status.
    Unspecified = 0,
    /// ACTIVE defines a governor that is active.
    Active = 1,
    /// INACTIVE defines a governor that is inactive.
    Inactive = 2,
}
/// GovernorValShares holds the number of virtual shares from the
/// specific validator that a governor can use to vote on proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GovernorValShares {
    #[prost(string, tag = "1")]
    pub governor_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    /// shares define the delegation shares available from this validator.
    #[prost(string, tag = "3")]
    pub shares: ::prost::alloc::string::String,
}
/// LastMinDeposit is a record of the last time the minimum deposit
/// was updated in the store, both its value and a timestamp
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct LastMinDeposit {
    /// value is the value of the minimum deposit
    #[prost(message, repeated, tag = "1")]
    pub value: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// time is the time the minimum deposit was last updated
    #[prost(message, optional, tag = "2")]
    pub time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
}
/// MsgConstitutionAmendment is the Msg/ProposeConstitutionAmendment request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgProposeConstitutionAmendment {
    /// authority is the address that controls the module (defaults to x/gov unless
    /// overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// amendment is the amendment to the constitution. It must be in valid GNU patch format.
    #[prost(string, tag = "2")]
    pub amendment: ::prost::alloc::string::String,
}
/// MsgCreateGovernor defines a SDK message for creating a new governor.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgCreateGovernor {
    /// address is the base account address that is creating the governor.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub description: ::core::option::Option<GovernorDescription>,
}
/// MsgCreateGovernorrResponse defines the Msg/CreateGovernor response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgCreateGovernorResponse {}
/// MsgDelegateGovernor defines a SDK message for performing a delegation of governance voting power
/// from a delegator to a governor.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgDelegateGovernor {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub governor_address: ::prost::alloc::string::String,
}
/// MsgDelegateGovernorResponse defines the Msg/Delegate response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgDelegateGovernorResponse {}
/// MsgDeposit defines a message to submit a deposit to an existing proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgDeposit {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// depositor defines the deposit addresses from the proposals.
    #[prost(string, tag = "2")]
    pub depositor: ::prost::alloc::string::String,
    /// amount to be deposited by depositor.
    #[prost(message, repeated, tag = "3")]
    pub amount: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// MsgDepositResponse defines the Msg/Deposit response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgDepositResponse {}
/// MsgEditGovernor defines a SDK message for editing an existing governor.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgEditGovernor {
    /// address is the base account address that is editing the corresponding governor.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub description: ::core::option::Option<GovernorDescription>,
}
/// MsgEditGovernorResponse defines the Msg/EditGovernor response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgEditGovernorResponse {}
/// MsgExecLegacyContent is used to wrap the legacy content field into a message.
/// This ensures backwards compatibility with v1beta1.MsgSubmitProposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgExecLegacyContent {
    /// content is the proposal's content.
    #[prost(message, optional, tag = "1")]
    pub content: ::core::option::Option<super::super::super::google::protobuf::Any>,
    /// authority must be the gov module address.
    #[prost(string, tag = "2")]
    pub authority: ::prost::alloc::string::String,
}
/// MsgExecLegacyContentResponse defines the Msg/ExecLegacyContent response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgExecLegacyContentResponse {}
/// MsgLaw is the Msg/ProposeLaw request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgProposeLaw {
    /// authority is the address that controls the module (defaults to x/gov unless
    /// overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
}
/// MsgProposeConstitutionAmendmentResponse defines the response structure for executing a
/// MsgProposeConstitutionAmendment message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgProposeConstitutionAmendmentResponse {}
/// MsgProposeLawResponse defines the response structure for executing a
/// MsgProposeLaw message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgProposeLawResponse {}
/// MsgSubmitProposal defines an sdk.Msg type that supports submitting arbitrary
/// proposal Content.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgSubmitProposal {
    /// messages are the arbitrary messages to be executed if proposal passes.
    #[prost(message, repeated, tag = "1")]
    pub messages: ::prost::alloc::vec::Vec<super::super::super::google::protobuf::Any>,
    /// initial_deposit is the deposit value that must be paid at proposal
    /// submission.
    #[prost(message, repeated, tag = "2")]
    pub initial_deposit: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
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
}
/// MsgSubmitProposalResponse defines the Msg/SubmitProposal response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgSubmitProposalResponse {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
}
/// MsgUndelegateGovernor defines a SDK message for undelegating governance voting power
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUndelegateGovernor {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
/// MsgUndelegateGovernorResponse defines the Msg/UndelegateGovernor response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUndelegateGovernorResponse {}
/// MsgUpdateGovernorStatus defines a SDK message for updating the status of a governor.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateGovernorStatus {
    /// address is the base account address that is editing the corresponding governor.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(enumeration = "GovernorStatus", tag = "2")]
    pub status: i32,
}
/// MsgUpdateGovernorStatusResponse defines the Msg/UpdateGovernorStatus response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateGovernorStatusResponse {}
/// MsgUpdateParams is the Msg/UpdateParams request type.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless
    /// overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/gov parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateParamsResponse {}
/// MsgVote defines a message to cast a vote.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// MsgVoteResponse defines the Msg/Vote response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgVoteResponse {}
/// MsgVoteWeighted defines a message to cast a vote.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// MsgVoteWeightedResponse defines the Msg/VoteWeighted response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgVoteWeightedResponse {}
/// Params defines the parameters for the x/gov module.
///
/// Since: cosmos-sdk 0.47
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Params {
    /// Minimum deposit for a proposal to enter voting period.
    /// Deprecated: a dynamic system now determines the minimum deposit,
    /// see the other params inside the min_deposit_throttler field.
    /// While setting this value returns an error, when queried it is set to the
    /// value of the current minimum deposit value as determined by the dynamic
    /// system for backward compatibility.
    #[deprecated]
    #[prost(message, repeated, tag = "1")]
    pub min_deposit: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Maximum period for Atom holders to deposit on a proposal. Initial value: 2
    /// months.
    #[prost(message, optional, tag = "2")]
    pub max_deposit_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// Duration of the voting period.
    #[prost(message, optional, tag = "3")]
    pub voting_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    ///   Minimum percentage of total stake needed to vote for a result to be
    ///   considered valid. Default value: 0.25.
    #[deprecated]
    #[prost(string, tag = "4")]
    pub quorum: ::prost::alloc::string::String,
    ///   Minimum proportion of Yes votes for proposal to pass. Default value: 2/3.
    #[prost(string, tag = "5")]
    pub threshold: ::prost::alloc::string::String,
    ///   The ratio representing the proportion of the deposit value that must be paid at proposal submission.
    #[deprecated]
    #[prost(string, tag = "7")]
    pub min_initial_deposit_ratio: ::prost::alloc::string::String,
    /// burn deposits if a proposal does not meet quorum
    #[prost(bool, tag = "13")]
    pub burn_vote_quorum: bool,
    /// burn deposits if the proposal does not enter voting period
    #[prost(bool, tag = "14")]
    pub burn_proposal_deposit_prevote: bool,
    /// The ratio representing the proportion of the deposit value minimum that
    /// must be met when making a deposit. Default value: 0.01. Meaning that for a
    /// chain with a min_deposit of 100stake, a deposit of 1stake would be
    /// required.
    ///
    /// Since: cosmos-sdk 0.50
    /// NOTE: backported from v50 (<https://github.com/cosmos/cosmos-sdk/pull/18146>)
    #[prost(string, tag = "15")]
    pub min_deposit_ratio: ::prost::alloc::string::String,
    /// quorum for constitution amendment proposals
    #[deprecated]
    #[prost(string, tag = "16")]
    pub constitution_amendment_quorum: ::prost::alloc::string::String,
    /// Minimum proportion of Yes votes for a Constitution Amendment proposal to pass. Default value: 0.9.
    #[prost(string, tag = "17")]
    pub constitution_amendment_threshold: ::prost::alloc::string::String,
    /// quorum for law proposals
    #[deprecated]
    #[prost(string, tag = "18")]
    pub law_quorum: ::prost::alloc::string::String,
    /// Minimum proportion of Yes votes for a Law proposal to pass. Default value: 0.9.
    #[prost(string, tag = "19")]
    pub law_threshold: ::prost::alloc::string::String,
    /// Duration of time after a proposal enters the voting period, during which quorum
    /// must be achieved to not incur in a voting period extension.
    #[prost(message, optional, tag = "20")]
    pub quorum_timeout: ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// Duration that expresses the maximum amount of time by which a proposal voting period
    /// can be extended.
    #[prost(message, optional, tag = "21")]
    pub max_voting_period_extension:
        ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// Number of times a proposal should be checked for quorum after the quorum timeout
    /// has elapsed. Used to compute the amount of time in between quorum checks.
    #[prost(uint64, tag = "22")]
    pub quorum_check_count: u64,
    #[prost(message, optional, tag = "23")]
    pub min_deposit_throttler: ::core::option::Option<MinDepositThrottler>,
    #[prost(message, optional, tag = "24")]
    pub min_initial_deposit_throttler: ::core::option::Option<MinInitialDepositThrottler>,
    /// Minimum proportion of No Votes for a proposal deposit to be burnt.
    #[prost(string, tag = "25")]
    pub burn_deposit_no_threshold: ::prost::alloc::string::String,
    /// Achievable quorum
    #[prost(message, optional, tag = "26")]
    pub quorum_range: ::core::option::Option<QuorumRange>,
    /// Achievable quorum for constitution amendment proposals
    #[prost(message, optional, tag = "27")]
    pub constitution_amendment_quorum_range: ::core::option::Option<QuorumRange>,
    /// Achievable quorum for law proposals
    #[prost(message, optional, tag = "28")]
    pub law_quorum_range: ::core::option::Option<QuorumRange>,
    /// Defines the duration of time that need to elapse between governor status changes.
    #[prost(message, optional, tag = "29")]
    pub governor_status_change_period:
        ::core::option::Option<super::super::super::google::protobuf::Duration>,
    /// Defines the minimum amound of bonded tokens, aka the "self-delegation" (because active governors
    /// must have the governance VP from the base account automatically delegated to them), that a governor
    /// must have to be considered active.
    #[prost(string, tag = "30")]
    pub min_governor_self_delegation: ::prost::alloc::string::String,
}
/// Proposal defines the core field members of a governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    pub total_deposit: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// voting_start_time is the starting time to vote on a proposal.
    #[prost(message, optional, tag = "8")]
    pub voting_start_time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// voting_end_time is the end time of voting on a proposal.
    #[prost(message, optional, tag = "9")]
    pub voting_end_time: ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// metadata is any arbitrary metadata attached to the proposal.
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
    /// Proposer is the address of the proposal sumbitter
    ///
    /// Since: cosmos-sdk 0.47
    #[prost(string, tag = "13")]
    pub proposer: ::prost::alloc::string::String,
    /// endorsed is a boolean indicating whether the proposal has been endorsed
    /// by the Steering DAO.
    #[prost(bool, tag = "14")]
    pub endorsed: bool,
    /// annotation is an optional field that contains annotations
    /// added by the Steering DAO.
    #[prost(string, tag = "15")]
    pub annotation: ::prost::alloc::string::String,
    /// times_voting_period_extended is the number of times the voting period
    /// has been extended from one of the core DAOs.
    #[prost(uint32, tag = "16")]
    pub times_voting_period_extended: u32,
}
/// ProposalStatus enumerates the valid statuses of a proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, :: prost :: Enumeration)]
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
    /// PROPOSAL_STATUS_VETOED defines a proposal status of a proposal that has
    /// been vetoed.
    Vetoed = 6,
}
/// QueryConstitutionRequest is the request type for the Query/Constitution RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryConstitutionRequest {}
/// QueryConstitutionResponse is the response type for the Query/Constitution RPC method
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryConstitutionResponse {
    #[prost(string, tag = "1")]
    pub constitution: ::prost::alloc::string::String,
}
/// QueryDepositRequest is the request type for the Query/Deposit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryDepositResponse {
    /// deposit defines the requested deposit.
    #[prost(message, optional, tag = "1")]
    pub deposit: ::core::option::Option<Deposit>,
}
/// QueryDepositsRequest is the request type for the Query/Deposits RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryDepositsRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryDepositsResponse is the response type for the Query/Deposits RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryDepositsResponse {
    /// deposits defines the requested deposits.
    #[prost(message, repeated, tag = "1")]
    pub deposits: ::prost::alloc::vec::Vec<Deposit>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryGovernanceDelegationRequest is the request type for the Query/GovernanceDelegation RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernanceDelegationRequest {
    /// delegator_address defines the address of the delegator.
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
}
/// QueryGovernanceDelegationResponse is the response type for the Query/GovernanceDelegation RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernanceDelegationResponse {
    /// governor_address defines the address of the governor.
    #[prost(string, tag = "1")]
    pub governor_address: ::prost::alloc::string::String,
}
/// QueryGovernanceDelegationsRequest is the request type for the Query/GovernanceDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernanceDelegationsRequest {
    /// governor_address defines the address of the governor.
    #[prost(string, tag = "1")]
    pub governor_address: ::prost::alloc::string::String,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryGovernanceDelegationsResponse is the response type for the Query/GovernanceDelegations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernanceDelegationsResponse {
    /// delegations defines the requested delegations.
    #[prost(message, repeated, tag = "1")]
    pub delegations: ::prost::alloc::vec::Vec<GovernanceDelegation>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryGovernorRequest is the request type for the Query/Governor RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernorRequest {
    /// governor_address defines the address of the governor.
    #[prost(string, tag = "1")]
    pub governor_address: ::prost::alloc::string::String,
}
/// QueryGovernorResponse is the response type for the Query/Governor RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernorResponse {
    /// governor defines the requested governor.
    #[prost(message, optional, tag = "1")]
    pub governor: ::core::option::Option<Governor>,
}
/// QueryGovernorValSharesRequest is the request type for the Query/GovernorValShares RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernorValSharesRequest {
    /// governor_address defines the address of the governor.
    #[prost(string, tag = "1")]
    pub governor_address: ::prost::alloc::string::String,
    /// pagination defines the pagination in the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryGovernorValSharesResponse is the response type for the Query/GovernorValShares RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernorValSharesResponse {
    /// val_shares defines the requested validator shares.
    #[prost(message, repeated, tag = "1")]
    pub val_shares: ::prost::alloc::vec::Vec<GovernorValShares>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryGovernorsRequest is the request type for the Query/Governors RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernorsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "1")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryGovernorsResponse is the response type for the Query/Governors RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryGovernorsResponse {
    /// governors defines the requested governors.
    #[prost(message, repeated, tag = "1")]
    pub governors: ::prost::alloc::vec::Vec<Governor>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryMinDepositRequest is the request type for the Query/MinDeposit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryMinDepositRequest {}
/// QueryMinDepositResponse is the response type for the Query/MinDeposit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryMinDepositResponse {
    /// min_deposit defines the minimum deposit required for a proposal to enter voting period.
    #[prost(message, repeated, tag = "1")]
    pub min_deposit: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// QueryMinInitialDepositRequest is the request type for the Query/MinInitialDeposit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryMinInitialDepositRequest {}
/// QueryMinInitialDepositResponse is the response type for the Query/MinInitialDeposit RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryMinInitialDepositResponse {
    /// min_initial_deposit defines the minimum initial deposit required for a proposal to be submitted.
    #[prost(message, repeated, tag = "1")]
    pub min_initial_deposit:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// QueryParamsRequest is the request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryParamsRequest {
    /// params_type defines which parameters to query for, can be one of "voting",
    /// "tallying" or "deposit".
    #[prost(string, tag = "1")]
    pub params_type: ::prost::alloc::string::String,
}
/// QueryParamsResponse is the response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
/// QueryParticipationEMAsRequest is the request type for the Query/ParticipationEMAs RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryParticipationEmAsRequest {}
/// QueryParticipationEMAsResponse is the response type for the Query/ParticipationEMAs RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryParticipationEmAsResponse {
    /// participation_ema defines the requested participation EMA for proposals.
    #[prost(string, tag = "1")]
    pub participation_ema: ::prost::alloc::string::String,
    /// constitution_amendment_participation_ema defines the requested participation EMA for
    /// constitution amendment proposals.
    #[prost(string, tag = "2")]
    pub constitution_amendment_participation_ema: ::prost::alloc::string::String,
    /// law_participation_ema defines the requestedparticipation EMA for law proposals.
    #[prost(string, tag = "3")]
    pub law_participation_ema: ::prost::alloc::string::String,
}
/// QueryProposalRequest is the request type for the Query/Proposal RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryProposalRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
}
/// QueryProposalResponse is the response type for the Query/Proposal RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryProposalResponse {
    /// proposal is the requested governance proposal.
    #[prost(message, optional, tag = "1")]
    pub proposal: ::core::option::Option<Proposal>,
}
/// QueryProposalsRequest is the request type for the Query/Proposals RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryProposalsResponse is the response type for the Query/Proposals RPC
/// method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryProposalsResponse {
    /// proposals defines all the requested governance proposals.
    #[prost(message, repeated, tag = "1")]
    pub proposals: ::prost::alloc::vec::Vec<Proposal>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QueryQuorumsRequest is the request type for the Query/Quorums RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryQuorumsRequest {}
/// QueryQuorumsResponse is the response type for the Query/Quorums RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryQuorumsResponse {
    /// quorum defines the requested quorum.
    #[prost(string, tag = "1")]
    pub quorum: ::prost::alloc::string::String,
    /// constitution_amendment_quorum defines the requested quorum for
    /// constitution amendment proposals.
    #[prost(string, tag = "2")]
    pub constitution_amendment_quorum: ::prost::alloc::string::String,
    /// law_quorum defines the requested quorum for law proposals.
    #[prost(string, tag = "3")]
    pub law_quorum: ::prost::alloc::string::String,
}
/// QueryTallyResultRequest is the request type for the Query/Tally RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryTallyResultRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
}
/// QueryTallyResultResponse is the response type for the Query/Tally RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryTallyResultResponse {
    /// tally defines the requested tally.
    #[prost(message, optional, tag = "1")]
    pub tally: ::core::option::Option<TallyResult>,
}
/// QueryVoteRequest is the request type for the Query/Vote RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryVoteResponse {
    /// vote defines the queried vote.
    #[prost(message, optional, tag = "1")]
    pub vote: ::core::option::Option<Vote>,
}
/// QueryVotesRequest is the request type for the Query/Votes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryVotesRequest {
    /// proposal_id defines the unique id of the proposal.
    #[prost(uint64, tag = "1")]
    pub proposal_id: u64,
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
}
/// QueryVotesResponse is the response type for the Query/Votes RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryVotesResponse {
    /// votes defines the queried votes.
    #[prost(message, repeated, tag = "1")]
    pub votes: ::prost::alloc::vec::Vec<Vote>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// QuorumCheckQueueEntry defines a quorum check queue entry.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QuorumCheckQueueEntry {
    /// quorum_timeout_time is the time after which quorum checks start happening
    /// and voting period is extended if proposal reaches quorum.
    #[prost(message, optional, tag = "1")]
    pub quorum_timeout_time:
        ::core::option::Option<super::super::super::google::protobuf::Timestamp>,
    /// quorum_check_count is the number of times quorum will be checked.
    /// This is a snapshot of the parameter value with the same name when the
    /// proposal is initially added to the queue.
    #[prost(uint64, tag = "2")]
    pub quorum_check_count: u64,
    /// quorum_checks_done is the number of quorum checks that have been done.
    #[prost(uint64, tag = "3")]
    pub quorum_checks_done: u64,
}
/// TallyParams defines the params for tallying votes on governance proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct TallyParams {
    /// Minimum percentage of total stake needed to vote for a result to be
    /// considered valid.
    #[deprecated]
    #[prost(string, tag = "1")]
    pub quorum: ::prost::alloc::string::String,
    /// Minimum proportion of Yes votes for proposal to pass. Default value: 2/3.
    #[prost(string, tag = "2")]
    pub threshold: ::prost::alloc::string::String,
    /// quorum for constitution amendment proposals
    #[deprecated]
    #[prost(string, tag = "3")]
    pub constitution_amendment_quorum: ::prost::alloc::string::String,
    /// Minimum proportion of Yes votes for a Constitution Amendment proposal to pass. Default value: 0.9.
    #[prost(string, tag = "4")]
    pub constitution_amendment_threshold: ::prost::alloc::string::String,
    /// quorum for law proposals
    #[deprecated]
    #[prost(string, tag = "5")]
    pub law_quorum: ::prost::alloc::string::String,
    /// Minimum proportion of Yes votes for a Law proposal to pass. Default value: 0.9.
    #[prost(string, tag = "6")]
    pub law_threshold: ::prost::alloc::string::String,
}
/// TallyResult defines a standard tally for a governance proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
}
/// Vote defines a vote on a governance proposal.
/// A Vote consists of a proposal ID, the voter, and the vote option.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
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
    /// metadata is any  arbitrary metadata to attached to the vote.
    #[prost(string, tag = "5")]
    pub metadata: ::prost::alloc::string::String,
}
/// VoteOption enumerates the valid vote options for a given governance proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, :: prost :: Enumeration)]
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
}
/// VotingParams defines the params for voting on governance proposals.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct VotingParams {
    /// Duration of the voting period.
    #[prost(message, optional, tag = "1")]
    pub voting_period: ::core::option::Option<super::super::super::google::protobuf::Duration>,
}
/// WeightedVoteOption defines a unit of vote for vote split.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct WeightedVoteOption {
    /// option defines the valid vote options, it must not contain duplicate vote
    /// options.
    #[prost(enumeration = "VoteOption", tag = "1")]
    pub option: i32,
    /// weight is the vote weight associated with the vote option.
    #[prost(string, tag = "2")]
    pub weight: ::prost::alloc::string::String,
}
impl ::prost::Name for Deposit {
    const NAME: &'static str = "Deposit";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for DepositParams {
    const NAME: &'static str = "DepositParams";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GovernanceDelegation {
    const NAME: &'static str = "GovernanceDelegation";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Governor {
    const NAME: &'static str = "Governor";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GovernorDescription {
    const NAME: &'static str = "GovernorDescription";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for GovernorValShares {
    const NAME: &'static str = "GovernorValShares";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for LastMinDeposit {
    const NAME: &'static str = "LastMinDeposit";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MinDepositThrottler {
    const NAME: &'static str = "MinDepositThrottler";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MinInitialDepositThrottler {
    const NAME: &'static str = "MinInitialDepositThrottler";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgCreateGovernor {
    const NAME: &'static str = "MsgCreateGovernor";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgCreateGovernorResponse {
    const NAME: &'static str = "MsgCreateGovernorResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgDelegateGovernor {
    const NAME: &'static str = "MsgDelegateGovernor";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgDelegateGovernorResponse {
    const NAME: &'static str = "MsgDelegateGovernorResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgDeposit {
    const NAME: &'static str = "MsgDeposit";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgDepositResponse {
    const NAME: &'static str = "MsgDepositResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgEditGovernor {
    const NAME: &'static str = "MsgEditGovernor";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgEditGovernorResponse {
    const NAME: &'static str = "MsgEditGovernorResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgExecLegacyContent {
    const NAME: &'static str = "MsgExecLegacyContent";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgExecLegacyContentResponse {
    const NAME: &'static str = "MsgExecLegacyContentResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgProposeConstitutionAmendment {
    const NAME: &'static str = "MsgProposeConstitutionAmendment";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgProposeConstitutionAmendmentResponse {
    const NAME: &'static str = "MsgProposeConstitutionAmendmentResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgProposeLaw {
    const NAME: &'static str = "MsgProposeLaw";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgProposeLawResponse {
    const NAME: &'static str = "MsgProposeLawResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgSubmitProposal {
    const NAME: &'static str = "MsgSubmitProposal";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgSubmitProposalResponse {
    const NAME: &'static str = "MsgSubmitProposalResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUndelegateGovernor {
    const NAME: &'static str = "MsgUndelegateGovernor";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUndelegateGovernorResponse {
    const NAME: &'static str = "MsgUndelegateGovernorResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateGovernorStatus {
    const NAME: &'static str = "MsgUpdateGovernorStatus";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateGovernorStatusResponse {
    const NAME: &'static str = "MsgUpdateGovernorStatusResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgVote {
    const NAME: &'static str = "MsgVote";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgVoteResponse {
    const NAME: &'static str = "MsgVoteResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgVoteWeighted {
    const NAME: &'static str = "MsgVoteWeighted";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgVoteWeightedResponse {
    const NAME: &'static str = "MsgVoteWeightedResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Proposal {
    const NAME: &'static str = "Proposal";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryConstitutionRequest {
    const NAME: &'static str = "QueryConstitutionRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryConstitutionResponse {
    const NAME: &'static str = "QueryConstitutionResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryDepositRequest {
    const NAME: &'static str = "QueryDepositRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryDepositResponse {
    const NAME: &'static str = "QueryDepositResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryDepositsRequest {
    const NAME: &'static str = "QueryDepositsRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryDepositsResponse {
    const NAME: &'static str = "QueryDepositsResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernanceDelegationRequest {
    const NAME: &'static str = "QueryGovernanceDelegationRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernanceDelegationResponse {
    const NAME: &'static str = "QueryGovernanceDelegationResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernanceDelegationsRequest {
    const NAME: &'static str = "QueryGovernanceDelegationsRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernanceDelegationsResponse {
    const NAME: &'static str = "QueryGovernanceDelegationsResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernorRequest {
    const NAME: &'static str = "QueryGovernorRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernorResponse {
    const NAME: &'static str = "QueryGovernorResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernorValSharesRequest {
    const NAME: &'static str = "QueryGovernorValSharesRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernorValSharesResponse {
    const NAME: &'static str = "QueryGovernorValSharesResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernorsRequest {
    const NAME: &'static str = "QueryGovernorsRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryGovernorsResponse {
    const NAME: &'static str = "QueryGovernorsResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryMinDepositRequest {
    const NAME: &'static str = "QueryMinDepositRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryMinDepositResponse {
    const NAME: &'static str = "QueryMinDepositResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryMinInitialDepositRequest {
    const NAME: &'static str = "QueryMinInitialDepositRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryMinInitialDepositResponse {
    const NAME: &'static str = "QueryMinInitialDepositResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryParticipationEmAsRequest {
    const NAME: &'static str = "QueryParticipationEMAsRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryParticipationEmAsResponse {
    const NAME: &'static str = "QueryParticipationEMAsResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryProposalRequest {
    const NAME: &'static str = "QueryProposalRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryProposalResponse {
    const NAME: &'static str = "QueryProposalResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryProposalsRequest {
    const NAME: &'static str = "QueryProposalsRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryProposalsResponse {
    const NAME: &'static str = "QueryProposalsResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryQuorumsRequest {
    const NAME: &'static str = "QueryQuorumsRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryQuorumsResponse {
    const NAME: &'static str = "QueryQuorumsResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryTallyResultRequest {
    const NAME: &'static str = "QueryTallyResultRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryTallyResultResponse {
    const NAME: &'static str = "QueryTallyResultResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryVoteRequest {
    const NAME: &'static str = "QueryVoteRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryVoteResponse {
    const NAME: &'static str = "QueryVoteResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryVotesRequest {
    const NAME: &'static str = "QueryVotesRequest";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryVotesResponse {
    const NAME: &'static str = "QueryVotesResponse";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QuorumCheckQueueEntry {
    const NAME: &'static str = "QuorumCheckQueueEntry";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QuorumRange {
    const NAME: &'static str = "QuorumRange";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for TallyParams {
    const NAME: &'static str = "TallyParams";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for TallyResult {
    const NAME: &'static str = "TallyResult";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Vote {
    const NAME: &'static str = "Vote";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for VotingParams {
    const NAME: &'static str = "VotingParams";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for WeightedVoteOption {
    const NAME: &'static str = "WeightedVoteOption";
    const PACKAGE: &'static str = "atomone.gov.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.gov.v1.{}", Self::NAME)
    }
}
impl GovernorStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GovernorStatus::Unspecified => "GOVERNOR_STATUS_UNSPECIFIED",
            GovernorStatus::Active => "GOVERNOR_STATUS_ACTIVE",
            GovernorStatus::Inactive => "GOVERNOR_STATUS_INACTIVE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "GOVERNOR_STATUS_UNSPECIFIED" => Some(Self::Unspecified),
            "GOVERNOR_STATUS_ACTIVE" => Some(Self::Active),
            "GOVERNOR_STATUS_INACTIVE" => Some(Self::Inactive),
            _ => None,
        }
    }
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
            ProposalStatus::Vetoed => "PROPOSAL_STATUS_VETOED",
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
            "PROPOSAL_STATUS_VETOED" => Some(Self::Vetoed),
            _ => None,
        }
    }
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
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "VOTE_OPTION_UNSPECIFIED" => Some(Self::Unspecified),
            "VOTE_OPTION_YES" => Some(Self::Yes),
            "VOTE_OPTION_ABSTAIN" => Some(Self::Abstain),
            "VOTE_OPTION_NO" => Some(Self::No),
            _ => None,
        }
    }
}
