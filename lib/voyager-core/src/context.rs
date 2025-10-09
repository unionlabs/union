use std::{collections::HashMap, path::PathBuf};

use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use voyager_plugin_protocol::WorkerClient;
use voyager_primitives::{ChainId, ClientType, ConsensusType, IbcInterface, IbcSpecId};
use voyager_rpc::{
    UNPROCESSABLE_JSONRPC_ERROR_CODE,
    types::{
        ClientBootstrapModuleInfo, ClientModuleInfo, FinalityModuleInfo, InfoResponse,
        ProofModuleInfo, StateModuleInfo,
    },
};
use voyager_vm::QueueError;

use crate::{equivalent_chain_ids::EquivalentChainIds, ibc_spec_handlers::IbcSpecHandlers};

pub struct Context {
    pub(crate) state_modules: HashMap<(ChainId, IbcSpecId), WorkerClient>,
    pub(crate) proof_modules: HashMap<(ChainId, IbcSpecId), WorkerClient>,

    /// map of chain id to consensus module.
    pub(crate) finality_modules: HashMap<ChainId, WorkerClient>,

    pub(crate) client_modules: HashMap<(ClientType, IbcInterface, IbcSpecId), WorkerClient>,

    pub(crate) client_bootstrap_modules: HashMap<(ChainId, ClientType), WorkerClient>,

    pub(crate) chain_consensus_types: HashMap<ChainId, ConsensusType>,

    pub(crate) client_consensus_types: HashMap<ClientType, ConsensusType>,

    pub(crate) plugins: HashMap<String, WorkerClient>,

    pub(crate) equivalent_chain_ids: EquivalentChainIds,

    // ibc version id => handler
    pub(crate) ibc_spec_handlers: IbcSpecHandlers,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PluginConfig {
    pub path: PathBuf,
    pub config: Value,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModulesConfig {
    pub state: Vec<ModuleConfig<StateModuleInfo>>,
    pub proof: Vec<ModuleConfig<ProofModuleInfo>>,
    pub consensus: Vec<ModuleConfig<FinalityModuleInfo>>,
    pub client: Vec<ModuleConfig<ClientModuleInfo>>,
    pub client_bootstrap: Vec<ModuleConfig<ClientBootstrapModuleInfo>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModuleConfig<T> {
    pub path: PathBuf,
    pub info: T,
    #[serde(default = "default_config")]
    pub config: Value,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_config() -> Value {
    Value::Object(serde_json::Map::new())
}

const fn default_enabled() -> bool {
    true
}

impl Context {
    pub fn info(&self) -> InfoResponse {
        let state = self
            .state_modules
            .keys()
            .cloned()
            .map(|(chain_id, ibc_spec_id)| StateModuleInfo {
                chain_id,
                ibc_spec_id,
            })
            .collect();

        let proof = self
            .proof_modules
            .keys()
            .cloned()
            .map(|(chain_id, ibc_spec_id)| ProofModuleInfo {
                chain_id,
                ibc_spec_id,
            })
            .collect();

        let consensus = self
            .finality_modules
            .keys()
            .cloned()
            .map(|chain_id| FinalityModuleInfo {
                consensus_type: self.chain_consensus_types[&chain_id].clone(),
                chain_id,
            })
            .collect();

        let client = self
            .client_modules
            .keys()
            .map(
                |(client_type, ibc_interface, ibc_spec_id)| ClientModuleInfo {
                    consensus_type: self.client_consensus_types[client_type].clone(),
                    client_type: client_type.clone(),
                    ibc_interface: ibc_interface.clone(),
                    ibc_spec_id: ibc_spec_id.clone(),
                },
            )
            .collect();

        let client_bootstrap = self
            .client_bootstrap_modules
            .keys()
            .map(|(chain_id, client_type)| ClientBootstrapModuleInfo {
                client_type: client_type.clone(),
                chain_id: chain_id.clone(),
            })
            .collect();

        InfoResponse {
            state,
            proof,
            consensus,
            client,
            client_bootstrap,
        }
    }

    pub fn plugin<'a>(&'a self, name: &str) -> Result<&'a WorkerClient, PluginNotFound> {
        self.plugins.get(name).ok_or_else(|| PluginNotFound {
            name: name.to_owned(),
        })
    }

    pub fn equivalent_chain_ids(&self) -> &EquivalentChainIds {
        &self.equivalent_chain_ids
    }

    pub fn chain_consensus_type<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
    ) -> Result<&'a ConsensusType, FinalityModuleNotFound> {
        self.chain_consensus_types
            .get(chain_id)
            .ok_or_else(|| FinalityModuleNotFound(chain_id.clone()))
    }

