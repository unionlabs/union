use blake2::{Blake2b, Digest as _};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use unionlabs_primitives::{
    encoding::{Base58, Base64, Encoding as _},
    Bytes,
};

use crate::{crypto::AuthorityPublicKeyBytes, digest::Digest, U64};

pub type CheckpointSequenceNumber = u64;
pub type CheckpointTimestamp = u64;
pub type EpochId = u64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointSummary {
    pub epoch: EpochId,
    pub sequence_number: CheckpointSequenceNumber,
    /// Total number of transactions committed since genesis, including those in this
    /// checkpoint.
    pub network_total_transactions: u64,
    pub content_digest: Digest,
    pub previous_digest: Option<Digest>,
    /// The running total gas costs of all transactions included in the current epoch so far
    /// until this checkpoint.
    pub epoch_rolling_gas_cost_summary: GasCostSummary,

    /// Timestamp of the checkpoint - number of milliseconds from the Unix epoch
    /// Checkpoint timestamps are monotonic, but not strongly monotonic - subsequent
    /// checkpoints can have same timestamp if they originate from the same underlining consensus commit
    pub timestamp_ms: CheckpointTimestamp,

    /// Commitments to checkpoint-specific state (e.g. txns in checkpoint, objects read/written in
    /// checkpoint).
    ///
    /// NOTE: This is not used in sui network atm and not useful to us as well. So we just ignore.
    pub checkpoint_commitments: Vec<()>,

    /// Present only on the final checkpoint of the epoch.
    pub end_of_epoch_data: Option<EndOfEpochData>,

    /// CheckpointSummary is not an evolvable structure - it must be readable by any version of the
    /// code. Therefore, in order to allow extensions to be added to CheckpointSummary, we allow
    /// opaque data to be added to checkpoints which can be deserialized based on the current
    /// protocol version.
    ///
    /// This is implemented with BCS-serialized `CheckpointVersionSpecificData`.
    pub version_specific_data: Vec<u8>,
}

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde_as]
#[serde(rename_all = "camelCase")]
pub struct GasCostSummary {
    /// Cost of computation/execution
    pub computation_cost: U64,
    /// Storage cost, it's the sum of all storage cost for all objects created or mutated.
    pub storage_cost: U64,
    /// The amount of storage cost refunded to the user for all objects deleted or mutated in the
    /// transaction.
    pub storage_rebate: U64,
    /// The fee for the rebate. The portion of the storage rebate kept by the system.
    pub non_refundable_storage_fee: U64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndOfEpochData {
    /// next_epoch_committee is `Some` if and only if the current checkpoint is
    /// the last checkpoint of an epoch.
    /// Therefore next_epoch_committee can be used to pick the last checkpoint of an epoch,
    /// which is often useful to get epoch level summary stats like total gas cost of an epoch,
    /// or the total number of transactions from genesis to the end of an epoch.
    /// The committee is stored as a vector of validator pub key and stake pairs. The vector
    /// should be sorted based on the Committee data structure.
    pub next_epoch_committee: Vec<(AuthorityPublicKeyBytes, U64)>,

    /// The protocol version that is in effect during the epoch that starts immediately after this
    /// checkpoint.
    pub next_epoch_protocol_version: U64,

    /// Commitments to epoch specific state (e.g. live object set)
    ///
    /// This is not used, so we ignore it
    pub epoch_commitments: Vec<()>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointContents {
    V1(CheckpointContentsV1),
}

impl CheckpointContents {
    pub fn digest(&self) -> Bytes<Base58> {
        let mut hasher = Blake2b::<typenum::U32>::new();
        hasher.update("CheckpointContents::");
        bcs::serialize_into(&mut hasher, self).unwrap();
        Bytes::new(hasher.finalize().to_vec())
    }
}

/// CheckpointContents are the transactions included in an upcoming checkpoint.
/// They must have already been causally ordered. Since the causal order algorithm
/// is the same among validators, we expect all honest validators to come up with
/// the same order for each checkpoint content.
#[derive(Clone, Debug, PartialEq, Serialize, Eq, Deserialize)]
pub struct CheckpointContentsV1 {
    pub transactions: Vec<ExecutionDigests>,

    /// This field 'pins' user signatures for the checkpoint
    /// The length of this vector is same as length of transactions vector
    /// System transactions has empty signatures
    pub user_signatures: Vec<Vec<GenericSignature>>,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Hash, Serialize, Deserialize, Debug)]
pub struct ExecutionDigests {
    pub transaction: Digest,
    pub effects: Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenericSignature {
    /// TODO(aeryz): this enum normally contains more field, see if we need the other fields
    Signature(Bytes<Base64>),
}

impl GenericSignature {
    fn inner(&self) -> &Bytes<Base64> {
        match self {
            Self::Signature(inner) => inner,
        }
    }
}

impl<'de> ::serde::Deserialize<'de> for GenericSignature {
    fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            #[derive(serde::Deserialize)]
            struct GenericSignature(String);
            let s = GenericSignature::deserialize(deserializer)?;
            Ok(Self::Signature(Bytes::new(
                Base64::decode(s.0.as_bytes()).map_err(serde::de::Error::custom)?,
            )))
        } else {
            #[derive(serde::Deserialize)]
            struct GenericSignature(Vec<u8>);

            let data = GenericSignature::deserialize(deserializer)?;
            // Self::from_bytes(&data.0).map_err(|e| Error::custom(e.to_string()))
            Ok(Self::Signature(Bytes::new(
                Base64::decode(data.0.as_slice()).map_err(serde::de::Error::custom)?,
            )))
        }
    }
}

impl ::serde::Serialize for GenericSignature {
    fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            #[derive(serde::Serialize)]
            struct GenericSignature(String);
            GenericSignature(format!("{}", self.inner())).serialize(serializer)
        } else {
            #[derive(serde::Serialize)]
            struct GenericSignature<'a>(&'a [u8]);
            GenericSignature(self.inner().as_ref()).serialize(serializer)
        }
    }
}
