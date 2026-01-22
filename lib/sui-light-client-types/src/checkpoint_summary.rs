use blake2::{Blake2b, Digest as _};
use unionlabs_primitives::{
    Bytes, FixedBytes,
    encoding::{Base64, Encoding as _},
};

use crate::{Digest, U64, crypto::AuthorityPublicKeyBytes};

pub type CheckpointSequenceNumber = u64;
pub type CheckpointTimestamp = u64;
pub type EpochId = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
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

#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
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

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
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
    pub epoch_commitments: Vec<CheckpointCommitment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum CheckpointCommitment {
    ECMHLiveObjectSetDigest(ECMHLiveObjectSetDigest),
    // Other commitment types (e.g. merkle roots) go here.
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ECMHLiveObjectSetDigest {
    pub digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum CheckpointContents {
    V1(CheckpointContentsV1),
    V2(CheckpointContentsV2),
}

impl CheckpointContents {
    #[cfg(feature = "serde")]
    pub fn digest(&self) -> Digest {
        use crate::fixed_bytes::SuiFixedBytes;

        let mut hasher = Blake2b::<typenum::U32>::new();
        hasher.update("CheckpointContents::");
        bcs::serialize_into(&mut hasher, self).unwrap();
        SuiFixedBytes(FixedBytes::new(hasher.finalize().into()))
    }
}

/// CheckpointContents are the transactions included in an upcoming checkpoint.
/// They must have already been causally ordered. Since the causal order algorithm
/// is the same among validators, we expect all honest validators to come up with
/// the same order for each checkpoint content.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct CheckpointContentsV1 {
    pub transactions: Vec<ExecutionDigests>,

    /// This field 'pins' user signatures for the checkpoint
    /// The length of this vector is same as length of transactions vector
    /// System transactions has empty signatures
    pub user_signatures: Vec<Vec<GenericSignature>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct CheckpointContentsV2 {
    pub transactions: Vec<CheckpointTransactionContents>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct CheckpointTransactionContents {
    pub digest: ExecutionDigests,

    /// Each signature is paired with the version of the AddressAliases object
    /// that was used to verify it. Signatures always appear here in the same
    /// order as the `required_signers` of the input `Transaction`.
    pub user_signatures: Vec<(GenericSignature, Option<u64>)>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ExecutionDigests {
    pub transaction: Digest,
    pub effects: Digest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
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
            Ok(Self::Signature(Bytes::new(data.0)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_contents_v2_deser_and_digest_correct() {
        // Downloaded from: https://checkpoints.testnet.sui.io/289011363.chk
        let checkpoint_data = r#"
            {"V2":{"transactions":[{"digest":{"transaction":"At3rd3ymVNKVH8BJbE78oodKrDSVovaXtNWWME9YmdK1","effects":"HTqPVCSuCHqU12SDkuXnCVFUExF9DTHHPUrr6u3M1T2B"},"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",null]]},{"digest":{"transaction":"FGXTrAyNDbXxvJdJTfo6zh85FnyDv5oDotnckHP1q4rK","effects":"4BkT6LA73Kdz7ZPLGBh2RY3do1c9ZCAAFVjEKpKvhVdL"},"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",null]]},{"digest":{"transaction":"7GSnoPibLLqcmJqXNqKZQUUmMuoEz2k3cyS99K37RMLS","effects":"35jKZx2KF4VrVbwVxvEPeBXJ18eB5u3vCBq1Cd8jyyHw"},"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",null]]},{"digest":{"transaction":"Di6Prjnqb3j3UeSXPbFWFotnY1fAm9KUuVaLPXqiojX","effects":"97SnWAJjU8amZ59zYDybkxQCmPZe8Vvhx72eJS1XFRsp"},"user_signatures":[["BQNMMTY5NDYwNzE4MjMzMzM5NzYzODAzMjI4MjYyMjkxNTA2NDU5MDM5NDU0NzkwNzQ3MTMwMjg1MzY1ODg1OTg0NzMxNTc2MTIzMjQ3Nkw4MjYwNjE1OTAzMTIzNzYzMzgyODU2ODc5ODIxMDMzNjM1MTQxNDc3NzUzMjc5MDY0OTM2NjQ5MDA3NzcyMzUxMzg0OTM2NjkwNjQ4ATEDAk0xNjYzODMzNzA0NzM1OTM5NDE4NDA4MTE0MzQ3MzA4Mzc2NzIwODg5MzcyNTcxMjQ3NzU4NjMzNzQ1OTUxOTg3MzMzMDMzNTY1MzIzOEs1NzkxNjk3NTYwODMwNDk0OTgxOTE3MTA2MjM4NzQ3NTMxODY1Mzg5ODQyNTU2OTk2NDc4MjU3NTI0ODc3MTEwMTIyODEyMTY0NjQCTDUyODQ3NjA3NTU3MjM3MTQwNTkxMTA1MTk2Mzk3NDgyMDg3NTkyMTk2NDUwMzM3MzU1NjE5ODY1NTY4NjA0ODk4NjM2MjE4MDU3MzZMNDAzNTg5NTAyMjg0NzY5MTIyMTY2NjUwOTI4NzI4MjY4NTU0MzIwNjg4Mzg3Nzk0ODY1OTU5OTE1MTcxNTg3NjQwNjAwMzQ3MDQyNgIBMQEwA00xNTk5NDA3MTY2NDU1MzUwODk4NjMxNDY0NjEyODY0MzIxNzE4MDU2MjE2ODczMDg3OTIyOTQ4NzUyNjYxNDQ5MjA4OTMyNDYzMTcxNUw0ODMzNTA3MzY4NjczMDA0NDUzMjE3NDM3OTU5MzA1NTg5OTk4NDE4MTIwNDUzMDk5NTY3Mzg0Nzg5ODY1MzAzMjI1OTQwMzA3NTQ2ATExeUpwYzNNaU9pSm9kSFJ3Y3pvdkwyRmpZMjkxYm5SekxtZHZiMmRzWlM1amIyMGlMQwFmZXlKaGJHY2lPaUpTVXpJMU5pSXNJbXRwWkNJNklqazFORFJrTUdabU1EVTVNR1l3TWpVek1ERTJORE5tTXpJM05XSm1OamczTnpZM05qVTRNaklpTENKMGVYQWlPaUpLVjFRaWZRTTIwMjg5ODc0MjMwODU4MzcxMzY3MDkyOTQ0OTcwMzQ2NTcwMDI3OTQ4MDU2NTAyNDczMjU4NTA4MzU0MTYyNDU2OTY3Nzc2ODk1ODAz3AMAAAAAAABhAIbAfX8AnBVre4a5Z2hX0eIYX5BWaFoNu0/GuuAP8IjWU7FBxyaIFu7RR6srnCukUdnsjgOEq3h2Fek55qW+Bgh0jHOh9YVcSecNCJaxWdkMW1lgJ0Uxf2r2jj2ECMMhqQ==",null]]},{"digest":{"transaction":"4ArJiW3d7YvEBpwLVgYs567FYRMZr5tjz41mzNaTp9LM","effects":"AsmjVowb4SXp8V7aFNfhNoVfnCxTeheDXDgAiATvf4Nv"},"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",null]]},{"digest":{"transaction":"5oJ8sGNSW4VjBDL8cAa3dos98XerLgeWykXxAB8AFrGR","effects":"CyZvsq84cbJPmUPoWLfDDrXgaZPTYPSAS79UNWJSnoiE"},"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",null]]},{"digest":{"transaction":"6vMXTd1DXH2G6ZvjdS3PNsA1wUisr5gE2SX3NUyv3Rxh","effects":"HLUf9PhkTxnKmTACvLhExYQW9AvJbroJtFzS12BSr1tn"},"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",null]]},{"digest":{"transaction":"7hB26TvBUGTNF56ek7TVNWUdivrSNTzW3X5D2zihreS","effects":"6ZWdA3zYeKJf9T4zek6zneARC1XUA1NhEmYukazPX9q2"},"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",null]]},{"digest":{"transaction":"7JHT6wsNiXEbxMovEYuuYHZ1BNjhhRD2SXYALgf9CQui","effects":"ECae8DB9RdNs3ada3xRxLxtT48K6bMYxswEB3e3KtLVq"},"user_signatures":[["AMfCJWIOGABSYUw+J5BqAdPuDZJi5T+GMVaD+mEvC1bOSbqW1JqAvtQPrfhVXqv7sElS9+koOKaWDT8gf4bo4AlDP7cDT8ulfrHjkspxPlVaCktKtkzm2sp9yAJP0iFICQ==",null]]},{"digest":{"transaction":"HxjcqZEBoCpTc1srMmaXagDiEZ1n9QCeaAxqbU9XpHA","effects":"C6cvZffLsp7F88iYRdaRthZ3Po3UbaMz2B3TA8s1ckWb"},"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",null]]}]}}
        "#;

        let checkpoint_contents: CheckpointContents =
            serde_json::from_str(checkpoint_data).unwrap();

        assert_eq!(
            checkpoint_contents.digest().0.as_ref(),
            hex_literal::hex!("840e3a1948a2b329fdc47744e8cc6d3fe2dc50ea3f5be603a7ce6ba36dfc2cb9")
        );
    }
}
