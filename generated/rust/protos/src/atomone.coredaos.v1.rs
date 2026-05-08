/// GenesisState defines the x/coredaos module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// MsgAnnotateProposal defines a message for annotating a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgAnnotateProposal {
    /// annotator is the address of the dao annotating the proposal.
    #[prost(string, tag = "1")]
    pub annotator: ::prost::alloc::string::String,
    /// proposal_id is the ID of the proposal to annotate.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
    /// annotation is the annotation to add to the proposal.
    #[prost(string, tag = "3")]
    pub annotation: ::prost::alloc::string::String,
    /// overwrite is a boolean indicating whether to overwrite the existing annotation.
    /// Must be set to true if the proposal already has an annotation.
    /// Ignored if the proposal does not have yet an annotation.
    #[prost(bool, tag = "4")]
    pub overwrite: bool,
}
/// MsgAnnotateProposalResponse defines the response for MsgAnnotateProposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgAnnotateProposalResponse {}
/// MsgEndorseProposal defines a message for endorsing a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgEndorseProposal {
    /// endorser is the address of the dao endorsing the proposal.
    #[prost(string, tag = "1")]
    pub endorser: ::prost::alloc::string::String,
    /// proposal_id is the ID of the proposal to endorse.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
}
/// MsgEndorseProposalResponse defines the response for MsgEndorseProposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgEndorseProposalResponse {}
/// MsgExtendVotingPeriod defines a message for extending the voting period of a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgExtendVotingPeriod {
    /// extender is the address of the dao extending the voting period.
    #[prost(string, tag = "1")]
    pub extender: ::prost::alloc::string::String,
    /// proposal_id is the ID of the proposal to extend.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
}
/// MsgExtendVotingPeriodResponse defines the response for MsgExtendVotingPeriod.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgExtendVotingPeriodResponse {}
/// MsgUpdateParams is the Msg/UpdateParams request type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateParams {
    /// authority is the address that controls the module (defaults to x/gov unless
    /// overwritten).
    #[prost(string, tag = "1")]
    pub authority: ::prost::alloc::string::String,
    /// params defines the x/coredaos parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
/// MsgUpdateParamsResponse defines the response structure for executing a
/// MsgUpdateParams message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgUpdateParamsResponse {}
/// MsgVetoProposal defines a message for vetoing a proposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgVetoProposal {
    /// vetoer is the address of the dao vetoing the proposal.
    #[prost(string, tag = "1")]
    pub vetoer: ::prost::alloc::string::String,
    /// proposal_id is the ID of the proposal to veto.
    #[prost(uint64, tag = "2")]
    pub proposal_id: u64,
    /// burn_deposit is a boolean indicating whether to burn the deposit of the proposal.
    /// If true, the deposit is burned and not refunded.
    #[prost(bool, tag = "3")]
    pub burn_deposit: bool,
}
/// MsgVetoProposalResponse defines the response for MsgVetoProposal.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct MsgVetoProposalResponse {}
/// Params defines the parameters for the x/coredaos module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct Params {
    /// steering_dao_address defines the address which has authority
    /// to execute messages as Steering DAO.
    #[prost(string, tag = "1")]
    pub steering_dao_address: ::prost::alloc::string::String,
    /// oversight_dao_address defines the address which has authority
    /// to execute messages as Oversight DAO.
    #[prost(string, tag = "2")]
    pub oversight_dao_address: ::prost::alloc::string::String,
    /// voting_period_extensions_limit defines the maximum number of times
    /// a proposal's voting period can be extended.
    #[prost(uint32, tag = "3")]
    pub voting_period_extensions_limit: u32,
    /// voting_period_extension_duration defines the duration for which
    /// a proposal's voting period can be extended.
    #[prost(message, optional, tag = "4")]
    pub voting_period_extension_duration:
        ::core::option::Option<super::super::super::google::protobuf::Duration>,
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse is response type for the Query/Params RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, :: prost :: Message)]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgAnnotateProposal {
    const NAME: &'static str = "MsgAnnotateProposal";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgAnnotateProposalResponse {
    const NAME: &'static str = "MsgAnnotateProposalResponse";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgEndorseProposal {
    const NAME: &'static str = "MsgEndorseProposal";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgEndorseProposalResponse {
    const NAME: &'static str = "MsgEndorseProposalResponse";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgExtendVotingPeriod {
    const NAME: &'static str = "MsgExtendVotingPeriod";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgExtendVotingPeriodResponse {
    const NAME: &'static str = "MsgExtendVotingPeriodResponse";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgVetoProposal {
    const NAME: &'static str = "MsgVetoProposal";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for MsgVetoProposalResponse {
    const NAME: &'static str = "MsgVetoProposalResponse";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryParamsRequest {
    const NAME: &'static str = "QueryParamsRequest";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
impl ::prost::Name for QueryParamsResponse {
    const NAME: &'static str = "QueryParamsResponse";
    const PACKAGE: &'static str = "atomone.coredaos.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("atomone.coredaos.v1.{}", Self::NAME)
    }
}
