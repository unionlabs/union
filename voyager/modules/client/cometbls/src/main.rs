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
    bytes::Bytes,
    encoding::{Bcs, DecodeAs, EncodeAs, EthAbi, Proto},
    google::protobuf::any::Any,
    ibc::lightclients::wasm,
    union::ics23,
    ErrorReporter,
};
use voyager_message::{
    core::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType,
        IbcGo08WasmClientMetadata, IbcInterface,
    },
    module::{ClientModuleInfo, ClientModuleServer},
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
}

impl TryFrom<String> for SupportedIbcInterface {
    // TODO: Better error type here
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            IbcInterface::IBC_SOLIDITY => Ok(SupportedIbcInterface::IbcSolidity),
            IbcInterface::IBC_MOVE_APTOS => Ok(SupportedIbcInterface::IbcMoveAptos),
            IbcInterface::IBC_GO_V8_08_WASM => Ok(SupportedIbcInterface::IbcGoV8_08Wasm),
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
            SupportedIbcInterface::IbcSolidity => {
                ConsensusState::decode_as::<EthAbi>(consensus_state).map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode consensus state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
            }
            SupportedIbcInterface::IbcMoveAptos => {
                ConsensusState::decode_as::<Bcs>(consensus_state).map_err(|err| {
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
            chain_id: ChainId::new(cs.chain_id.as_str().to_owned()),
            height: cs.latest_height,
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
            timestamp_nanos: cs.timestamp,
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
                SupportedIbcInterface::IbcSolidity => cs.encode_as::<EthAbi>(),
                SupportedIbcInterface::IbcMoveAptos => cs.encode_as::<Bcs>(),
                SupportedIbcInterface::IbcGoV8_08Wasm => {
                    Any(wasm::consensus_state::ConsensusState { data: cs }).encode_as::<Proto>()
                }
            })
            .map(Into::into)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_client_state(
        &self,
        _: &Extensions,
        client_state: Bytes,
        _client_type: ClientType,
    ) -> RpcResult<Bytes> {
        Ok(client_state)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
        _client_type: ClientType,
    ) -> RpcResult<Bytes> {
        Ok(consensus_state)
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
                SupportedIbcInterface::IbcMoveAptos => {
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
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => ics23::merkle_proof::MerkleProof::try_from(
                    protos::ibc::core::commitment::v1::MerkleProof::from(cs),
                )
                .unwrap()
                .encode_as::<EthAbi>(),
                SupportedIbcInterface::IbcMoveAptos => encode_merkle_proof_for_move(
                    ics23::merkle_proof::MerkleProof::try_from(
                        protos::ibc::core::commitment::v1::MerkleProof::from(cs),
                    )
                    .unwrap(),
                ),
                SupportedIbcInterface::IbcGoV8_08Wasm => cs.encode_as::<Proto>(),
            })
            .map(Into::into)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_proto() {
        let bz = hex::decode("000000000000000000000000000000000000000000000000000000000000121e00000000000000000000000000000000000000000000000000000000673e5f09000000000000000000000000000000000000000000000000000000001678f3752f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d2f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d121916d2ccd9d1e831d4bba7333b22130cc71592ce5976b91cf1ee7a212a5a87000000000000000000000000000000000000000000000000000000000000103d0000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000018002d7784e0777d028503c72a1d7f854c4fe5e87fcd4e6792f0b3eefbb1d64ec95149a1c4be3204cf019d678c656b4ddab73185a704852dabef605be0bfad006a8071f587e2dd229db96ba3d022df762b42c166dbdcd0d363f35d411aeec3068d8009f1f4506591fd95f3f8d21b1e29f1c48738bf670f55ddd450d5070540e103d05e4bc2ef4d090ef06fcae2873e242e3bc02d26a5d2625b4a38d765505e7b5cb1a4cc5476bb9b43f4a2812d53a81183af71e9c437627fe91281adeddc7db19932117d5816f60344878430d900070abf2102ec8a9cb73b5c66c5933a79e0a1ef101bd374756d2bfa07f9e1adcc136236a13b261c5dcfd86977421a3dffcc5550f0a244429e18a6162ff9299b09bfcc878f4d386c1c62e8788103b43b0fdfbd3ab03fc8dd2b4e69444c27f3a3120aa7875c10cb67b1d65f079d4b1be92ca91ea1a23ee9387988930d9160d962d779006ea0f8a93d869149ff809fe557181c7f6462f119c0672321a4a5b5c47ffe8faeae4d209d2a49e0ef392ac307c4e6880a4b8").unwrap();

        let header = <Header>::decode_as::<EthAbi>(&bz).unwrap();

        println!("{}", serde_json::to_string_pretty(&header).unwrap());
    }
}
