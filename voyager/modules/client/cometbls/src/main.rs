use alloy_sol_types::SolValue;
use cometbls_groth16_verifier::ZKP;
use cometbls_light_client_types::{ClientState, ConsensusState, Header};
use garaga_rs::calldata::{
    G1PointBigUint, G2PointBigUint,
    cometbls_groth16::CometblsGroth16VerifyingKey,
    full_proof_with_hints::groth16::{Groth16Proof, Groth16VerificationKey},
};
use jsonrpsee::{Extensions, core::async_trait};
use macros::model;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use substrate_bn::{AffineG1, AffineG2, Fq, G1, G2};
use tracing::{debug, instrument};
use unionlabs::{
    encoding::{Bcs, Bincode, DecodeAs, EncodeAs, EthAbi},
    primitives::{Bytes, H256},
    union::ics23,
};
use voyager_sdk::{
    anyhow::{self, anyhow},
    ensure_null,
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
    async fn encode_header(&self, e: &Extensions, header: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<(cometbls_light_client_types::ChainId, H256, Header)>(header)
            .map_err(RpcError::fatal("unable to deserialize header"))
            .and_then(
                |(chain_id, trusted_validators_hash, mut header)| match self.ibc_interface {
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
                    SupportedIbcInterface::IbcCairo => {
                        header.zero_knowledge_proof =
                            prepare_zkp_for_cairo(chain_id, trusted_validators_hash, &header)
                                .unwrap();
                        todo!()
                    }
                },
            )
    }

    #[instrument(skip_all)]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        debug!(%proof, "encoding proof");

        serde_json::from_value::<unionlabs::ibc::core::commitment::merkle_proof::MerkleProof>(proof)
            .map_err(RpcError::fatal("unable to deserialize proof"))
            .map(|proof| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => encode_merkle_proof_for_evm(proof),
                SupportedIbcInterface::IbcCosmwasm => proof.encode_as::<Bincode>(),
                SupportedIbcInterface::IbcMoveAptos | SupportedIbcInterface::IbcMoveSui => {
                    encode_merkle_proof_for_move(
                        ics23::merkle_proof::MerkleProof::try_from(
                            protos::ibc::core::commitment::v1::MerkleProof::from(proof),
                        )
                        .unwrap(),
                    )
                }
                // TODO(aeryz): cairo serde
                SupportedIbcInterface::IbcCairo => proof.encode_as::<Bincode>(),
            })
            .map(Into::into)
    }
}

fn prepare_zkp_for_cairo(
    chain_id: cometbls_light_client_types::ChainId,
    trusted_validators_hash: H256,
    header: &Header,
) -> Result<Bytes, cometbls_groth16_verifier::Error> {
    let zkp = ZKP::try_from(header.zero_knowledge_proof.as_ref())?;

    let public_inputs = cometbls_groth16_verifier::public_inputs(
        &chain_id,
        trusted_validators_hash,
        &header.signed_header,
        &zkp,
    )?;

    let make_big_uint = |point: Fq| {
        let mut buffer = Vec::new();
        point.to_big_endian(&mut buffer).unwrap();
        BigUint::from_bytes_be(&buffer)
    };

    let make_g1_affine = |point: AffineG1| G1PointBigUint {
        x: make_big_uint(point.x()),
        y: make_big_uint(point.y()),
    };

    let make_g1 = |point: G1| G1PointBigUint {
        x: make_big_uint(point.x()),
        y: make_big_uint(point.y()),
    };

    let make_g2_affine = |point: AffineG2| G2PointBigUint {
        x0: make_big_uint(point.x().real()),
        x1: make_big_uint(point.x().imaginary()),
        y0: make_big_uint(point.y().real()),
        y1: make_big_uint(point.y().imaginary()),
    };

    let make_g2 = |point: G2| G2PointBigUint {
        x0: make_big_uint(point.x().real()),
        x1: make_big_uint(point.x().imaginary()),
        y0: make_big_uint(point.y().real()),
        y1: make_big_uint(point.y().imaginary()),
    };

    let proof = Groth16Proof {
        a: make_g1_affine(zkp.proof.a),
        b: make_g2_affine(zkp.proof.b),
        c: make_g1_affine(zkp.proof.c),
        public_inputs: public_inputs
            .into_iter()
            .map(|pi| {
                let mut buffer = Vec::new();
                pi.to_big_endian(&mut buffer).unwrap();
                BigUint::from_bytes_be(&buffer)
            })
            .collect(),
        image_id_journal_risc0: None,
        vkey_public_values_sp1: None,
    };

    let vk = Groth16VerificationKey {
        alpha: make_g1(cometbls_groth16_verifier::ALPHA_G1),
        beta: make_g2(cometbls_groth16_verifier::BETA_NEG_G2),
        gamma: make_g2(cometbls_groth16_verifier::GAMMA_NEG_G2),
        delta: make_g2(cometbls_groth16_verifier::DELTA_NEG_G2),
        ic: cometbls_groth16_verifier::GAMMA_ABC_G1
            .into_iter()
            .map(make_g1)
            .collect(),
    };

    let vk = CometblsGroth16VerifyingKey {
        groth16_vk: vk,
        commitment_key_g: make_g2(cometbls_groth16_verifier::PEDERSEN_G),
        commitment_key_g_root_sigma_neg: make_g2(
            cometbls_groth16_verifier::PEDERSEN_G_ROOT_SIGMA_NEG,
        ),
    };

    let proof_commitment = make_g1_affine(zkp.proof_commitment);
    let proof_commitment_pok = make_g1_affine(zkp.proof_commitment_pok);

    let proof = garaga_rs::calldata::cometbls_groth16::CometblsGroth16Proof::generate_calldata(
        proof,
        vk,
        proof_commitment,
        proof_commitment_pok,
    );

    todo!()
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
    sub_proof: ics23::existence_proof::ExistenceProof,
    top_level_proof: ics23::existence_proof::ExistenceProof,
}

#[model]
struct MoveNonMembershipProof {
    existence_proof: ics23::existence_proof::ExistenceProof,
    non_existence_proof: ics23::non_existence_proof::NonExistenceProof,
}

fn encode_merkle_proof_for_move(proof: ics23::merkle_proof::MerkleProof) -> Vec<u8> {
    match proof {
        ics23::merkle_proof::MerkleProof::Membership(sub_proof, top_level_proof) => {
            MoveMembershipProof {
                sub_proof,
                top_level_proof,
            }
            .encode_as::<Bcs>()
        }
        ics23::merkle_proof::MerkleProof::NonMembership(non_existence_proof, existence_proof) => {
            MoveNonMembershipProof {
                existence_proof,
                non_existence_proof,
            }
            .encode_as::<Bcs>()
        }
    }
}
