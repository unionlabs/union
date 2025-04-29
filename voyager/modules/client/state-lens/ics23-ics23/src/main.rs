use alloy_sol_types::SolValue as _;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use macros::model;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use state_lens_ics23_ics23_light_client_types::{ClientState, ConsensusState};
use state_lens_light_client_types::Header;
use tracing::instrument;
use unionlabs::{
    self,
    encoding::{Bcs, DecodeAs, EncodeAs, EthAbi},
    ibc::core::client::height::Height,
    primitives::Bytes,
    tuple::AsTuple,
    union::ics23,
    ErrorReporter,
};
use voyager_message::{
    into_value,
    module::{ClientModuleInfo, ClientModuleServer},
    primitives::{
        ChainId, ClientStateMeta, ClientType, ConsensusStateMeta, ConsensusType, IbcInterface,
        Timestamp,
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
}

impl TryFrom<String> for SupportedIbcInterface {
    // TODO: Better error type here
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            IbcInterface::IBC_SOLIDITY => Ok(SupportedIbcInterface::IbcSolidity),
            IbcInterface::IBC_MOVE_APTOS => Ok(SupportedIbcInterface::IbcMoveAptos),
            _ => Err(format!("unsupported IBC interface: `{value}`")),
        }
    }
}

impl SupportedIbcInterface {
    fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcInterface::IbcSolidity => IbcInterface::IBC_SOLIDITY,
            SupportedIbcInterface::IbcMoveAptos => IbcInterface::IBC_MOVE_APTOS,
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

    async fn new(_: Self::Config, info: ClientModuleInfo) -> Result<Self, BoxDynError> {
        info.ensure_client_type(ClientType::STATE_LENS_ICS23_ICS23)?;
        info.ensure_consensus_type(ConsensusType::TENDERMINT)?;
        Ok(Self {
            ibc_interface: SupportedIbcInterface::try_from(info.ibc_interface.to_string())?,
        })
    }
}

impl Module {
    pub fn decode_consensus_state(consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        ConsensusState::decode_as::<EthAbi>(consensus_state).map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to decode consensus state: {}", ErrorReporter(err)),
                None::<()>,
            )
        })
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => {
                ClientState::abi_decode_params(client_state, true).map_err(|err| {
                    ErrorObject::owned(
                        FATAL_JSONRPC_ERROR_CODE,
                        format!("unable to decode client state: {}", ErrorReporter(err)),
                        None::<()>,
                    )
                })
            }
            SupportedIbcInterface::IbcMoveAptos => {
                <ClientState as AsTuple>::Tuple::decode_as::<Bcs>(client_state)
                    .map(ClientState::from_tuple)
                    .map_err(|err| {
                        ErrorObject::owned(
                            FATAL_JSONRPC_ERROR_CODE,
                            format!("unable to decode client state: {}", ErrorReporter(err)),
                            None::<()>,
                        )
                    })
            }
        }
    }

    pub fn make_height(revision_height: u64) -> Height {
        Height::new(revision_height)
    }
}

#[async_trait]
impl ClientModuleServer for Module {
    #[instrument]
    async fn decode_client_state_meta(
        &self,
        _: &Extensions,
        client_state: Bytes,
    ) -> RpcResult<ClientStateMeta> {
        let cs = self.decode_client_state(&client_state)?;

        Ok(ClientStateMeta {
            counterparty_chain_id: ChainId::new(cs.l2_chain_id.to_string()),
            counterparty_height: Module::make_height(cs.l2_latest_height),
        })
    }

    #[instrument]
    async fn decode_consensus_state_meta(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<ConsensusStateMeta> {
        let cs = Module::decode_consensus_state(&consensus_state)?;

        Ok(ConsensusStateMeta {
            timestamp: Timestamp::from_nanos(cs.timestamp),
        })
    }

    #[instrument]
    async fn decode_client_state(&self, _: &Extensions, client_state: Bytes) -> RpcResult<Value> {
        Ok(into_value(self.decode_client_state(&client_state)?))
    }

    #[instrument]
    async fn decode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Bytes,
    ) -> RpcResult<Value> {
        Ok(into_value(Module::decode_consensus_state(
            &consensus_state,
        )?))
    }

    #[instrument]
    async fn encode_client_state(
        &self,
        _: &Extensions,
        client_state: Value,
        metadata: Value,
    ) -> RpcResult<Bytes> {
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

        serde_json::from_value::<ClientState>(client_state)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize client state: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcMoveAptos => cs.as_tuple().encode_as::<Bcs>(),
                SupportedIbcInterface::IbcSolidity => cs.abi_encode_params(),
            })
            .map(Into::into)
    }

    #[instrument]
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
            .map(|cs| cs.encode_as::<EthAbi>())
            .map(Into::into)
    }

    #[instrument]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<Header>(header)
            .map_err(|err| {
                ErrorObject::owned(
                    FATAL_JSONRPC_ERROR_CODE,
                    format!("unable to deserialize header: {}", ErrorReporter(err)),
                    None::<()>,
                )
            })
            .map(|header| match self.ibc_interface {
                SupportedIbcInterface::IbcMoveAptos => header.encode_as::<Bcs>(),
                SupportedIbcInterface::IbcSolidity => header.encode_as::<EthAbi>(),
            })
            .map(Into::into)
    }

    #[instrument]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        // TODO(aeryz): handle this for cosmos
        let proof = serde_json::from_value::<
            unionlabs::ibc::core::commitment::merkle_proof::MerkleProof,
        >(proof)
        .map_err(|err| {
            ErrorObject::owned(
                FATAL_JSONRPC_ERROR_CODE,
                format!("unable to deserialize proof: {}", ErrorReporter(err)),
                None::<()>,
            )
        })?;

        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => Ok(encode_merkle_proof_for_evm(proof).into()),
            SupportedIbcInterface::IbcMoveAptos => Ok(encode_merkle_proof_for_move(proof).into()),
        }
    }
}

#[model]
struct MoveMembershipProof {
    sub_proof: ics23::existence_proof::ExistenceProof,
    top_level_proof: ics23::existence_proof::ExistenceProof,
}

fn encode_merkle_proof_for_move(
    proof: unionlabs::ibc::core::commitment::merkle_proof::MerkleProof,
) -> Vec<u8> {
    let proof = ics23::merkle_proof::MerkleProof::try_from(
        protos::ibc::core::commitment::v1::MerkleProof::from(proof),
    )
    .unwrap();
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

alloy_sol_types::sol! {
    #[derive(Debug)]
    struct ExistenceProof {
        bytes key;
        bytes value;
        bytes leafPrefix;
        InnerOp[] path;
    }

    #[derive(Debug)]
    struct NonExistenceProof {
        bytes key;
        ExistenceProof left;
        ExistenceProof right;
    }

    #[derive(Debug)]
    struct InnerOp {
        bytes prefix;
        bytes suffix;
    }

    #[derive(Debug)]
    struct ProofSpec {
        uint256 childSize;
        uint256 minPrefixLength;
        uint256 maxPrefixLength;
    }
}

fn encode_merkle_proof_for_evm(
    proof: unionlabs::ibc::core::commitment::merkle_proof::MerkleProof,
) -> Vec<u8> {
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
