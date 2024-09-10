use std::collections::VecDeque;

use ark_serialize::{CanonicalSerialize, SerializationError, Valid};
use clap::Subcommand;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
};
use macros::model;
use queue_msg::{BoxDynError, Op};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, instrument};
use unionlabs::{
    self,
    encoding::{Bcs, DecodeAs, EncodeAs, EthAbi},
    ibc::lightclients::cometbls::{
        client_state::ClientState, consensus_state::ConsensusState, header::Header,
    },
    ErrorReporter,
};
use voyager_message::{
    data::Data,
    plugin::{
        ClientModuleServer, ClientStateMeta, ConsensusStateMeta, PluginInfo, PluginKind,
        PluginModuleServer, SupportedInterface,
    },
    run_module_server, ChainId, ClientType, IbcInterface, VoyagerMessage, FATAL_JSONRPC_ERROR_CODE,
};

use crate::{call::ModuleCall, callback::ModuleCallback, data::ModuleData};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server(
        Module::new,
        ClientModuleServer::into_rpc,
        |config, cmd| async move { Module::new(config, String::new()).await?.cmd(cmd).await },
    )
    .await
}

#[model(no_serde)]
pub enum SupportedIbcInterfaces {
    IbcSolidity,
    IbcMoveAptos,
}

impl SupportedIbcInterfaces {
    pub fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcInterfaces::IbcSolidity => IbcInterface::IBC_SOLIDITY,
            SupportedIbcInterfaces::IbcMoveAptos => IbcInterface::IBC_MOVE_APTOS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub ibc_interface: SupportedIbcInterfaces,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ibc_interface: IbcInterface<'static>,
}

#[derive(Subcommand)]
pub enum Cmd {
    Test,
}

impl Module {
    async fn cmd(&self, cmd: Cmd) -> Result<(), BoxDynError> {
        match cmd {
            Cmd::Test => {
                println!("test");

                Ok(())
            }
        }
    }

    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        format!("{PLUGIN_NAME}/{}", self.ibc_interface.as_str())
    }

    pub async fn new(config: Config, _voyager_socket: String) -> Result<Self, BoxDynError> {
        Ok(Self {
            ibc_interface: match config.ibc_interface.as_str() {
                IbcInterface::IBC_SOLIDITY => SupportedIbcInterfaces::IbcSolidity,
                IbcInterface::IBC_MOVE_APTOS => SupportedIbcInterfaces::IbcMoveAptos,
                i => return Err(format!("unsupported IBC interface {i}").into()),
            },
        })
    }

    pub fn decode_consensus_state(&self, consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        match self.ibc_interface {
            SupportedIbcInterfaces::IbcSolidity => {
                ConsensusState::decode_as::<EthAbi>(consensus_state).map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode consensus state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
            }
            SupportedIbcInterfaces::IbcMoveAptos => {
                ConsensusState::decode_as::<Bcs>(consensus_state).map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode consensus state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
            }
        }
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterfaces::IbcSolidity => ClientState::decode_as::<EthAbi>(client_state)
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                }),
            SupportedIbcInterfaces::IbcMoveAptos => ClientState::decode_as::<Bcs>(client_state)
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                }),
        }
    }
}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all)]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: Some(PluginKind::Client),
            interest_filter: None,
        })
    }

    #[instrument(skip_all)]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {}
    }

    #[instrument(skip_all)]
    async fn callback(
        &self,
        cb: ModuleCallback,
        _data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }
}

