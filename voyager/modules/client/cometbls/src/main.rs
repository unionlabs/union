use alloy_sol_types::SolValue;
use ark_serialize::{CanonicalSerialize, SerializationError, Valid};
use cometbls_light_client_types::{ClientState, ConsensusState, Header};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use macros::model;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{debug, instrument};
use unionlabs::{
    self,
    encoding::{Bcs, Bincode, DecodeAs, EncodeAs, EthAbi, Proto},
    google::protobuf::any::Any,
    ibc::lightclients::wasm,
    primitives::Bytes,
    union::ics23,
    ErrorReporter,
};
use voyager_message::{
    module::{ClientModuleInfo, ClientModuleServer},
    primitives::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType,
        IbcGo08WasmClientMetadata, IbcInterface,
    },
    ClientModule, FATAL_JSONRPC_ERROR_CODE,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SupportedIbcInterface {
    IbcSolidity,
    IbcMoveAptos,
    IbcGoV8_08Wasm,
    IbcCosmwasm,
}

impl TryFrom<String> for SupportedIbcInterface {
    // TODO: Better error type here
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            IbcInterface::IBC_SOLIDITY => Ok(SupportedIbcInterface::IbcSolidity),
            IbcInterface::IBC_MOVE_APTOS => Ok(SupportedIbcInterface::IbcMoveAptos),
            IbcInterface::IBC_GO_V8_08_WASM => Ok(SupportedIbcInterface::IbcGoV8_08Wasm),
            IbcInterface::IBC_COSMWASM => Ok(SupportedIbcInterface::IbcCosmwasm),
            _ => Err(format!("unsupported IBC interface: `{value}`")),
        }
    }
}

impl SupportedIbcInterface {
    fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcInterface::IbcSolidity => IbcInterface::IBC_SOLIDITY,
            SupportedIbcInterface::IbcMoveAptos => IbcInterface::IBC_MOVE_APTOS,
            SupportedIbcInterface::IbcGoV8_08Wasm => IbcInterface::IBC_GO_V8_08_WASM,
            SupportedIbcInterface::IbcCosmwasm => IbcInterface::IBC_COSMWASM,
        }
    }
}

impl From<SupportedIbcInterface> for String {
    fn from(value: SupportedIbcInterface) -> Self {
        value.as_str().to_owned()
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub ibc_interface: SupportedIbcInterface,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {}

impl ClientModule for Module {
    type Config = Config;

    async fn new(Config {}: Self::Config, info: ClientModuleInfo) -> Result<Self, BoxDynError> {
        info.ensure_client_type(ClientType::COMETBLS_GROTH16)?;
        info.ensure_consensus_type(ConsensusType::COMETBLS)?;

        Ok(Self {
            ibc_interface: SupportedIbcInterface::try_from(info.ibc_interface.to_string())?,
        })
    }
}

impl Module {
    pub fn decode_consensus_state(&self, consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity
            | SupportedIbcInterface::IbcMoveAptos
            | SupportedIbcInterface::IbcCosmwasm => {
                ConsensusState::decode_as::<EthAbi>(consensus_state).map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode consensus state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
            }
            SupportedIbcInterface::IbcGoV8_08Wasm => {
                <Any<wasm::consensus_state::ConsensusState<ConsensusState>>>::decode_as::<Proto>(
                    consensus_state,
                )
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode consensus state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
                .map(|any| any.0.data)
            }
        }
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => ClientState::decode_as::<EthAbi>(client_state)
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                }),
            SupportedIbcInterface::IbcMoveAptos => ClientState::decode_as::<Bcs>(client_state)
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                }),
            SupportedIbcInterface::IbcGoV8_08Wasm => {
                <Any<wasm::client_state::ClientState<ClientState>>>::decode_as::<Proto>(
                    client_state,
                )
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
                .map(|any| any.0.data)
            }
            SupportedIbcInterface::IbcCosmwasm => ClientState::decode_as::<Bincode>(client_state)
                .map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {err}"),
                        None::<()>,
                    )
                }),
        }
    }
}

