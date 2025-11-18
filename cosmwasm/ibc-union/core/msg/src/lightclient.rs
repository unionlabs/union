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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct InitMsg {
    pub ibc_host: String,
    pub access_managed_init_msg: access_managed::InitMsg,
}

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
    #[enumorph(ignore)]
    GetTimestamp { client_id: ClientId, height: u64 },
    #[enumorph(ignore)]
    GetLatestHeight { client_id: ClientId },
    #[enumorph(ignore)]
    GetStatus { client_id: ClientId },

    #[serde(untagged)]
    Verification(WhenNotPaused<VerificationQueryMsg>),

    #[serde(untagged)]
    AccessManaged(access_managed::QueryMsg),
    #[serde(untagged)]
    Pausable(PausableQuery),
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Enumorph)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum VerificationQueryMsg {
    VerifyCreation(VerifyCreationQuery),
    VerifyMembership(VerifyMembershipQuery),
    VerifyNonMembership(VerifyNonMembershipQuery),
    UpdateState(UpdateStateQuery),
    Misbehaviour(MisbehaviourQuery),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// NOTE: Reads state through the `QueryStore`.
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// NOTE: Reads state through the `QueryStore`.
pub struct UpdateStateQuery {
    pub caller: String,
    pub client_id: ClientId,
    pub relayer: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// TODO: Should read state through the `QueryStore`.
pub struct MisbehaviourQuery {
    pub caller: String,
    pub client_id: ClientId,
    pub message: Bytes,
    pub relayer: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct UpdateStateResponse {
    /// The height to save the consensus state at
    pub height: u64,
    /// The client state to overwrite the current one with if provided
    pub client_state_bytes: Option<Bytes>,
    /// The consensus state to save at the `update_height`
    pub consensus_state_bytes: Bytes,
    /// The storage writes which will be written under the client's storage in the core module
    pub storage_writes: StorageWrites,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct VerifyCreationResponse {
    pub counterparty_chain_id: String,
    pub client_state_bytes: Option<Bytes>,
    pub storage_writes: StorageWrites,
    pub events: Vec<VerifyCreationResponseEvent>,
}

pub type StorageWrites = BTreeMap<Bytes, Bytes>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub enum VerifyCreationResponseEvent {
    CreateLensClient {
        l1_client_id: ClientId,
        l2_client_id: ClientId,
        l2_chain_id: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct MisbehaviourResponse {
    pub client_state: Bytes,
}

macro_rules! into_query_msg {
    ($t:ty => |$value:ident| $expr:expr) => {
        impl From<$t> for QueryMsg {
            fn from($value: $t) -> Self {
                $expr
            }
        }
    };
}

into_query_msg!(VerifyCreationQuery      => |value| QueryMsg::Verification(WhenNotPaused::wrap(value.into())));
into_query_msg!(VerifyMembershipQuery    => |value| QueryMsg::Verification(WhenNotPaused::wrap(value.into())));
into_query_msg!(VerifyNonMembershipQuery => |value| QueryMsg::Verification(WhenNotPaused::wrap(value.into())));
into_query_msg!(UpdateStateQuery         => |value| QueryMsg::Verification(WhenNotPaused::wrap(value.into())));
into_query_msg!(MisbehaviourQuery        => |value| QueryMsg::Verification(WhenNotPaused::wrap(value.into())));
