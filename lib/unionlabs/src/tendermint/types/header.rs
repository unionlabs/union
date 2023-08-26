use prost::Message;
use protos::google::protobuf::{BytesValue, Int64Value, StringValue};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use serde::{Deserialize, Serialize};

use crate::{
    bounded_int::{BoundedI64, BoundedIntError},
    errors::{required, InvalidLength, MissingField},
    ethereum::{Address, H256},
    ibc::google::protobuf::timestamp::Timestamp,
    tendermint::{types::block_id::BlockId, version::consensus::Consensus},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    /// basic block info
    pub version: Consensus,
    pub chain_id: String,
    pub height: BoundedI64<0, { i64::MAX }>,
    pub time: Timestamp,
    /// prev block info
    pub last_block_id: BlockId,
    /// hashes of block data
    ///
    /// commit from validators from the last block
    pub last_commit_hash: H256,
    /// transactions
    pub data_hash: H256,
    /// hashes from the app output from the prev block
    ///
    /// validators for the current block
    pub validators_hash: H256,
    /// validators for the next block
    pub next_validators_hash: H256,
    /// consensus params for current block
    pub consensus_hash: H256,
    /// state after txs from the previous block
    pub app_hash: H256,
    /// root hash of all results from the txs from the previous block
    pub last_results_hash: H256,
    /// consensus info
    ///
    /// evidence included in the block
    pub evidence_hash: H256,
    /// original proposer of the block
    pub proposer_address: Address,
}

