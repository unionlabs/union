use serde::{Deserialize, Serialize};

use crate::EventAttribute;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "@type", deny_unknown_fields)]
pub enum Event {
    #[serde(rename = "/tm.Event")]
    TmEvent(TmEvent),
    #[serde(rename = "/tm.StorageDepositEvent")]
    TmStorageDepositEvent(TmStorageDepositEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TmEvent {
    #[serde(rename = "type")]
    pub ty: String,
    pub attrs: Option<Vec<EventAttribute>>,
    pub pkg_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TmStorageDepositEvent {
    pub bytes_delta: String,
    pub fee_delta: String,
    pub pkg_path: String,
}