#[async_trait]
impl ClientModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument(skip_all)]
    async fn supported_interface(&self) -> RpcResult<SupportedInterface> {
        Ok(SupportedInterface {
            client_type: ClientType::new(ClientType::COMETBLS),
            ibc_interface: IbcInterface::new(self.ibc_interface.as_str()),
        })
    }

    #[instrument(skip_all)]
    async fn decode_client_state_meta(
        &self,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<ClientStateMeta> {
        let cs = self.decode_client_state(&client_state.0)?;

        Ok(ClientStateMeta {
            chain_id: ChainId::new(cs.chain_id),
            height: cs.latest_height,
        })
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state_meta(
        &self,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = self.decode_consensus_state(&consensus_state.0)?;

        Ok(ConsensusStateMeta {
            timestamp_nanos: cs.timestamp,
        })
    }

    #[instrument(skip_all)]
    async fn decode_client_state(&self, client_state: Hex<Vec<u8>>) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_client_state(&client_state.0)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state(&self, consensus_state: Hex<Vec<u8>>) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_consensus_state(&consensus_state.0)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn encode_client_state(
        &self,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
        if !metadata.is_null() {
            return Err(ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                "metadata was provided, but this client type does not require metadata for client \
                state encoding",
                Some(json!({
                    "provided_metadata": metadata,
                })),
            ));
        }

        serde_json::from_value::<ClientState>(client_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize client state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterfaces::IbcSolidity => cs.encode_as::<EthAbi>(),
                SupportedIbcInterfaces::IbcMoveAptos => cs.encode_as::<Bcs>(),
            })
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn encode_consensus_state(&self, consensus_state: Value) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<ConsensusState>(consensus_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!(
                        "unable to deserialize consensus state: {}",
                        ErrorReporter(err)
                    ),
                    None::<()>,
                )
            })
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterfaces::IbcSolidity => cs.encode_as::<EthAbi>(),
                SupportedIbcInterfaces::IbcMoveAptos => cs.encode_as::<Bcs>(),
            })
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_client_state(
        &self,
        client_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        Ok(client_state)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_consensus_state(
        &self,
        consensus_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        Ok(consensus_state)
    }

    #[instrument(skip_all)]
    async fn encode_header(&self, header: Value) -> RpcResult<Hex<Vec<u8>>> {
        serde_json::from_value::<Header>(header)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize header: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|mut header| match self.ibc_interface {
                SupportedIbcInterfaces::IbcSolidity => Ok(header.encode_as::<EthAbi>()),
                SupportedIbcInterfaces::IbcMoveAptos => {
                    header.zero_knowledge_proof =
                        reencode_zkp_for_move(&header.zero_knowledge_proof).map_err(|e| {
                            ErrorObject::owned(
                                FATAL_JSONRPC_ERROR_CODE,
                                format!("unable to decode zkp: {}", e),
                                None::<()>,
                            )
                        })?;
                    Ok(header.encode_as::<Bcs>())
                }
            })?
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn encode_proof(&self, proof: Value) -> RpcResult<Hex<Vec<u8>>> {
        debug!(%proof, "encoding proof");

        serde_json::from_value::<unionlabs::ibc::core::commitment::merkle_proof::MerkleProof>(proof)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize proof: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterfaces::IbcSolidity => {
                    unionlabs::union::ics23::merkle_proof::MerkleProof::try_from(
                        protos::ibc::core::commitment::v1::MerkleProof::from(cs),
                    )
                    .unwrap()
                    .encode_as::<EthAbi>()
                }
                SupportedIbcInterfaces::IbcMoveAptos => {
                    // TODO: Currently disabled, test this later
                    unionlabs::union::ics23::merkle_proof::MerkleProof::try_from(
                        protos::ibc::core::commitment::v1::MerkleProof::from(cs),
                    )
                    .unwrap()
                    .encode_as::<Bcs>()
                }
            })
            .map(Hex)
    }
}

fn reencode_zkp_for_move(zkp: &[u8]) -> Result<Vec<u8>, SerializationError> {
    let mut buf = Vec::new();

    let serialize_g1 =
        |cursor: &mut usize, buf: &mut Vec<u8>, zkp: &[u8]| -> Result<(), SerializationError> {
            let proof = ark_bn254::G1Affine::new_unchecked(
                ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
                    &zkp[*cursor..*cursor + 32],
                )),
                ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
                    &zkp[*cursor + 32..*cursor + 64],
                )),
            );
            proof.check()?;
            *cursor += 64;
            proof.serialize_compressed(buf)?;
            Ok(())
        };

    let serialize_g2 =
        |cursor: &mut usize, buf: &mut Vec<u8>, zkp: &[u8]| -> Result<(), SerializationError> {
            let proof = ark_bn254::G2Affine::new_unchecked(
                ark_bn254::Fq2::new(
                    ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
                        &zkp[*cursor + 32..*cursor + 64],
                    )),
                    ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
                        &zkp[*cursor..*cursor + 32],
                    )),
                ),
                ark_bn254::Fq2::new(
                    ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
                        &zkp[*cursor + 96..*cursor + 128],
                    )),
                    ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
                        &zkp[*cursor + 64..*cursor + 96],
                    )),
                ),
            );
            proof.check()?;
            *cursor += 128;
            proof.serialize_compressed(buf)?;
            Ok(())
        };

    let mut cursor = 0;
    // zkp.proof.a
    serialize_g1(&mut cursor, &mut buf, zkp)?;
    // zkp.proof.b
    serialize_g2(&mut cursor, &mut buf, zkp)?;
    // zkp.proof.c
    serialize_g1(&mut cursor, &mut buf, zkp)?;
    // zkp.poc
    serialize_g1(&mut cursor, &mut buf, zkp)?;
    // zkp.pok
    serialize_g1(&mut cursor, &mut buf, zkp)?;

    Ok(buf)
}