impl Header {
    #[must_use]
    pub fn calculate_merkle_root(&self) -> Option<[u8; 32]> {
        let header: protos::tendermint::types::Header = self.clone().into();

        let do_hash = |prefix: u8, v: Vec<u8>| {
            Sha256::hash(
                &[prefix]
                    .into_iter()
                    .chain(v.into_iter())
                    .collect::<Vec<_>>(),
            )
        };

        const LEAF_PREFIX: u8 = 0;
        const INNER_PREFIX: u8 = 1;

        let leaf_hash = |v: Vec<u8>| do_hash(LEAF_PREFIX, v);
        let inner_hash = |l: &<Sha256 as Hasher>::Hash, r: &<Sha256 as Hasher>::Hash| {
            do_hash(
                INNER_PREFIX,
                l.iter()
                    .copied()
                    .chain(r.iter().copied())
                    .collect::<Vec<_>>(),
            )
        };

        let mut leaves = [
            leaf_hash(header.version?.encode_to_vec()),
            leaf_hash(
                StringValue {
                    value: header.chain_id,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                Int64Value {
                    value: header.height,
                }
                .encode_to_vec(),
            ),
            leaf_hash(header.time?.encode_to_vec()),
            leaf_hash(header.last_block_id?.encode_to_vec()),
            leaf_hash(
                BytesValue {
                    value: header.last_commit_hash,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.data_hash,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.validators_hash,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.next_validators_hash,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.consensus_hash,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.app_hash,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.last_results_hash,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.evidence_hash,
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.proposer_address,
                }
                .encode_to_vec(),
            ),
        ];

        leaves[0] = inner_hash(&leaves[0], &leaves[1]);
        leaves[1] = inner_hash(&leaves[2], &leaves[3]);
        leaves[2] = inner_hash(&leaves[4], &leaves[5]);
        leaves[3] = inner_hash(&leaves[6], &leaves[7]);
        leaves[4] = inner_hash(&leaves[8], &leaves[9]);
        leaves[5] = inner_hash(&leaves[10], &leaves[11]);
        leaves[6] = inner_hash(&leaves[12], &leaves[13]);

        leaves[0] = inner_hash(&leaves[0], &leaves[1]);
        leaves[1] = inner_hash(&leaves[2], &leaves[3]);
        leaves[2] = inner_hash(&leaves[4], &leaves[5]);
        leaves[3] = leaves[6];

        leaves[0] = inner_hash(&leaves[0], &leaves[1]);
        leaves[1] = inner_hash(&leaves[2], &leaves[3]);

        let root = inner_hash(&leaves[0], &leaves[1]);

        Some(root)
    }
}

impl From<Header> for protos::tendermint::types::Header {
    fn from(value: Header) -> Self {
        Self {
            version: Some(value.version.into()),
            chain_id: value.chain_id,
            height: value.height.into(),
            time: Some(value.time.into()),
            last_block_id: Some(value.last_block_id.into()),
            last_commit_hash: value.last_commit_hash.into(),
            data_hash: value.data_hash.into(),
            validators_hash: value.validators_hash.into(),
            next_validators_hash: value.next_validators_hash.into(),
            consensus_hash: value.consensus_hash.into(),
            app_hash: value.app_hash.into(),
            last_results_hash: value.last_results_hash.into(),
            evidence_hash: value.evidence_hash.into(),
            proposer_address: value.proposer_address.into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromHeaderError {
    MissingField(MissingField),
    LastBlockId(TryFromProtoErrorOf<BlockId>),
    Height(BoundedIntError<i64>),
    Timestamp(TryFromProtoErrorOf<Timestamp>),
    LastCommitHash(InvalidLength),
    DataHash(InvalidLength),
    ValidatorsHash(InvalidLength),
    NextValidatorsHash(InvalidLength),
    ConsensusHash(InvalidLength),
    AppHash(InvalidLength),
    LastResultsHash(InvalidLength),
    EvidenceHash(InvalidLength),
    ProposerAddress(InvalidLength),
}

impl TryFrom<protos::tendermint::types::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(value: protos::tendermint::types::Header) -> Result<Self, Self::Error> {
        Ok(Self {
            version: required!(value.version)?.into(),
            chain_id: value.chain_id,
            height: value
                .height
                .try_into()
                .map_err(TryFromHeaderError::Height)?,
            time: required!(value.time)?
                .try_into()
                .map_err(TryFromHeaderError::Timestamp)?,
            last_block_id: required!(value.last_block_id)?
                .try_into()
                .map_err(TryFromHeaderError::LastBlockId)?,
            last_commit_hash: value
                .last_commit_hash
                .try_into()
                .map_err(TryFromHeaderError::LastCommitHash)?,
            data_hash: value
                .data_hash
                .try_into()
                .map_err(TryFromHeaderError::DataHash)?,
            validators_hash: value
                .validators_hash
                .try_into()
                .map_err(TryFromHeaderError::ValidatorsHash)?,
            next_validators_hash: value
                .next_validators_hash
                .try_into()
                .map_err(TryFromHeaderError::NextValidatorsHash)?,
            consensus_hash: value
                .consensus_hash
                .try_into()
                .map_err(TryFromHeaderError::ConsensusHash)?,
            app_hash: value
                .app_hash
                .try_into()
                .map_err(TryFromHeaderError::AppHash)?,
            last_results_hash: value
                .last_results_hash
                .try_into()
                .map_err(TryFromHeaderError::LastResultsHash)?,
            evidence_hash: value
                .evidence_hash
                .try_into()
                .map_err(TryFromHeaderError::EvidenceHash)?,
            proposer_address: value
                .proposer_address
                .try_into()
                .map_err(TryFromHeaderError::ProposerAddress)?,
        })
    }
}

impl Proto for Header {
    type Proto = protos::tendermint::types::Header;
}

impl TypeUrl for protos::tendermint::types::Header {
    const TYPE_URL: &'static str = "/tendermint.types.Header";
}

#[cfg(feature = "ethabi")]
impl crate::EthAbi for Header {
    type EthAbi = contracts::glue::TendermintTypesHeaderData;
}

#[cfg(feature = "ethabi")]
impl From<Header> for contracts::glue::TendermintTypesHeaderData {
    fn from(value: Header) -> Self {
        Self {
            version: value.version.into(),
            chain_id: value.chain_id,
            height: value.height.into(),
            time: value.time.into(),
            last_block_id: value.last_block_id.into(),
            last_commit_hash: value.last_commit_hash.into(),
            data_hash: value.data_hash.into(),
            validators_hash: value.validators_hash.into(),
            next_validators_hash: value.next_validators_hash.into(),
            consensus_hash: value.consensus_hash.into(),
            app_hash: value.app_hash.into(),
            last_results_hash: value.last_results_hash.into(),
            evidence_hash: value.evidence_hash.into(),
            proposer_address: value.proposer_address.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
#[derive(Debug)]
pub enum TryFromEthAbiHeaderError {
    LastBlockId(crate::TryFromEthAbiErrorOf<BlockId>),
    Height(BoundedIntError<i64>),
    Timestamp(crate::TryFromEthAbiErrorOf<Timestamp>),
    LastCommitHash(InvalidLength),
    DataHash(InvalidLength),
    ValidatorsHash(InvalidLength),
    NextValidatorsHash(InvalidLength),
    ConsensusHash(InvalidLength),
    AppHash(InvalidLength),
    LastResultsHash(InvalidLength),
    EvidenceHash(InvalidLength),
    ProposerAddress(InvalidLength),
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::TendermintTypesHeaderData> for Header {
    type Error = TryFromEthAbiHeaderError;

    fn try_from(value: contracts::glue::TendermintTypesHeaderData) -> Result<Self, Self::Error> {
        Ok(Self {
            version: value.version.into(),
            chain_id: value.chain_id,
            height: value
                .height
                .try_into()
                .map_err(TryFromEthAbiHeaderError::Height)?,
            time: value
                .time
                .try_into()
                .map_err(TryFromEthAbiHeaderError::Timestamp)?,
            last_block_id: value
                .last_block_id
                .try_into()
                .map_err(TryFromEthAbiHeaderError::LastBlockId)?,
            last_commit_hash: value
                .last_commit_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::LastCommitHash)?,
            data_hash: value
                .data_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::DataHash)?,
            validators_hash: value
                .validators_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::ValidatorsHash)?,
            next_validators_hash: value
                .next_validators_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::NextValidatorsHash)?,
            consensus_hash: value
                .consensus_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::ConsensusHash)?,
            app_hash: value
                .app_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::AppHash)?,
            last_results_hash: value
                .last_results_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::LastResultsHash)?,
            evidence_hash: value
                .evidence_hash
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::EvidenceHash)?,
            proposer_address: value
                .proposer_address
                .to_vec()
                .try_into()
                .map_err(TryFromEthAbiHeaderError::ProposerAddress)?,
        })
    }
}
