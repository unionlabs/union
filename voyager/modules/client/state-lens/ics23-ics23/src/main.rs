use alloy_sol_types::SolValue as _;
use jsonrpsee::{Extensions, core::async_trait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state_lens_ics23_ics23_light_client_types::{ClientState, ConsensusState};
use state_lens_light_client_types::Header;
use tracing::instrument;
use unionlabs::{
    ErrorReporter,
    encoding::{Bincode, DecodeAs, EncodeAs, EthAbi},
    ibc::core::client::height::Height,
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
    IbcCosmwasm,
}

impl TryFrom<String> for SupportedIbcInterface {
    // TODO: Better error type here
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match &*value {
            IbcInterface::IBC_SOLIDITY => Ok(SupportedIbcInterface::IbcSolidity),
            IbcInterface::IBC_COSMWASM => Ok(SupportedIbcInterface::IbcCosmwasm),
            _ => Err(format!("unsupported IBC interface: `{value}`")),
        }
    }
}

impl SupportedIbcInterface {
    fn as_str(&self) -> &'static str {
        match self {
            SupportedIbcInterface::IbcSolidity => IbcInterface::IBC_SOLIDITY,
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

    async fn new(_: Self::Config, info: ClientModuleInfo) -> anyhow::Result<Self> {
        info.ensure_client_type(ClientType::STATE_LENS_ICS23_ICS23)?;
        info.ensure_consensus_type(ConsensusType::TENDERMINT)?;
        Ok(Self {
            ibc_interface: SupportedIbcInterface::try_from(info.ibc_interface.to_string())
                .map_err(|e| anyhow!(e))?,
        })
    }
}

impl Module {
    pub fn decode_consensus_state(consensus_state: &[u8]) -> RpcResult<ConsensusState> {
        ConsensusState::decode_as::<EthAbi>(consensus_state)
            .map_err(RpcError::fatal("unable to decode consensus state"))
    }

    pub fn decode_client_state(&self, client_state: &[u8]) -> RpcResult<ClientState> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => ClientState::decode_as::<EthAbi>(client_state)
                .map_err(RpcError::fatal("unable to decode client state")),
            SupportedIbcInterface::IbcCosmwasm => ClientState::decode_as::<Bincode>(client_state)
                .map_err(RpcError::fatal("unable to decode client state")),
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
            counterparty_chain_id: ChainId::new(cs.l2_chain_id),
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
            timestamp: cs.timestamp,
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
        ensure_null(metadata)?;

        serde_json::from_value::<ClientState>(client_state)
            .map_err(RpcError::fatal("unable to deserialize client state"))
            .map(|cs| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => cs.encode_as::<EthAbi>().into(),
                SupportedIbcInterface::IbcCosmwasm => cs.encode_as::<Bincode>().into(),
            })
    }

    #[instrument]
    async fn encode_consensus_state(
        &self,
        _: &Extensions,
        consensus_state: Value,
    ) -> RpcResult<Bytes> {
        serde_json::from_value::<ConsensusState>(consensus_state)
            .map_err(RpcError::fatal("unable to deserialize consensus state"))
            .map(|cs| cs.encode_as::<EthAbi>().into())
    }

    #[instrument]
    async fn encode_header(&self, _: &Extensions, header: Value) -> RpcResult<Bytes> {
        serde_json::from_value::<Header>(header)
            .map_err(RpcError::fatal("unable to deserialize header"))
            .map(|header| match self.ibc_interface {
                SupportedIbcInterface::IbcSolidity => header.encode_as::<EthAbi>().into(),
                SupportedIbcInterface::IbcCosmwasm => header.encode_as::<Bincode>().into(),
            })
    }

    #[instrument]
    async fn decode_header(&self, _: &Extensions, header: Bytes) -> RpcResult<Value> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => Header::decode_as::<EthAbi>(&header)
                .map_err(RpcError::fatal("unable to decode header")),
            SupportedIbcInterface::IbcCosmwasm => Header::decode_as::<Bincode>(&header)
                .map_err(RpcError::fatal("unable to decode header")),
        }
        .map(into_value)
    }

    #[instrument]
    async fn encode_proof(&self, _: &Extensions, proof: Value) -> RpcResult<Bytes> {
        // TODO(aeryz): handle this for cosmos
        let proof = serde_json::from_value::<
            unionlabs::ibc::core::commitment::merkle_proof::MerkleProof,
        >(proof)
        .map_err(RpcError::fatal("unable to deserialize proof"))?;

        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => Ok(encode_merkle_proof_for_evm(proof).into()),
            SupportedIbcInterface::IbcCosmwasm => Ok(proof.encode_as::<Bincode>().into()),
        }
    }

    #[instrument]
    async fn decode_proof(&self, _: &Extensions, proof: Bytes) -> RpcResult<Value> {
        match self.ibc_interface {
            SupportedIbcInterface::IbcSolidity => decode_merkle_proof_for_evm(proof),
            SupportedIbcInterface::IbcCosmwasm => {
                unionlabs::ibc::core::commitment::merkle_proof::MerkleProof::decode_as::<Bincode>(
                    &proof,
                )
                .map(into_value)
                .map_err(RpcError::fatal("unable to decode proof"))
            }
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

fn encode_merkle_proof_for_evm(
    proof: unionlabs::ibc::core::commitment::merkle_proof::MerkleProof,
) -> Vec<u8> {
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
