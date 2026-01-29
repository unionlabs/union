use alloy_sol_types::SolValue;
use cometbls_light_client_types::{ClientState, ConsensusState, Header};
use jsonrpsee::{Extensions, core::async_trait};
use macros::model;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, instrument};
use unionlabs::{
    ErrorReporter,
    encoding::{Bcs, Bincode, DecodeAs, EncodeAs, EthAbi},
    ibc::core::commitment::merkle_proof::MerkleProof,
    primitives::Bytes,
};
use voyager_sdk::{
    anyhow::{self, anyhow},
    ensure_null, into_value,
    plugin::ClientModule,
    primitives::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType, IbcInterface,
    },
    rpc::{ClientModuleServer, RpcError, RpcResult, types::ClientModuleInfo},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum SupportedIbcInterface {
    IbcSolidity,
    IbcMoveAptos,
    IbcMoveSui,
    IbcCosmwasm,
    IbcCairo,
}

impl TryFrom<String> for SupportedIbcInterface {
    // TODO: Better error type here
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            IbcInterface::IBC_SOLIDITY => Ok(SupportedIbcInterface::IbcSolidity),
            IbcInterface::IBC_MOVE_APTOS => Ok(SupportedIbcInterface::IbcMoveAptos),
            IbcInterface::IBC_MOVE_SUI => Ok(SupportedIbcInterface::IbcMoveSui),
            IbcInterface::IBC_COSMWASM => Ok(SupportedIbcInterface::IbcCosmwasm),
            IbcInterface::IBC_CAIRO => Ok(SupportedIbcInterface::IbcCairo),
            _ => Err(format!("unsupported IBC interface: `{value}`")),
        }
    }
}

impl SupportedIbcInterface {
    fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcInterface::IbcSolidity => IbcInterface::IBC_SOLIDITY,
            SupportedIbcInterface::IbcMoveAptos => IbcInterface::IBC_MOVE_APTOS,
            SupportedIbcInterface::IbcMoveSui => IbcInterface::IBC_MOVE_SUI,
            // SupportedIbcInterface::IbcGoV8_08Wasm => IbcInterface::IBC_GO_V8_08_WASM,
            SupportedIbcInterface::IbcCosmwasm => IbcInterface::IBC_COSMWASM,
            SupportedIbcInterface::IbcCairo => IbcInterface::IBC_CAIRO,
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

    async fn new(Config {}: Self::Config, info: ClientModuleInfo) -> anyhow::Result<Self> {
        info.ensure_client_type(ClientType::COMETBLS_GROTH16)?;
        info.ensure_consensus_type(ConsensusType::COMETBLS)?;

        Ok(Self {
            ibc_interface: SupportedIbcInterface::try_from(info.ibc_interface.to_string())
                .map_err(|e| anyhow!(e))?,
        })
    }
}