    pub fn client_consensus_type<'a, 'b, 'c: 'a>(
        &'a self,
        client_type: &ClientType,
    ) -> Result<&'a ConsensusType, ClientModuleNotFound> {
        self.client_consensus_types.get(client_type).ok_or_else(|| {
            ClientModuleNotFound::ClientTypeNotFound {
                client_type: client_type.clone(),
            }
        })
    }

    pub fn state_module<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
    ) -> Result<&'a WorkerClient, StateModuleNotFound> {
        self.state_modules
            .get(&(chain_id.clone(), ibc_spec_id.clone()))
            .ok_or_else(|| StateModuleNotFound {
                chain_id: chain_id.clone(),
                ibc_spec_id: ibc_spec_id.clone(),
            })
    }

    pub fn proof_module<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
        ibc_spec_id: &IbcSpecId,
    ) -> Result<&'a WorkerClient, ProofModuleNotFound> {
        self.proof_modules
            .get(&(chain_id.clone(), ibc_spec_id.clone()))
            .ok_or_else(|| ProofModuleNotFound {
                chain_id: chain_id.clone(),
                ibc_spec_id: ibc_spec_id.clone(),
            })
    }

    pub fn finality_module<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
    ) -> Result<&'a WorkerClient, FinalityModuleNotFound> {
        self.finality_modules
            .get(chain_id)
            .ok_or_else(|| FinalityModuleNotFound(chain_id.clone()))
    }

    pub fn client_module<'a, 'b, 'c: 'a>(
        &'a self,
        client_type: &ClientType,
        ibc_interface: &IbcInterface,
        ibc_spec_id: &IbcSpecId,
    ) -> Result<&'a WorkerClient, ClientModuleNotFound> {
        match self.client_modules.get(&(
            client_type.clone(),
            ibc_interface.clone(),
            ibc_spec_id.clone(),
        )) {
            Some(client_module) => Ok(client_module),
            None => Err(ClientModuleNotFound::NotFound {
                client_type: client_type.clone(),
                ibc_interface: ibc_interface.clone(),
                ibc_spec_id: ibc_spec_id.clone(),
            }),
        }
    }

    pub fn client_bootstrap_module<'a, 'b, 'c: 'a>(
        &'a self,
        chain_id: &ChainId,
        client_type: &ClientType,
    ) -> Result<&'a WorkerClient, ClientBootstrapModuleNotFound> {
        self.client_bootstrap_modules
            .get(&(chain_id.clone(), client_type.clone()))
            .ok_or_else(|| ClientBootstrapModuleNotFound {
                chain_id: chain_id.clone(),
                client_type: client_type.clone(),
            })
    }
}

macro_rules! module_error {
    ($Error:ident) => {
        impl From<$Error> for QueueError {
            fn from(value: $Error) -> Self {
                Self::Fatal(Box::new(value))
            }
        }

        impl From<$Error> for ErrorObjectOwned {
            fn from(value: $Error) -> Self {
                ErrorObject::owned(
                    UNPROCESSABLE_JSONRPC_ERROR_CODE,
                    value.to_string(),
                    None::<()>,
                )
            }
        }

        impl From<$Error> for jsonrpsee::core::client::Error {
            fn from(value: $Error) -> Self {
                ErrorObject::from(value).into()
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for state on chain `{chain_id}` and IBC version `{ibc_spec_id}`")]
pub struct StateModuleNotFound {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
}

module_error!(StateModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for proofs on chain `{chain_id}` and IBC version `{ibc_spec_id}`")]
pub struct ProofModuleNotFound {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
}

module_error!(ProofModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("no module loaded for finality on chain `{0}`")]
pub struct FinalityModuleNotFound(pub ChainId);

module_error!(FinalityModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error(
    "no module loaded for client bootstrapping on chain `{chain_id}` for client type `{client_type}`"
)]
pub struct ClientBootstrapModuleNotFound {
    pub chain_id: ChainId,
    pub client_type: ClientType,
}

module_error!(ClientBootstrapModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ClientModuleNotFound {
    #[error("no client module loaded for client type `{}`", client_type)]
    ClientTypeNotFound { client_type: ClientType },
    #[error(
        "no client module loaded supporting client type `{client_type}`, IBC interface `{ibc_interface}`, and IBC version `{ibc_spec_id}`"
    )]
    NotFound {
        client_type: ClientType,
        ibc_interface: IbcInterface,
        ibc_spec_id: IbcSpecId,
    },
}

module_error!(ClientModuleNotFound);

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("plugin `{name}` not found")]
pub struct PluginNotFound {
    pub name: String,
}

module_error!(PluginNotFound);