#[async_trait]
impl ClientModuleServer for Module {
    #[instrument(skip_all)]
    async fn decode_client_state_meta(
        &self,
        _: &Extensions,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        let cs = self.decode_client_state(&client_state)?;

        Ok(ClientStateMeta {
            counterparty_chain_id: ChainId::new(cs.chain_id.as_str().to_owned()),
            counterparty_height: cs.latest_height,
        })
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state_meta(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = self.decode_consensus_state(&consensus_state)?;

        Ok(ConsensusStateMeta {
            timestamp: cs.timestamp,
        })
    }

    #[instrument(skip_all)]
    async fn decode_client_state(&self, _: &Extensions, client_state: Bytes) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_client_state(&client_state)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_consensus_state(&consensus_state)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn encode_client_state(
        &self,
        _: &Extensions,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Bytes> {
        serde_json::from_value::<ClientState>(client_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize client state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .and_then(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => {
                    if !metadata.is_null() {
                        return Err(ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            "metadata was provided, but this client type does not require \
                            metadata for client state encoding",
                            Some(json!({
                                "provided_metadata": metadata,
                            })),
                        ));
                    }

                    Ok(cs.encode_as::<EthAbi>())
                }
                SupportedIbcInterface::IbcMoveAptos => {
                    if !metadata.is_null() {
                        return Err(ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            "metadata was provided, but this client type does not require \
                            metadata for client state encoding",
                            Some(json!({
                                "provided_metadata": metadata,
                            })),
                        ));
                    }

                    Ok(cs.encode_as::<Bcs>())
                }
                SupportedIbcInterface::IbcCosmwasm => {
                    if !metadata.is_null() {
                        return Err(ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            "metadata was provided, but this client type does not require \
                            metadata for client state encoding",
                            Some(json!({
                                "provided_metadata": metadata,
                            })),
                        ));
                    }
                    Ok(cs.encode_as::<Bincode>())
                }
                SupportedIbcInterface::IbcGoV8_08Wasm => {
                    let metadata =
                        serde_json::from_value::<IbcGo08WasmClientMetadata>(metadata.clone())
                            .map_err(|e| {
                                ErrorObject::owned(
                                    FATAL_JSONRPC_ERROR_CODE,
                                    format!("unable to decode metadata: {}", ErrorReporter(e)),
                                    Some(json!({
                                        "provided_metadata": metadata,
                                    })),
                                )
                            })?;

                    Ok(Any(wasm::client_state::ClientState {
                        latest_height: cs.latest_height,
                        data: cs,
                        checksum: metadata.checksum,
                    })
                    .encode_as::<Proto>())
                }
            })
            .map(Into::into)
    }

    #[instrument(skip_all)]
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
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
                SupportedIbcInterface::IbcSolidity
                | SupportedIbcInterface::IbcMoveAptos
                | SupportedIbcInterface::IbcCosmwasm => cs.encode_as::<EthAbi>(),
                SupportedIbcInterface::IbcGoV8_08Wasm => {
                    Any(wasm::consensus_state::ConsensusState { data: cs }).encode_as::<Proto>()
                }
            })
            .map(Into::into)
    }

    #[instrument(skip_all)]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<Header>(header)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize header: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|mut header| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => Ok(header.encode_as::<EthAbi>()),
                SupportedIbcInterface::IbcCosmwasm => Ok(header.encode_as::<Bincode>()),
                SupportedIbcInterface::IbcMoveAptos => {
                    header.zero_knowledge_proof =
                        reencode_zkp_for_move(&header.zero_knowledge_proof)
                            .map_err(|e| {
                                ErrorObject::owned(
                                    FATAL_JSONRPC_ERROR_CODE,
                                    format!("unable to decode zkp: {}", e),
                                    None::<()>,
                                )
                            })?
                            .into();
                    Ok(header.encode_as::<Bcs>())
                }
                SupportedIbcInterface::IbcGoV8_08Wasm => {
                    Ok(Any(wasm::client_message::ClientMessage { data: header })
                        .encode_as::<Proto>())
                }
            })?
            .map(Into::into)
    }

    #[instrument(skip_all)]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        debug!(%proof, "encoding proof");

        serde_json::from_value::<unionlabs::ibc::core::commitment::merkle_proof::MerkleProof>(proof)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize proof: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|proof| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => encode_merkle_proof_for_evm(proof),
                SupportedIbcInterface::IbcCosmwasm => proof.encode_as::<Bincode>(),
                SupportedIbcInterface::IbcMoveAptos => encode_merkle_proof_for_move(
                    ics23::merkle_proof::MerkleProof::try_from(
                        protos::ibc::core::commitment::v1::MerkleProof::from(proof),
                    )
                    .unwrap(),
                ),
                SupportedIbcInterface::IbcGoV8_08Wasm => proof.encode_as::<Proto>(),
            })
            .map(Into::into)
    }
}