impl Module {
    pub fn decode_consensus_state(&self, consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity
            | SupportedIbcInterface::IbcMoveAptos
            | SupportedIbcInterface::IbcMoveSui
            | SupportedIbcInterface::IbcCairo
            | SupportedIbcInterface::IbcCosmwasm => {
                ConsensusState::decode_as::<EthAbi>(consensus_state)
                    .map_err(RpcError::fatal("unable to decode consensus state"))
            }
        }
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => ClientState::decode_as::<EthAbi>(client_state)
                .map_err(RpcError::fatal("unable to decode client state")),
            SupportedIbcInterface::IbcMoveAptos | SupportedIbcInterface::IbcMoveSui => {
                ClientState::decode_as::<Bcs>(client_state)
                    .map_err(RpcError::fatal("unable to decode client state"))
            }
            SupportedIbcInterface::IbcCosmwasm => ClientState::decode_as::<Bincode>(client_state)
                .map_err(RpcError::fatal("unable to decode client state")),
            // TODO(aeryz): cairo serde
            SupportedIbcInterface::IbcCairo => ClientState::decode_as::<Bincode>(client_state)
                .map_err(RpcError::fatal("unable to decode client state")),
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
            .map_err(RpcError::fatal("unable to deserialize client state"))
            .and_then(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => {
                    ensure_null(metadata)?;

                    Ok(cs.encode_as::<EthAbi>().into())
                }
                SupportedIbcInterface::IbcMoveAptos | SupportedIbcInterface::IbcMoveSui => {
                    ensure_null(metadata)?;

                    Ok(cs.encode_as::<Bcs>().into())
                }
                SupportedIbcInterface::IbcCosmwasm => {
                    ensure_null(metadata)?;

                    Ok(cs.encode_as::<Bincode>().into())
                }
                // TODO(aeryz): cairo serde
                SupportedIbcInterface::IbcCairo => {
                    ensure_null(metadata)?;

                    Ok(cs.encode_as::<Bincode>().into())
                }
            })
    }

    #[instrument(skip_all)]
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
        serde_json::from_value::<ConsensusState>(consensus_state)
            .map_err(RpcError::fatal("unable to deserialize consensus state"))
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity
                | SupportedIbcInterface::IbcMoveAptos
                | SupportedIbcInterface::IbcMoveSui
                | SupportedIbcInterface::IbcCairo
                | SupportedIbcInterface::IbcCosmwasm => cs.encode_as::<EthAbi>().into(),
            })
    }

    #[instrument(skip_all)]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<Header>(header)
            .map_err(RpcError::fatal("unable to deserialize header"))
            .and_then(|mut header| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => Ok(header.encode_as::<EthAbi>().into()),
                SupportedIbcInterface::IbcCosmwasm => Ok(header.encode_as::<Bincode>().into()),
                SupportedIbcInterface::IbcMoveAptos | SupportedIbcInterface::IbcMoveSui => {
                    header.zero_knowledge_proof =
                        gnark_key_parser::bls12381::reencode_evm_zkp_for_sui(
                            &header.zero_knowledge_proof,
                        )
                        .map_err(RpcError::fatal("unable to decode zk"))?
                        .into();
                    Ok(header.encode_as::<Bcs>().into())
                }
                SupportedIbcInterface::IbcCairo => Ok(header.encode_as::<Bincode>().into()),
            })
    }

    #[instrument]
    async fn decode_header(&self, _: &Extensions, header: Bytes) -> RpcResult<Value> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => Header::decode_as::<EthAbi>(&header)
                .map_err(RpcError::fatal("unable to decode header")),
            SupportedIbcInterface::IbcMoveAptos | SupportedIbcInterface::IbcMoveSui => {
                Header::decode_as::<Bincode>(&header)
                    .map_err(RpcError::fatal("unable to decode header"))
            }
            SupportedIbcInterface::IbcCosmwasm => Header::decode_as::<Bcs>(&header)
                .map_err(RpcError::fatal("unable to decode header")),
            SupportedIbcInterface::IbcCairo => Header::decode_as::<Bincode>(&header)
                .map_err(RpcError::fatal("unable to decode header")),
        }
        .map(into_value)
    }

    #[instrument(skip_all)]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        debug!(%proof, "encoding proof");

        serde_json::from_value::<MerkleProof>(proof)
            .map_err(RpcError::fatal("unable to deserialize proof"))
            .map(|proof| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => encode_merkle_proof_for_evm(proof),
                SupportedIbcInterface::IbcCosmwasm => proof.encode_as::<Bincode>(),
                SupportedIbcInterface::IbcMoveAptos | SupportedIbcInterface::IbcMoveSui => {
                    encode_merkle_proof_for_move(
                        unionlabs::union::ics23::merkle_proof::MerkleProof::try_from(
                            protos::ibc::core::commitment::v1::MerkleProof::from(proof),
                        )
                        .unwrap(),
                    )
                }
                SupportedIbcInterface::IbcCairo => proof.encode_as::<Bincode>(),
            })
            .map(Into::into)
    }

    #[instrument]
    async fn decode_proof(&self, _: &Extensions, proof: Bytes) -> RpcResult<Value> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => decode_merkle_proof_for_evm(proof),
            SupportedIbcInterface::IbcMoveAptos | SupportedIbcInterface::IbcMoveSui => {
                decode_merkle_proof_for_move(proof)
            }
            SupportedIbcInterface::IbcCosmwasm => MerkleProof::decode_as::<Bincode>(&proof)
                .map(into_value)
                .map_err(RpcError::fatal("unable to proof")),
            SupportedIbcInterface::IbcCairo => MerkleProof::decode_as::<Bincode>(&proof)
                .map(into_value)
                .map_err(RpcError::fatal("unable to proof")),
        }
    }
}

