// use prost::Message;
// use protos::google::protobuf::{BytesValue, Int64Value, StringValue};
// use rs_merkle::{algorithms::Sha256, Hasher};
use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    google::protobuf::timestamp::Timestamp,
    hash::{H160, H256},
};

use crate::{types::block_id::BlockId, version::consensus::Consensus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    /// basic block info
    pub version: Consensus,
    pub chain_id: String,
    #[serde(with = "::serde_utils::string")]
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
    pub proposer_address: H160,
}

impl Header {
    #[cfg(all(feature = "proto", feature = "hash"))]
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn calculate_merkle_root(&self) -> Option<H256> {
        use prost::Message;
        use protos::google::protobuf::{BytesValue, Int64Value, StringValue};
        use sha2::{digest::FixedOutput, Digest, Sha256};

        const LEAF_PREFIX: u8 = 0;
        const INNER_PREFIX: u8 = 1;

        fn do_hash(prefix: u8, v: impl AsRef<[u8]>) -> [u8; 32] {
            Sha256::new()
                .chain_update([prefix])
                .chain_update(v)
                .finalize_fixed()
                .into()
        }

        fn leaf_hash(v: Vec<u8>) -> [u8; 32] {
            do_hash(LEAF_PREFIX, v)
        }

        fn inner_hash(l: impl AsRef<[u8]>, r: impl AsRef<[u8]>) -> [u8; 32] {
            Sha256::new()
                .chain_update([INNER_PREFIX])
                .chain_update(l)
                .chain_update(r)
                .finalize_fixed()
                .into()
        }

        let header: protos::tendermint::types::Header = self.clone().into();

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
                    value: header.last_commit_hash.into(),
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.data_hash.into(),
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.validators_hash.into(),
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.next_validators_hash.into(),
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.consensus_hash.into(),
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.app_hash.into(),
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.last_results_hash.into(),
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.evidence_hash.into(),
                }
                .encode_to_vec(),
            ),
            leaf_hash(
                BytesValue {
                    value: header.proposer_address.into(),
                }
                .encode_to_vec(),
            ),
        ];

        leaves[0] = inner_hash(leaves[0], leaves[1]);
        leaves[1] = inner_hash(leaves[2], leaves[3]);
        leaves[2] = inner_hash(leaves[4], leaves[5]);
        leaves[3] = inner_hash(leaves[6], leaves[7]);
        leaves[4] = inner_hash(leaves[8], leaves[9]);
        leaves[5] = inner_hash(leaves[10], leaves[11]);
        leaves[6] = inner_hash(leaves[12], leaves[13]);

        leaves[0] = inner_hash(leaves[0], leaves[1]);
        leaves[1] = inner_hash(leaves[2], leaves[3]);
        leaves[2] = inner_hash(leaves[4], leaves[5]);
        leaves[3] = leaves[6];

        leaves[0] = inner_hash(leaves[0], leaves[1]);
        leaves[1] = inner_hash(leaves[2], leaves[3]);

        let root = inner_hash(leaves[0], leaves[1]);

        Some(H256::new(root))
    }
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        bounded::BoundedIntError,
        errors::{InvalidLength, MissingField},
        google::protobuf::timestamp::TryFromTimestampError,
        required,
    };

    use crate::types::{block_id, header::Header};

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

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid last block id")]
        LastBlockId(#[source] block_id::proto::Error),
        #[error("invalid height")]
        Height(#[source] BoundedIntError<i64>),
        #[error("invalid timestamp")]
        Timestamp(#[source] TryFromTimestampError),
        #[error("invalid last commit hash")]
        LastCommitHash(#[source] InvalidLength),
        #[error("invalid data hash")]
        DataHash(#[source] InvalidLength),
        #[error("invalid validators hash")]
        ValidatorsHash(#[source] InvalidLength),
        #[error("invalid next validators hash")]
        NextValidatorsHash(#[source] InvalidLength),
        #[error("invalid consensus hash")]
        ConsensusHash(#[source] InvalidLength),
        #[error("invalid app hash")]
        AppHash(#[source] InvalidLength),
        #[error("invalid last results hash")]
        LastResultsHash(#[source] InvalidLength),
        #[error("invalid evidence hash")]
        EvidenceHash(#[source] InvalidLength),
        #[error("invalid proposer address")]
        ProposerAddress(#[source] InvalidLength),
    }

    impl TryFrom<protos::tendermint::types::Header> for Header {
        type Error = Error;

        fn try_from(value: protos::tendermint::types::Header) -> Result<Self, Self::Error> {
            Ok(Self {
                version: required!(value.version)?.into(),
                chain_id: value.chain_id,
                height: value.height.try_into().map_err(Error::Height)?,
                time: required!(value.time)?
                    .try_into()
                    .map_err(Error::Timestamp)?,
                last_block_id: required!(value.last_block_id)?
                    .try_into()
                    .map_err(Error::LastBlockId)?,
                last_commit_hash: value
                    .last_commit_hash
                    .try_into()
                    .map_err(Error::LastCommitHash)?,
                data_hash: value.data_hash.try_into().map_err(Error::DataHash)?,
                validators_hash: value
                    .validators_hash
                    .try_into()
                    .map_err(Error::ValidatorsHash)?,
                next_validators_hash: value
                    .next_validators_hash
                    .try_into()
                    .map_err(Error::NextValidatorsHash)?,
                consensus_hash: value
                    .consensus_hash
                    .try_into()
                    .map_err(Error::ConsensusHash)?,
                app_hash: value.app_hash.try_into().map_err(Error::AppHash)?,
                last_results_hash: value
                    .last_results_hash
                    .try_into()
                    .map_err(Error::LastResultsHash)?,
                evidence_hash: value
                    .evidence_hash
                    .try_into()
                    .map_err(Error::EvidenceHash)?,
                proposer_address: value
                    .proposer_address
                    .try_into()
                    .map_err(Error::ProposerAddress)?,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_hash() {
        // https://stride-testnet-rpc.polkachu.com/block?hash=B1228B7D3D788854D73B082BFE307A9D12010550F369E9F9CFAD9DF9A1A13507
        let expected_merkle_root =
            hex_literal::hex!("B1228B7D3D788854D73B082BFE307A9D12010550F369E9F9CFAD9DF9A1A13507");
        let header: Header = serde_json::from_str::<protos::tendermint::types::Header>(
            r#"
{
  "version": {
    "block": "11"
  },
  "chain_id": "stride-internal-1",
  "height": "8777721",
  "time": "2024-10-24T14:02:35.29813191Z",
  "last_block_id": {
    "hash": "FD01D47BAC82C15CB5E1EAC46A75A9BB3C9BAC25822FB0D3E322DD9C674F0A3F",
    "parts": {
      "total": 1,
      "hash": "E0EE3204DEE193A24EC770580073CC3B0A67CA14FF74505D229C3843B144B1E1"
    }
  },
  "last_commit_hash": "1C5718183AFC33BF5D4852B2759EEE53D8ED17C27A59EED201C70CD4A328827A",
  "data_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
  "validators_hash": "5A73E37A4DC01CAD4CC0B16196826B548B6F476D1E62BD4E12EACCBA8A22D294",
  "next_validators_hash": "5A73E37A4DC01CAD4CC0B16196826B548B6F476D1E62BD4E12EACCBA8A22D294",
  "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
  "app_hash": "7B3339C0D3E2AF530F9B7A1ED8436A39423E9DC86845E9AAFB0E70B3C9A37E30",
  "last_results_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
  "evidence_hash": "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
  "proposer_address": "246F8FE3F3ADA28DAD4018E38AFD2CD28CCBDD95"
}
"#,
        )
        .unwrap()
        .try_into()
        .unwrap();

        assert_eq!(
            header.calculate_merkle_root().unwrap(),
            <H256>::new(expected_merkle_root)
        );
    }
}
