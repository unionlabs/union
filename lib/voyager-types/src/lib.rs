use core::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use unionlabs::ibc::core::client::height::Height;
use voyager_primitives::IbcSpec;

/// Simple wrapper around a [`Value`] for raw client ids.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct RawClientId(Value);

impl From<String> for RawClientId {
    fn from(value: String) -> Self {
        // attempt to parse the string as json, if that fails just treat the whole string as a json
        // string value
        RawClientId(
            value
                .parse::<Value>()
                .unwrap_or_else(|_| Value::String(value.to_owned())),
        )
    }
}

impl fmt::Display for RawClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RawClientId {
    pub fn new(t: impl Serialize) -> Self {
        Self(serde_json::to_value(t).unwrap())
    }

    pub fn decode_spec<V: IbcSpec>(self) -> Result<V::ClientId, serde_json::Error> {
        serde_json::from_value(self.0)
    }

    pub fn as_raw(&self) -> &Value {
        &self.0
    }

    pub fn into_raw(self) -> Value {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct IbcProof {
    pub proof_type: ProofType,
    /// The height that the proof was read at.
    pub height: Height,
    pub proof: Value,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProofType {
    Membership,
    NonMembership,
}
