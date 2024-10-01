use ark_serialize::{CanonicalSerialize, SerializationError, Valid};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use macros::model;
use queue_msg::BoxDynError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serde_utils::Hex;
use tracing::{debug, instrument};
use unionlabs::{
    self,
    encoding::{Bcs, DecodeAs, EncodeAs, EthAbi, Proto},
    google::protobuf::any::Any,
    ibc::lightclients::{
        cometbls::{client_state::ClientState, consensus_state::ConsensusState, header::Header},
        wasm,
    },
    union::ics23,
    ErrorReporter,
};
use voyager_message::{
    core::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType,
        IbcGo08WasmClientMetadata, IbcInterface,
    },
    module::{ClientModuleInfo, ClientModuleServer, ModuleInfo},
    run_module_server, DefaultCmd, ModuleContext, FATAL_JSONRPC_ERROR_CODE,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    run_module_server::<Module>().await
}

#[model(no_serde)]
#[derive(Copy, Serialize, Deserialize)]
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
pub struct Config {
    pub ibc_interface: SupportedIbcInterface,
}

impl ModuleContext for Module {
    type Config = Config;
    type Cmd = DefaultCmd;
    type Info = ClientModuleInfo;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Self {
            ibc_interface: config.ibc_interface,
        })
    }

    fn info(config: Self::Config) -> ModuleInfo<Self::Info> {
        ModuleInfo {
            kind: ClientModuleInfo {
                client_type: ClientType::new(ClientType::COMETBLS_GROTH16),
                consensus_type: ConsensusType::new(ConsensusType::COMETBLS),
                ibc_interface: IbcInterface::new(config.ibc_interface.as_str()),
            },
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
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
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = self.decode_consensus_state(&consensus_state.0)?;

        Ok(ConsensusStateMeta {
            timestamp_nanos: cs.timestamp,
        })
    }

    #[instrument(skip_all)]
    async fn decode_client_state(
        &self,
        _: &Extensions,
        client_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_client_state(&client_state.0)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
    ) -> RpcResult<Value> {
        Ok(serde_json::to_value(self.decode_consensus_state(&consensus_state.0)?).unwrap())
    }

    #[instrument(skip_all)]
    async fn encode_client_state(
        &self,
        _: &Extensions,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
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
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Hex<Vec<u8>>> {
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
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_client_state(
        &self,
        _: &Extensions,
        client_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        Ok(client_state)
    }

    #[instrument(skip_all)]
    async fn reencode_counterparty_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Hex<Vec<u8>>,
        _client_type: ClientType<'static>,
    ) -> RpcResult<Hex<Vec<u8>>> {
        Ok(consensus_state)
    }

    #[instrument(skip_all)]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Hex<Vec<u8>>> {
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
            .map(Hex)
    }

    #[instrument(skip_all)]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Hex<Vec<u8>>> {
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
        let bz = hex::decode("0a272f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e744d6573736167651286040a83040a77088907120c0880a8c7b70610fabddce0021a202f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d22202f4975ab7e75a677f43efebf53e0ec05460d2cf55506ad08d6b05254f96a500d2a20eddaa32275fbbf44c6a21e32b59b097bed5374be715eab22f093399a9700a1e41205080110bf051a80031d530ee22263bc9e7008e3bd982c966b226d1018814e5b4d07597b4d35aea56b2ef63fdddb29fe06ef99cf645201a12e8b98b9ff7a7cec0819f696e17413294b0c638c4f946f4d4af4da8dd0815de2f5af8fd8612d1c98e9846846ea1ec78aac046df852b916de3fd8b3332bc3d23073e11b252b023711c18b19952507428da12e2baf74a03ca7bdc37edd0123e47f0a3a029f6da43a32dc6830e126b4ddf8712f2a0e021ac0f6414f171156f6a9019d6ea53cd30762c1e60d6a0e029778586c0cc1e2e13f7c45347a2a3ba82e43eccdc468fc8a05ba0a95fef26777872c27e42317f2c76c0a5f41e63088b8b394c5a7a3066809952f489718142107bd7b24572074be60bdb7611f1c916061a5ab3dc75a62b953a19650d839027a885801252a1e1cd84f8ba570047c2f1d220f26f7b11e69b7519f092d31ff954e92fd012a931ea2b4d20942376502043ba98e69f351f60b12e5a7ff180e5a1a966697d80696066694fa833420f5db7e3ae1b91dbce06fe2ffa1ea0a503af6a93f61ad7aa4f4").unwrap();

        let header =
            <Any<wasm::client_message::ClientMessage<Header>>>::decode_as::<Proto>(&bz).unwrap();

        dbg!(serde_json::to_string_pretty(&header).unwrap());
    }
}