alloy_sol_types::sol! {
    #[derive(Default, PartialEq)]
    struct SolExistenceProof {
        bytes key;
        bytes value;
        bytes leafPrefix;
        SolInnerOp[] path;
    }

    #[derive(Default, PartialEq)]
    struct SolNonExistenceProof {
        bytes key;
        SolExistenceProof left;
        SolExistenceProof right;
    }

    #[derive(Default, PartialEq)]
    struct SolInnerOp {
        bytes prefix;
        bytes suffix;
    }
}

fn encode_merkle_proof_for_evm(proof: MerkleProof) -> Vec<u8> {
    let merkle_proof = unionlabs::union::ics23::merkle_proof::MerkleProof::try_from(
        protos::ibc::core::commitment::v1::MerkleProof::from(proof),
    )
    .unwrap();

    let convert_inner_op = |i: unionlabs::union::ics23::inner_op::InnerOp| SolInnerOp {
        prefix: i.prefix.into(),
        suffix: i.suffix.into(),
    };

    let convert_existence_proof =
        |e: unionlabs::union::ics23::existence_proof::ExistenceProof| SolExistenceProof {
            key: e.key.into(),
            value: e.value.into(),
            leafPrefix: e.leaf_prefix.into(),
            path: e.path.into_iter().map(convert_inner_op).collect(),
        };

    let exist_default = || unionlabs::union::ics23::existence_proof::ExistenceProof {
        key: vec![].into(),
        value: vec![].into(),
        leaf_prefix: vec![].into(),
        path: vec![],
    };

    match merkle_proof {
        unionlabs::union::ics23::merkle_proof::MerkleProof::Membership(a, b) => {
            (convert_existence_proof(a), convert_existence_proof(b)).abi_encode_params()
        }
        unionlabs::union::ics23::merkle_proof::MerkleProof::NonMembership(a, b) => (
            SolNonExistenceProof {
                key: a.key.into(),
                left: convert_existence_proof(a.left.unwrap_or_else(exist_default)),
                right: convert_existence_proof(a.right.unwrap_or_else(exist_default)),
            },
            convert_existence_proof(b),
        )
            .abi_encode_params(),
    }
}

fn decode_merkle_proof_for_evm(proof: Bytes) -> RpcResult<Value> {
    let convert_inner_op = |i: SolInnerOp| unionlabs::union::ics23::inner_op::InnerOp {
        prefix: i.prefix.into(),
        suffix: i.suffix.into(),
    };

    let convert_existence_proof =
        |e: SolExistenceProof| unionlabs::union::ics23::existence_proof::ExistenceProof {
            key: e.key.into(),
            value: e.value.into(),
            leaf_prefix: e.leafPrefix.into(),
            path: e.path.into_iter().map(convert_inner_op).collect(),
        };

    match (
        <(SolExistenceProof, SolExistenceProof)>::abi_decode_params_validate(&proof),
        <(SolNonExistenceProof, SolExistenceProof)>::abi_decode_params_validate(&proof),
    ) {
        (Ok(_), Ok(_)) => Err(RpcError::fatal_from_message(
            "proof cannot be a both a valid existence and non existence proof",
        )),
        (Ok((a, b)), Err(_)) => Ok(into_value(
            unionlabs::union::ics23::merkle_proof::MerkleProof::Membership(
                convert_existence_proof(a),
                convert_existence_proof(b),
            ),
        )),
        (Err(_), Ok((a, b))) => Ok(into_value(
            unionlabs::union::ics23::merkle_proof::MerkleProof::NonMembership(
                unionlabs::union::ics23::non_existence_proof::NonExistenceProof {
                    key: a.key.to_vec(),
                    left: if a.left == Default::default() {
                        None
                    } else {
                        Some(convert_existence_proof(a.left))
                    },
                    right: if a.right == Default::default() {
                        None
                    } else {
                        Some(convert_existence_proof(a.right))
                    },
                },
                convert_existence_proof(b),
            ),
        )),
        (Err(existence_err), Err(non_existence_err)) => Err(RpcError::fatal_from_message(format!(
            "invalid proof, could not decode as existence ({}) or non existence ({})",
            ErrorReporter(existence_err),
            ErrorReporter(non_existence_err),
        ))),
    }
}

// fn reencode_zkp_for_move(zkp: &[u8]) -> Result<Vec<u8>, SerializationError> {
//     let mut buf = Vec::new();

