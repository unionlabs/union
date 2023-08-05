use std::num::TryFromIntError;

use crate::{
    errors::{required, InvalidLength, MissingField},
    ethereum::{Address, H256},
    ibc::google::protobuf::timestamp::Timestamp,
    tendermint::{types::block_id::BlockId, version::consensus::Consensus},
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Clone, PartialEq)]
pub struct Header {
    /// basic block info
    pub version: Consensus,
    pub chain_id: String,
    pub height: u32,
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
    Height(TryFromIntError),
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
            version: required!(value.version, TryFromHeaderError)?.into(),
            chain_id: value.chain_id,
            height: value
                .height
                .try_into()
                .map_err(TryFromHeaderError::Height)?,
            time: required!(value.time, TryFromHeaderError)?.into(),
            last_block_id: required!(value.last_block_id, TryFromHeaderError)?
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
