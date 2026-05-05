use std::collections::BTreeMap;

use access_managed::Restricted;
use enumorph::Enumorph;
use ibc_union_spec::ClientId;
use pausable::{
    WhenNotPaused,
    msg::{Pausable, PausableQuery},
};
use serde::{Deserialize, Serialize};
use unionlabs_primitives::Bytes;
use upgradable::msg::Upgradable;
#[cfg(doc)]
use {crate::msg::MsgCreateClient, ibc_union_spec::Status};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct InitMsg {
    pub ibc_host: String,
    pub access_managed_init_msg: access_managed::InitMsg,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrateMsg {}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum ExecuteMsg {
    #[serde(untagged)]
    AccessManaged(access_managed::ExecuteMsg),
    #[serde(untagged)]
    Upgradable(Restricted<Upgradable>),
    #[serde(untagged)]
    Pausable(Restricted<Pausable>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Enumorph)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum QueryMsg {
    /// Get the timestamp of the specified client at the specified height.
    #[enumorph(ignore)]
    GetTimestamp { client_id: ClientId, height: u64 },
    /// Get latest height of the specified client.
    #[enumorph(ignore)]
    GetLatestHeight { client_id: ClientId },
    /// Get [`Status`] of the specified client.
    #[enumorph(ignore)]
    GetStatus { client_id: ClientId },

    /// Verification queries, used mainly by the IBC core module.
    ///
    /// Note that these messages are pausable.
    #[serde(untagged)]
    Verification(WhenNotPaused<VerificationQueryMsg>),

    #[serde(untagged)]
    AccessManaged(access_managed::QueryMsg),
    #[serde(untagged)]
    Pausable(PausableQuery),
}

/// Verification queries, used mainly by the IBC core module.
#[derive(Debug, PartialEq, Serialize, Deserialize, Enumorph)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum VerificationQueryMsg {
    /// Verify the creation of the light client after the initial states have been stored by the IBC
    /// core module.
    VerifyCreation(VerifyCreationQuery),
    /// Verify the inclusion of a value under a key at a specific height.
    VerifyMembership(VerifyMembershipQuery),
    /// Verify that a key does not exist at a specific height.
    VerifyNonMembership(VerifyNonMembershipQuery),
    /// Update the light client with a header.
    UpdateState(UpdateStateQuery),
    /// Freeze the light client with a proof of misbehaviour.
    Misbehaviour(MisbehaviourQuery),
}

macro_rules! into_query_msg {
    ($($t:ty,)*) => {
        $(impl From<$t> for QueryMsg {
           fn from(value: $t) -> Self {
               QueryMsg::Verification(WhenNotPaused::wrap(value.into()))
           }
        })*
   };
}

into_query_msg!(
    VerifyCreationQuery,
    VerifyMembershipQuery,
    VerifyNonMembershipQuery,
    UpdateStateQuery,
    MisbehaviourQuery,
);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct VerifyCreationQuery {
    pub caller: String,
    pub client_id: ClientId,
    pub relayer: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct VerifyMembershipQuery {
    pub client_id: ClientId,
    pub height: u64,
    pub proof: Bytes,
    pub path: Bytes,
    pub value: Bytes,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct VerifyNonMembershipQuery {
    pub client_id: ClientId,
    pub height: u64,
    pub proof: Bytes,
    pub path: Bytes,
}

/// NOTE: Reads state through the `QueryStore`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UpdateStateQuery {
    pub caller: String,
    pub client_id: ClientId,
    pub relayer: String,
}

/// NOTE: Reads state through the `QueryStore`.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MisbehaviourQuery {
    pub caller: String,
    pub client_id: ClientId,
    pub relayer: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MisbehaviourResponse {
    /// The new client state to save after verifying the misbehaviour.
    pub client_state_bytes: Bytes,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UpdateStateResponse {
    /// The height to save the new consensus state at.
    pub height: u64,
    /// The client state to overwrite the current one with (if provided).
    ///
    /// This allows for updating to a height older than the current latest height, while not
    /// overwriting the latest state checkpoint.
    pub client_state_bytes: Option<Bytes>,
    /// The consensus state to save at the `update_height`.
    pub consensus_state_bytes: Bytes,
    /// The storage writes which will be written under the client's storage in the IBC core module.
    pub storage_writes: StorageWrites,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct VerifyCreationResponse {
    /// The chain ID of the counterparty chain this client tracks.
    pub counterparty_chain_id: String,
    /// The client state to save in storage as the initial client state. If this is not provided,
    /// the initial state as provided to [`MsgCreateClient`] will be saved as-is.
    pub client_state_bytes: Option<Bytes>,
    /// The storage writes which will be written under the client's storage in the IBC core module.
    pub storage_writes: StorageWrites,
    /// Events to be emitted by the IBC core contract.
    pub events: Vec<VerifyCreationResponseEvent>,
}

/// Arbitrary storage writes to be saved under the client's storage in the IBC core module. These
/// can then be accessed either through `IbcClientCtx::read_self_storage`, or directly via
/// `ClientStore<S>`.
pub type StorageWrites = BTreeMap<Bytes, Bytes>;

/// Events that can be returned by a light client after creation, to be emitted by the IBC core
/// module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum VerifyCreationResponseEvent {
    /// Lens client creation event. This signifies to off-chain parties that the client that was
    /// created does not directly verify the counterparty chain, but instead uses an intermediate
    /// client on another connected chain for some subset of the counterparty chain verification (as
    /// defined by the client type).
    CreateLensClient {
        /// The ID of the L1 client (the intermediary chain) on this chain. The chain ID of the
        /// intermediary chain can be found by querying the counterparty chain id of this client.
        l1_client_id: ClientId,
        /// The ID of the L2 client (the counterparty chain) on this chain that has just been
        /// created.
        l2_client_id: ClientId,
        /// The chain ID of the counterparty chain the newly created L2 client tracks.
        l2_chain_id: String,
    },
}