//     let serialize_g1 =
//         |cursor: &mut usize, buf: &mut Vec<u8>, zkp: &[u8]| -> Result<(), SerializationError> {
//             let proof = ark_bn254::G1Affine::new_unchecked(
//                 ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
//                     &zkp[*cursor..*cursor + 32],
//                 )),
//                 ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
//                     &zkp[*cursor + 32..*cursor + 64],
//                 )),
//             );
//             proof.check()?;
//             *cursor += 64;
//             proof.serialize_compressed(buf)?;
//             Ok(())
//         };

//     let serialize_g2 =
//         |cursor: &mut usize, buf: &mut Vec<u8>, zkp: &[u8]| -> Result<(), SerializationError> {
//             let proof = ark_bn254::G2Affine::new_unchecked(
//                 ark_bn254::Fq2::new(
//                     ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
//                         &zkp[*cursor + 32..*cursor + 64],
//                     )),
//                     ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
//                         &zkp[*cursor..*cursor + 32],
//                     )),
//                 ),
//                 ark_bn254::Fq2::new(
//                     ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
//                         &zkp[*cursor + 96..*cursor + 128],
//                     )),
//                     ark_bn254::Fq::from(num_bigint::BigUint::from_bytes_be(
//                         &zkp[*cursor + 64..*cursor + 96],
//                     )),
//                 ),
//             );
//             proof.check()?;
//             *cursor += 128;
//             proof.serialize_compressed(buf)?;
//             Ok(())
//         };

//     let mut cursor = 0;
//     // zkp.proof.a
//     serialize_g1(&mut cursor, &mut buf, zkp)?;
//     // zkp.proof.b
//     serialize_g2(&mut cursor, &mut buf, zkp)?;
//     // zkp.proof.c
//     serialize_g1(&mut cursor, &mut buf, zkp)?;
//     // zkp.poc
//     serialize_g1(&mut cursor, &mut buf, zkp)?;
//     // zkp.pok
//     serialize_g1(&mut cursor, &mut buf, zkp)?;

//     Ok(buf)
// }

#[model]
struct MoveMembershipProof {
    sub_proof: unionlabs::union::ics23::existence_proof::ExistenceProof,
    top_level_proof: unionlabs::union::ics23::existence_proof::ExistenceProof,
}

#[model]
struct MoveNonMembershipProof {
    existence_proof: unionlabs::union::ics23::existence_proof::ExistenceProof,
    non_existence_proof: unionlabs::union::ics23::non_existence_proof::NonExistenceProof,
}

fn encode_merkle_proof_for_move(
    proof: unionlabs::union::ics23::merkle_proof::MerkleProof,
) -> Vec<u8> {
    match proof {
        unionlabs::union::ics23::merkle_proof::MerkleProof::Membership(
            sub_proof,
            top_level_proof,
        ) => MoveMembershipProof {
            sub_proof,
            top_level_proof,
        }
        .encode_as::<Bcs>(),
        unionlabs::union::ics23::merkle_proof::MerkleProof::NonMembership(
            non_existence_proof,
            existence_proof,
        ) => MoveNonMembershipProof {
            existence_proof,
            non_existence_proof,
        }
        .encode_as::<Bcs>(),
    }
}

fn decode_merkle_proof_for_move(proof: Bytes) -> RpcResult<Value> {
    match (
        MoveMembershipProof::decode_as::<Bcs>(&proof),
        MoveNonMembershipProof::decode_as::<Bcs>(&proof),
    ) {
        (Ok(_), Ok(_)) => Err(RpcError::fatal_from_message(
            "proof cannot be a both a valid existence and non existence proof",
        )),
        (
            Ok(MoveMembershipProof {
                sub_proof,
                top_level_proof,
            }),
            Err(_),
        ) => Ok(into_value(
            unionlabs::union::ics23::merkle_proof::MerkleProof::Membership(
                sub_proof,
                top_level_proof,
            ),
        )),
        (
            Err(_),
            Ok(MoveNonMembershipProof {
                existence_proof,
                non_existence_proof,
            }),
        ) => Ok(into_value(
            unionlabs::union::ics23::merkle_proof::MerkleProof::NonMembership(
                non_existence_proof,
                existence_proof,
            ),
        )),
        (Err(existence_err), Err(non_existence_err)) => Err(RpcError::fatal_from_message(format!(
            "invalid proof, could not decode as existence ({}) or non existence ({})",
            ErrorReporter(existence_err),
            ErrorReporter(non_existence_err),
        ))),
    }
}
