use jsonrpsee::{core::RpcResult, types::ErrorObject};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use unionlabs::ibc::core::client::height::Height;
use voyager_primitives::{ChainId, ClientType, ConsensusType, IbcInterface, IbcSpecId};
use voyager_types::IbcProof;

use crate::MISSING_STATE_ERROR_CODE;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InfoResponse {
    pub state: Vec<StateModuleInfo>,
    pub proof: Vec<ProofModuleInfo>,
    pub consensus: Vec<FinalityModuleInfo>,
    pub client: Vec<ClientModuleInfo>,
    pub client_bootstrap: Vec<ClientBootstrapModuleInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StateModuleInfo {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
}

impl StateModuleInfo {
    pub fn id(&self) -> String {
        format!("state/{}/{}", self.ibc_spec_id, self.chain_id)
    }

    // TODO: Add this for ibc_spec_id
    pub fn ensure_chain_id(&self, chain_id: impl AsRef<str>) -> Result<(), UnexpectedChainIdError> {
        if chain_id.as_ref() != self.chain_id.as_str() {
            Err(UnexpectedChainIdError {
                expected: self.chain_id.clone(),
                found: chain_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    // TODO: Add this for ibc_spec_id
    pub fn ensure_ibc_spec_id(
        &self,
        ibc_spec_id: impl AsRef<str>,
    ) -> Result<(), UnexpectedIbcSpecIdError> {
        if ibc_spec_id.as_ref() != self.ibc_spec_id.as_str() {
            Err(UnexpectedIbcSpecIdError {
                expected: self.ibc_spec_id.clone(),
                found: ibc_spec_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ProofModuleInfo {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
}

impl ProofModuleInfo {
    pub fn id(&self) -> String {
        format!("proof/{}/{}", self.ibc_spec_id, self.chain_id)
    }

    // TODO: Add this for ibc_spec_id
    pub fn ensure_chain_id(&self, chain_id: impl AsRef<str>) -> Result<(), UnexpectedChainIdError> {
        if chain_id.as_ref() != self.chain_id.as_str() {
            Err(UnexpectedChainIdError {
                expected: self.chain_id.clone(),
                found: chain_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FinalityModuleInfo {
    pub chain_id: ChainId,
    pub consensus_type: ConsensusType,
    // REVIEW: Maybe we need this? Do different client types for a single consensus necessarily
    // have the same client and consensus state types? /// The type of client this consensus
    // module provides state for. #[arg(value_parser(|s: &str|
    // ok(ClientType::new(s.to_owned()))))] pub client_type: ClientType,
}

impl FinalityModuleInfo {
    pub fn id(&self) -> String {
        format!("consensus/{}/{}", self.chain_id, self.consensus_type)
    }

    pub fn ensure_chain_id(&self, chain_id: impl AsRef<str>) -> Result<(), UnexpectedChainIdError> {
        if chain_id.as_ref() != self.chain_id.as_str() {
            Err(UnexpectedChainIdError {
                expected: self.chain_id.clone(),
                found: chain_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    pub fn ensure_consensus_type(
        &self,
        consensus_type: impl AsRef<str>,
    ) -> Result<(), UnexpectedConsensusTypeError> {
        if consensus_type.as_ref() != self.consensus_type.as_str() {
            Err(UnexpectedConsensusTypeError {
                expected: self.consensus_type.clone(),
                found: consensus_type.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClientModuleInfo {
    /// The client type that this client module provides functionality for.
    pub client_type: ClientType,

    /// The consensus type that this client module verifies.
    pub consensus_type: ConsensusType,

    /// The IBC interface that this client module provides functionality for.
    pub ibc_interface: IbcInterface,

    /// The IBC version that this client module provides functionality for.
    pub ibc_spec_id: IbcSpecId,
}

impl ClientModuleInfo {
    pub fn id(&self) -> String {
        format!(
            "client/{}/{}/{}/{}",
            self.client_type, self.consensus_type, self.ibc_interface, self.ibc_spec_id
        )
    }

    pub fn ensure_client_type(
        &self,
        client_type: impl AsRef<str>,
    ) -> Result<(), UnexpectedClientTypeError> {
        if client_type.as_ref() != self.client_type.as_str() {
            Err(UnexpectedClientTypeError {
                expected: self.client_type.clone(),
                found: client_type.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    pub fn ensure_consensus_type(
        &self,
        consensus_type: impl AsRef<str>,
    ) -> Result<(), UnexpectedConsensusTypeError> {
        if consensus_type.as_ref() != self.consensus_type.as_str() {
            Err(UnexpectedConsensusTypeError {
                expected: self.consensus_type.clone(),
                found: consensus_type.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    pub fn ensure_ibc_interface(
        &self,
        ibc_interface: impl AsRef<str>,
    ) -> Result<(), UnexpectedIbcInterfaceError> {
        if ibc_interface.as_ref() != self.ibc_interface.as_str() {
            Err(UnexpectedIbcInterfaceError {
                expected: self.ibc_interface.clone(),
                found: ibc_interface.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    pub fn ensure_ibc_spec_id(
        &self,
        ibc_spec_id: impl AsRef<str>,
    ) -> Result<(), UnexpectedIbcSpecIdError> {
        if ibc_spec_id.as_ref() != self.ibc_spec_id.as_str() {
            Err(UnexpectedIbcSpecIdError {
                expected: self.ibc_spec_id.clone(),
                found: ibc_spec_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ClientBootstrapModuleInfo {
    /// The client type that this client bootstrap module provides functionality for.
    pub client_type: ClientType,

    /// The id of the chain that this client bootstrap module provides initial state for.
    pub chain_id: ChainId,
}

impl ClientBootstrapModuleInfo {
    pub fn id(&self) -> String {
        format!("client-bootstrap/{}/{}", self.client_type, self.chain_id)
    }

    pub fn ensure_client_type(
        &self,
        client_type: impl AsRef<str>,
    ) -> Result<(), UnexpectedClientTypeError> {
        if client_type.as_ref() != self.client_type.as_str() {
            Err(UnexpectedClientTypeError {
                expected: self.client_type.clone(),
                found: client_type.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }

    // TODO: Add this for ibc_spec_id
    pub fn ensure_chain_id(&self, chain_id: impl AsRef<str>) -> Result<(), UnexpectedChainIdError> {
        if chain_id.as_ref() != self.chain_id.as_str() {
            Err(UnexpectedChainIdError {
                expected: self.chain_id.clone(),
                found: chain_id.as_ref().to_owned(),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("invalid chain id: expected `{expected}` but the rpc responded with `{found}`")]
pub struct UnexpectedChainIdError {
    pub expected: ChainId,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error(
    "invalid consensus type: this module provides functionality for consensus type `{expected}`, but the config specifies `{found}`"
)]
pub struct UnexpectedConsensusTypeError {
    pub expected: ConsensusType,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error(
    "invalid client type: this module provides functionality for client type `{expected}`, but the config specifies `{found}`"
)]
pub struct UnexpectedClientTypeError {
    pub expected: ClientType,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error(
    "invalid IBC interface: this module provides functionality for IBC interface `{expected}`, but the config specifies `{found}`"
)]
pub struct UnexpectedIbcInterfaceError {
    pub expected: IbcInterface,
    pub found: String,
}

#[derive(Debug, Clone, thiserror::Error)]
#[error(
    "invalid IBC spec: this module provides functionality for IBC spec `{expected}`, but the config specifies `{found}`"
)]
pub struct UnexpectedIbcSpecIdError {
    pub expected: IbcSpecId,
    pub found: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PluginInfo {
    /// The name of this plugin. Any plugin messages with this name will be
    /// routed to this plugin.
    pub name: String,
    /// A jaq filter to run on every message before pushing them to the queue.
    /// This ***MUST*** return a bool. If this returns `true`, the message will
    /// be pushed to the optimization queue with this plugin's name as the tag,
    /// otherwise it will be passed on to the next plugin to be filtered.
    pub interest_filter: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct IbcStateResponse<State> {
    /// The height that the state was read at.
    pub height: Height,
    /// The state from the chain, as read at `height`.
    ///
    /// If the state does not exist on chain at `height`, this will be `None`.
    pub state: Option<State>,
}

impl IbcStateResponse<Value> {
    // pub fn decode_state<T: DeserializeOwned>(&self) -> RpcResult<T> {
    //     serde_json::from_value(self.state.clone()).map_err(|e| {
    //         ErrorObject::owned(
    //             FATAL_JSONRPC_ERROR_CODE,
    //             format!("error decoding IBC state: {}", ErrorReporter(e)),
    //             Some(json!({
    //                 "raw_state": self.state
    //             })),
    //         )
    //     })
    // }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SelfClientStateResponse {
    pub height: Height,
    pub state: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SelfConsensusStateResponse {
    pub height: Height,
    pub state: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum IbcProofResponse {
    Proof(IbcProof),
    /// Unable to query a proof for this state at the requested height. Try a more recent height.
    NotAvailable,
}

impl IbcProofResponse {
    /// Convert this proof response to a `Result`, returning an error if `self` is [`IbcProofResponse::NotAvailable`].
    pub fn into_result(self) -> RpcResult<IbcProof> {
        match self {
            Self::Proof(proof) => Ok(proof),
            Self::NotAvailable => Err(ErrorObject::owned(
                MISSING_STATE_ERROR_CODE,
                "proof not available",
                None::<()>,
            )),
        }
    }
}