fn encode_merkle_proof_for_evm(
    proof: unionlabs::ibc::core::commitment::merkle_proof::MerkleProof,
) -> Vec<u8> {
    alloy_sol_types::sol! {
        struct ExistenceProof {
            bytes key;
            bytes value;
            bytes leafPrefix;
            InnerOp[] path;
        }

        struct NonExistenceProof {
            bytes key;
            ExistenceProof left;
            ExistenceProof right;
        }

        struct InnerOp {
            bytes prefix;
            bytes suffix;
        }

        struct ProofSpec {
            uint256 childSize;
            uint256 minPrefixLength;
            uint256 maxPrefixLength;
        }
    }

    let merkle_proof = ics23::merkle_proof::MerkleProof::try_from(
        protos::ibc::core::commitment::v1::MerkleProof::from(proof),
    )
    .unwrap();

    let convert_inner_op = |i: unionlabs::union::ics23::inner_op::InnerOp| InnerOp {
        prefix: i.prefix.into(),
        suffix: i.suffix.into(),
    };

    let convert_existence_proof =
        |e: unionlabs::union::ics23::existence_proof::ExistenceProof| ExistenceProof {
            key: e.key.into(),
            value: e.value.into(),
            leafPrefix: e.leaf_prefix.into(),
            path: e.path.into_iter().map(convert_inner_op).collect(),
        };

    let exist_default = || ics23::existence_proof::ExistenceProof {
        key: vec![].into(),
        value: vec![].into(),
        leaf_prefix: vec![].into(),
        path: vec![],
    };

    match merkle_proof {
        ics23::merkle_proof::MerkleProof::Membership(a, b) => {
            (convert_existence_proof(a), convert_existence_proof(b)).abi_encode_params()
        }
        ics23::merkle_proof::MerkleProof::NonMembership(a, b) => (
            NonExistenceProof {
                key: a.key.into(),
                left: convert_existence_proof(a.left.unwrap_or_else(exist_default)),
                right: convert_existence_proof(a.right.unwrap_or_else(exist_default)),
            },
            convert_existence_proof(b),
        )
            .abi_encode_params(),
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

#[model]
struct MoveMembershipProof {
    sub_proof: ics23::existence_proof::ExistenceProof,
    top_level_proof: ics23::existence_proof::ExistenceProof,
}

fn encode_merkle_proof_for_move(proof: ics23::merkle_proof::MerkleProof) -> Vec<u8> {
    match proof {
        ics23::merkle_proof::MerkleProof::Membership(sub_proof, top_level_proof) => {
            MoveMembershipProof {
                sub_proof,
                top_level_proof,
            }
        }
        ics23::merkle_proof::MerkleProof::NonMembership(_, _) => todo!(),
    }
    .encode_as::<Bcs>()
}
