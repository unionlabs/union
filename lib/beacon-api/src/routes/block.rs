use beacon_api_types::{bellatrix, capella, deneb, electra, phase0};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use unionlabs::{ErrorReporter, primitives::H256};

use crate::client::{VersionedResponse, VersionedResponseTypes};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "BeaconBlockResponseRaw")]
pub struct BeaconBlockResponse {
    #[serde(flatten)]
    pub response: VersionedResponse<BeaconBlockResponseTypes>,
    pub execution_optimistic: Option<bool>,
    pub finalized: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeaconBlockData<H> {
    pub root: H256,
    pub canonical: bool,
    pub header: H,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeaconBlockResponseTypes {}

impl VersionedResponseTypes for BeaconBlockResponseTypes {
    type Phase0 = phase0::SignedBeaconBlock;
    type Altair = phase0::SignedBeaconBlock;
    type Bellatrix = bellatrix::SignedBeaconBlock;
    type Capella = capella::SignedBeaconBlock;
    type Deneb = deneb::SignedBeaconBlock;
    type Electra = electra::SignedBeaconBlock;
}

#[derive(Deserialize)]
pub struct BeaconBlockResponseRaw<'a> {
    // TODO: Make not optional
    version: Option<String>,
    #[serde(borrow)]
    data: &'a RawValue,
    execution_optimistic: Option<bool>,
    finalized: Option<bool>,
}

impl<'a> TryFrom<BeaconBlockResponseRaw<'a>> for BeaconBlockResponse {
    type Error = String;

    fn try_from(value: BeaconBlockResponseRaw<'a>) -> std::result::Result<Self, Self::Error> {
        Ok(match value.version.as_deref() {
            Some("phase0") => Self {
                execution_optimistic: value.execution_optimistic,
                finalized: value.finalized,
                response: VersionedResponse::Phase0(
                    serde_json::from_str(value.data.get())
                        .map_err(|e| ErrorReporter(e).to_string())?,
                ),
            },
            Some("altair") => Self {
                execution_optimistic: value.execution_optimistic,
                finalized: value.finalized,
                response: VersionedResponse::Altair(
                    serde_json::from_str(value.data.get())
                        .map_err(|e| ErrorReporter(e).to_string())?,
                ),
            },
            Some("bellatrix") => Self {
                execution_optimistic: value.execution_optimistic,
                finalized: value.finalized,
                response: VersionedResponse::Bellatrix(
                    serde_json::from_str(value.data.get())
                        .map_err(|e| ErrorReporter(e).to_string())?,
                ),
            },
            Some("capella") => Self {
                execution_optimistic: value.execution_optimistic,
                finalized: value.finalized,
                response: VersionedResponse::Capella(
                    serde_json::from_str(value.data.get())
                        .map_err(|e| ErrorReporter(e).to_string())?,
                ),
            },
            Some("deneb") => Self {
                execution_optimistic: value.execution_optimistic,
                finalized: value.finalized,
                response: VersionedResponse::Deneb(
                    serde_json::from_str(value.data.get())
                        .map_err(|e| ErrorReporter(e).to_string())?,
                ),
            },
            Some("electra") | None => Self {
                execution_optimistic: value.execution_optimistic,
                finalized: value.finalized,
                response: VersionedResponse::Electra(
                    serde_json::from_str(value.data.get())
                        .map_err(|e| ErrorReporter(e).to_string())?,
                ),
            },
            v => return Err(format!("unknown version {v:?}")),
        })
    }
}
