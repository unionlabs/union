// @generated
/// IdentifiedClientState defines a client state with an additional client
/// identifier field.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedClientState {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// client state
    #[prost(message, optional, tag = "2")]
    pub client_state: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for IdentifiedClientState {
    const NAME: &'static str = "IdentifiedClientState";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// ConsensusStateWithHeight defines a consensus state with an additional height
/// field.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConsensusStateWithHeight {
    /// consensus state height
    #[prost(message, optional, tag = "1")]
    pub height: ::core::option::Option<Height>,
    /// consensus state
    #[prost(message, optional, tag = "2")]
    pub consensus_state: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for ConsensusStateWithHeight {
    const NAME: &'static str = "ConsensusStateWithHeight";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// ClientConsensusStates defines all the stored consensus states for a given
/// client.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientConsensusStates {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// consensus states and their heights associated with the client
    #[prost(message, repeated, tag = "2")]
    pub consensus_states: ::prost::alloc::vec::Vec<ConsensusStateWithHeight>,
}
impl ::prost::Name for ClientConsensusStates {
    const NAME: &'static str = "ClientConsensusStates";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// Height is a monotonically increasing data type
/// that can be compared against another Height for the purposes of updating and
/// freezing clients
///
/// Normally the RevisionHeight is incremented at each height while keeping
/// RevisionNumber the same. However some consensus algorithms may choose to
/// reset the height in certain conditions e.g. hard forks, state-machine
/// breaking changes In these cases, the RevisionNumber is incremented so that
/// height continues to be monitonically increasing even as the RevisionHeight
/// gets reset
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Height {
    /// the revision that the client is currently on
    #[prost(uint64, tag = "1")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub revision_number: u64,
    /// the height within the given revision
    #[prost(uint64, tag = "2")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub revision_height: u64,
}
impl ::prost::Name for Height {
    const NAME: &'static str = "Height";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// Params defines the set of IBC light client parameters.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// allowed_clients defines the list of allowed client state types which can be created
    /// and interacted with. If a client type is removed from the allowed clients list, usage
    /// of this client will be disabled until it is added again to the list.
    #[prost(string, repeated, tag = "1")]
    pub allowed_clients: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
impl ::prost::Name for Params {
    const NAME: &'static str = "Params";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// ClientUpdateProposal is a legacy governance proposal. If it passes, the substitute
/// client's latest consensus state is copied over to the subject client. The proposal
/// handler may fail if the subject and the substitute do not match in client and
/// chain parameters (with exception to latest height, frozen height, and chain-id).
///
/// Deprecated: Please use MsgRecoverClient in favour of this message type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientUpdateProposal {
    /// the title of the update proposal
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    /// the description of the proposal
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// the client identifier for the client to be updated if the proposal passes
    #[prost(string, tag = "3")]
    pub subject_client_id: ::prost::alloc::string::String,
    /// the substitute client identifier for the client standing in for the subject
    /// client
    #[prost(string, tag = "4")]
    pub substitute_client_id: ::prost::alloc::string::String,
}
impl ::prost::Name for ClientUpdateProposal {
    const NAME: &'static str = "ClientUpdateProposal";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// UpgradeProposal is a gov Content type for initiating an IBC breaking
/// upgrade.
///
/// Deprecated: Please use MsgIBCSoftwareUpgrade in favour of this message type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpgradeProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    /// An UpgradedClientState must be provided to perform an IBC breaking upgrade.
    /// This will make the chain commit to the correct upgraded (self) client state
    /// before the upgrade occurs, so that connecting chains can verify that the
    /// new upgraded client is valid by verifying a proof on the previous version
    /// of the chain. This will allow IBC connections to persist smoothly across
    /// planned chain upgrades
    #[prost(message, optional, tag = "4")]
    pub upgraded_client_state: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for UpgradeProposal {
    const NAME: &'static str = "UpgradeProposal";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// GenesisState defines the ibc client submodule's genesis state.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisState {
    /// client states with their corresponding identifiers
    #[prost(message, repeated, tag = "1")]
    pub clients: ::prost::alloc::vec::Vec<IdentifiedClientState>,
    /// consensus states from each client
    #[prost(message, repeated, tag = "2")]
    pub clients_consensus: ::prost::alloc::vec::Vec<ClientConsensusStates>,
    /// metadata from each client
    #[prost(message, repeated, tag = "3")]
    pub clients_metadata: ::prost::alloc::vec::Vec<IdentifiedGenesisMetadata>,
    #[prost(message, optional, tag = "4")]
    pub params: ::core::option::Option<Params>,
    /// Deprecated: create_localhost has been deprecated.
    /// The localhost client is automatically created at genesis.
    #[deprecated]
    #[prost(bool, tag = "5")]
    pub create_localhost: bool,
    /// the sequence for the next generated client identifier
    #[prost(uint64, tag = "6")]
    pub next_client_sequence: u64,
}
impl ::prost::Name for GenesisState {
    const NAME: &'static str = "GenesisState";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// GenesisMetadata defines the genesis type for metadata that clients may return
/// with ExportMetadata
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GenesisMetadata {
    /// store key of metadata without clientID-prefix
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
    /// metadata value
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for GenesisMetadata {
    const NAME: &'static str = "GenesisMetadata";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// IdentifiedGenesisMetadata has the client metadata with the corresponding
/// client id.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IdentifiedGenesisMetadata {
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub client_metadata: ::prost::alloc::vec::Vec<GenesisMetadata>,
}
impl ::prost::Name for IdentifiedGenesisMetadata {
    const NAME: &'static str = "IdentifiedGenesisMetadata";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgCreateClient defines a message to create an IBC client
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateClient {
    /// light client state
    #[prost(message, optional, tag = "1")]
    pub client_state: ::core::option::Option<::pbjson_types::Any>,
    /// consensus state associated with the client that corresponds to a given
    /// height.
    #[prost(message, optional, tag = "2")]
    pub consensus_state: ::core::option::Option<::pbjson_types::Any>,
    /// signer address
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgCreateClient {
    const NAME: &'static str = "MsgCreateClient";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgCreateClientResponse defines the Msg/CreateClient response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateClientResponse {}
impl ::prost::Name for MsgCreateClientResponse {
    const NAME: &'static str = "MsgCreateClientResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgUpdateClient defines an sdk.Msg to update a IBC client state using
/// the given client message.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateClient {
    /// client unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// client message to update the light client
    #[prost(message, optional, tag = "2")]
    pub client_message: ::core::option::Option<::pbjson_types::Any>,
    /// signer address
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgUpdateClient {
    const NAME: &'static str = "MsgUpdateClient";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgUpdateClientResponse defines the Msg/UpdateClient response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateClientResponse {}
impl ::prost::Name for MsgUpdateClientResponse {
    const NAME: &'static str = "MsgUpdateClientResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgUpgradeClient defines an sdk.Msg to upgrade an IBC client to a new client
/// state
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpgradeClient {
    /// client unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// upgraded client state
    #[prost(message, optional, tag = "2")]
    pub client_state: ::core::option::Option<::pbjson_types::Any>,
    /// upgraded consensus state, only contains enough information to serve as a
    /// basis of trust in update logic
    #[prost(message, optional, tag = "3")]
    pub consensus_state: ::core::option::Option<::pbjson_types::Any>,
    /// proof that old chain committed to new client
    #[prost(bytes = "vec", tag = "4")]
    pub proof_upgrade_client: ::prost::alloc::vec::Vec<u8>,
    /// proof that old chain committed to new consensus state
    #[prost(bytes = "vec", tag = "5")]
    pub proof_upgrade_consensus_state: ::prost::alloc::vec::Vec<u8>,
    /// signer address
    #[prost(string, tag = "6")]
    pub signer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgUpgradeClient {
    const NAME: &'static str = "MsgUpgradeClient";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgUpgradeClientResponse defines the Msg/UpgradeClient response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpgradeClientResponse {}
impl ::prost::Name for MsgUpgradeClientResponse {
    const NAME: &'static str = "MsgUpgradeClientResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgSubmitMisbehaviour defines an sdk.Msg type that submits Evidence for
/// light client misbehaviour.
/// This message has been deprecated. Use MsgUpdateClient instead.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitMisbehaviour {
    /// client unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// misbehaviour used for freezing the light client
    #[prost(message, optional, tag = "2")]
    pub misbehaviour: ::core::option::Option<::pbjson_types::Any>,
    /// signer address
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgSubmitMisbehaviour {
    const NAME: &'static str = "MsgSubmitMisbehaviour";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgSubmitMisbehaviourResponse defines the Msg/SubmitMisbehaviour response
/// type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitMisbehaviourResponse {}
impl ::prost::Name for MsgSubmitMisbehaviourResponse {
    const NAME: &'static str = "MsgSubmitMisbehaviourResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgRecoverClient defines the message used to recover a frozen or expired client.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRecoverClient {
    /// the client identifier for the client to be updated if the proposal passes
    #[prost(string, tag = "1")]
    pub subject_client_id: ::prost::alloc::string::String,
    /// the substitute client identifier for the client which will replace the subject
    /// client
    #[prost(string, tag = "2")]
    pub substitute_client_id: ::prost::alloc::string::String,
    /// signer address
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgRecoverClient {
    const NAME: &'static str = "MsgRecoverClient";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgRecoverClientResponse defines the Msg/RecoverClient response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRecoverClientResponse {}
impl ::prost::Name for MsgRecoverClientResponse {
    const NAME: &'static str = "MsgRecoverClientResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgIBCSoftwareUpgrade defines the message used to schedule an upgrade of an IBC client using a v1 governance proposal
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgIbcSoftwareUpgrade {
    /// An UpgradedClientState must be provided to perform an IBC breaking upgrade.
    /// This will make the chain commit to the correct upgraded (self) client state
    /// before the upgrade occurs, so that connecting chains can verify that the
    /// new upgraded client is valid by verifying a proof on the previous version
    /// of the chain. This will allow IBC connections to persist smoothly across
    /// planned chain upgrades. Correspondingly, the UpgradedClientState field has been
    /// deprecated in the Cosmos SDK to allow for this logic to exist solely in
    /// the 02-client module.
    #[prost(message, optional, tag = "2")]
    pub upgraded_client_state: ::core::option::Option<::pbjson_types::Any>,
    /// signer address
    #[prost(string, tag = "3")]
    pub signer: ::prost::alloc::string::String,
}
impl ::prost::Name for MsgIbcSoftwareUpgrade {
    const NAME: &'static str = "MsgIBCSoftwareUpgrade";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgIBCSoftwareUpgradeResponse defines the Msg/IBCSoftwareUpgrade response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgIbcSoftwareUpgradeResponse {}
impl ::prost::Name for MsgIbcSoftwareUpgradeResponse {
    const NAME: &'static str = "MsgIBCSoftwareUpgradeResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParams defines the sdk.Msg type to update the client parameters.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    /// signer address
    #[prost(string, tag = "1")]
    pub signer: ::prost::alloc::string::String,
    /// params defines the client parameters to update.
    ///
    /// NOTE: All parameters must be supplied.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for MsgUpdateParams {
    const NAME: &'static str = "MsgUpdateParams";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// MsgUpdateParamsResponse defines the MsgUpdateParams response type.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {}
impl ::prost::Name for MsgUpdateParamsResponse {
    const NAME: &'static str = "MsgUpdateParamsResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryClientStateRequest is the request type for the Query/ClientState RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStateRequest {
    /// client state unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryClientStateRequest {
    const NAME: &'static str = "QueryClientStateRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryClientStateResponse is the response type for the Query/ClientState RPC
/// method. Besides the client state, it includes a proof and the height from
/// which the proof was retrieved.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStateResponse {
    /// client state associated with the request identifier
    #[prost(message, optional, tag = "1")]
    pub client_state: ::core::option::Option<::pbjson_types::Any>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<Height>,
}
impl ::prost::Name for QueryClientStateResponse {
    const NAME: &'static str = "QueryClientStateResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryClientStatesRequest is the request type for the Query/ClientStates RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStatesRequest {
    /// pagination request
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
impl ::prost::Name for QueryClientStatesRequest {
    const NAME: &'static str = "QueryClientStatesRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryClientStatesResponse is the response type for the Query/ClientStates RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStatesResponse {
    /// list of stored ClientStates of the chain.
    #[prost(message, repeated, tag = "1")]
    pub client_states: ::prost::alloc::vec::Vec<IdentifiedClientState>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryClientStatesResponse {
    const NAME: &'static str = "QueryClientStatesResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryConsensusStateRequest is the request type for the Query/ConsensusState
/// RPC method. Besides the consensus state, it includes a proof and the height
/// from which the proof was retrieved.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStateRequest {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// consensus state revision number
    #[prost(uint64, tag = "2")]
    pub revision_number: u64,
    /// consensus state revision height
    #[prost(uint64, tag = "3")]
    pub revision_height: u64,
    /// latest_height overrrides the height field and queries the latest stored
    /// ConsensusState
    #[prost(bool, tag = "4")]
    pub latest_height: bool,
}
impl ::prost::Name for QueryConsensusStateRequest {
    const NAME: &'static str = "QueryConsensusStateRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryConsensusStateResponse is the response type for the Query/ConsensusState
/// RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStateResponse {
    /// consensus state associated with the client identifier at the given height
    #[prost(message, optional, tag = "1")]
    pub consensus_state: ::core::option::Option<::pbjson_types::Any>,
    /// merkle proof of existence
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// height at which the proof was retrieved
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<Height>,
}
impl ::prost::Name for QueryConsensusStateResponse {
    const NAME: &'static str = "QueryConsensusStateResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryConsensusStatesRequest is the request type for the Query/ConsensusStates
/// RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStatesRequest {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// pagination request
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
impl ::prost::Name for QueryConsensusStatesRequest {
    const NAME: &'static str = "QueryConsensusStatesRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryConsensusStatesResponse is the response type for the
/// Query/ConsensusStates RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStatesResponse {
    /// consensus states associated with the identifier
    #[prost(message, repeated, tag = "1")]
    pub consensus_states: ::prost::alloc::vec::Vec<ConsensusStateWithHeight>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryConsensusStatesResponse {
    const NAME: &'static str = "QueryConsensusStatesResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryConsensusStateHeightsRequest is the request type for Query/ConsensusStateHeights
/// RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStateHeightsRequest {
    /// client identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// pagination request
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageRequest,
    >,
}
impl ::prost::Name for QueryConsensusStateHeightsRequest {
    const NAME: &'static str = "QueryConsensusStateHeightsRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryConsensusStateHeightsResponse is the response type for the
/// Query/ConsensusStateHeights RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryConsensusStateHeightsResponse {
    /// consensus state heights
    #[prost(message, repeated, tag = "1")]
    pub consensus_state_heights: ::prost::alloc::vec::Vec<Height>,
    /// pagination response
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<
        super::super::super::super::cosmos::base::query::v1beta1::PageResponse,
    >,
}
impl ::prost::Name for QueryConsensusStateHeightsResponse {
    const NAME: &'static str = "QueryConsensusStateHeightsResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryClientStatusRequest is the request type for the Query/ClientStatus RPC
/// method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStatusRequest {
    /// client unique identifier
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryClientStatusRequest {
    const NAME: &'static str = "QueryClientStatusRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryClientStatusResponse is the response type for the Query/ClientStatus RPC
/// method. It returns the current status of the IBC client.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientStatusResponse {
    #[prost(string, tag = "1")]
    pub status: ::prost::alloc::string::String,
}
impl ::prost::Name for QueryClientStatusResponse {
    const NAME: &'static str = "QueryClientStatusResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryClientParamsRequest is the request type for the Query/ClientParams RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientParamsRequest {}
impl ::prost::Name for QueryClientParamsRequest {
    const NAME: &'static str = "QueryClientParamsRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryClientParamsResponse is the response type for the Query/ClientParams RPC
/// method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryClientParamsResponse {
    /// params defines the parameters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
impl ::prost::Name for QueryClientParamsResponse {
    const NAME: &'static str = "QueryClientParamsResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryUpgradedClientStateRequest is the request type for the
/// Query/UpgradedClientState RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedClientStateRequest {}
impl ::prost::Name for QueryUpgradedClientStateRequest {
    const NAME: &'static str = "QueryUpgradedClientStateRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryUpgradedClientStateResponse is the response type for the
/// Query/UpgradedClientState RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedClientStateResponse {
    /// client state associated with the request identifier
    #[prost(message, optional, tag = "1")]
    pub upgraded_client_state: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for QueryUpgradedClientStateResponse {
    const NAME: &'static str = "QueryUpgradedClientStateResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryUpgradedConsensusStateRequest is the request type for the
/// Query/UpgradedConsensusState RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedConsensusStateRequest {}
impl ::prost::Name for QueryUpgradedConsensusStateRequest {
    const NAME: &'static str = "QueryUpgradedConsensusStateRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryUpgradedConsensusStateResponse is the response type for the
/// Query/UpgradedConsensusState RPC method.
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryUpgradedConsensusStateResponse {
    /// Consensus state associated with the request identifier
    #[prost(message, optional, tag = "1")]
    pub upgraded_consensus_state: ::core::option::Option<::pbjson_types::Any>,
}
impl ::prost::Name for QueryUpgradedConsensusStateResponse {
    const NAME: &'static str = "QueryUpgradedConsensusStateResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryVerifyMembershipRequest is the request type for the Query/VerifyMembership RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVerifyMembershipRequest {
    /// client unique identifier.
    #[prost(string, tag = "1")]
    pub client_id: ::prost::alloc::string::String,
    /// the proof to be verified by the client.
    #[prost(bytes = "vec", tag = "2")]
    pub proof: ::prost::alloc::vec::Vec<u8>,
    /// the height of the commitment root at which the proof is verified.
    #[prost(message, optional, tag = "3")]
    pub proof_height: ::core::option::Option<Height>,
    /// the commitment key path.
    #[prost(message, optional, tag = "4")]
    pub merkle_path: ::core::option::Option<super::super::commitment::v1::MerklePath>,
    /// the value which is proven.
    #[prost(bytes = "vec", tag = "5")]
    pub value: ::prost::alloc::vec::Vec<u8>,
    /// optional time delay
    #[prost(uint64, tag = "6")]
    pub time_delay: u64,
    /// optional block delay
    #[prost(uint64, tag = "7")]
    pub block_delay: u64,
}
impl ::prost::Name for QueryVerifyMembershipRequest {
    const NAME: &'static str = "QueryVerifyMembershipRequest";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
/// QueryVerifyMembershipResponse is the response type for the Query/VerifyMembership RPC method
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryVerifyMembershipResponse {
    /// boolean indicating success or failure of proof verification.
    #[prost(bool, tag = "1")]
    pub success: bool,
}
impl ::prost::Name for QueryVerifyMembershipResponse {
    const NAME: &'static str = "QueryVerifyMembershipResponse";
    const PACKAGE: &'static str = "ibc.core.client.v1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("ibc.core.client.v1.{}", Self::NAME)
    }
}
include!("ibc.core.client.v1.tonic.rs");
// @@protoc_insertion_point(module)
