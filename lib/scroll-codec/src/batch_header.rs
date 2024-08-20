use sha3::{Digest, Keccak256};
use unionlabs::{
    errors::{ExpectedLength, InvalidLength},
    hash::{H256, H512},
    ByteArrayExt,
};

/// @dev Below is the encoding for `BatchHeader` V3, total 193 bytes.
/// ```text
///   * Field                   Bytes       Type        Index   Comments
///   * version                 1           uint8       0       The batch version
///   * batchIndex              8           uint64      1       The index of the batch
///   * l1MessagePopped         8           uint64      9       Number of L1 messages popped in the batch
///   * totalL1MessagePopped    8           uint64      17      Number of total L1 messages popped after the batch
///   * dataHash                32          bytes32     25      The data hash of the batch
///   * blobVersionedHash       32          bytes32     57      The versioned hash of the blob with this batch’s data
///   * parentBatchHash         32          bytes32     89      The parent batch hash
///   * lastBlockTimestamp      8           uint64      121     A bitmap to indicate which L1 messages are skipped in the batch
///   * blobDataProof           64          bytes64     129     The blob data proof: z (32), y (32)
/// ```
/// The codes for `version`, `batchIndex`, `l1MessagePopped`, `totalL1MessagePopped`, `dataHash` and `computeBatchHash`
/// are the same as `BatchHeaderV0Codec`. The codes for `blobVersionedHash` and `parentBatchHash` are the same as
/// `BatchHeaderV1Codec`. However, we won't reuse the codes since they are very simple. Reusing the codes will introduce
/// extra code jump in solidity, which increase gas costs.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchHeaderV3 {
    /// The index of the batch
    pub batch_index: u64,
    /// Number of L1 messages popped in the batch
    pub l1_message_popped: u64,
    /// Number of total L1 messages popped after the batch
    pub total_l1_message_popped: u64,
    /// The data hash of the batch
    pub data_hash: H256,
    /// The versioned hash of the blob with this batch’s data
    pub blob_versioned_hash: H256,
    /// The parent batch hash
    pub parent_batch_hash: H256,
    /// The timestamp of the last block included in this batch
    pub last_block_timestamp: u64,
    /// The blob data proof: z (32), y (32)
    pub blob_data_proof: H512,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BatchHeaderV3DecodeError {
    #[error("input length too small")]
    LengthTooSmall(#[source] InvalidLength),
}

impl BatchHeaderV3 {
    const VERSION: u8 = 3;
    const FIXED_LENGTH: usize = 193;

    /// Try to decode the batch header from the input stream.
    ///
    /// # Errors
    ///
    /// Fails if the length of the stream doesn't match the expected length.
    ///
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self, BatchHeaderV3DecodeError> {
        let bz = bz.as_ref();

        let slice: [u8; Self::FIXED_LENGTH] = bz.try_into().map_err(|_| {
            BatchHeaderV3DecodeError::LengthTooSmall(InvalidLength {
                expected: ExpectedLength::Exact(Self::FIXED_LENGTH),
                found: bz.len(),
            })
        })?;

        let version = slice.array_slice::<0, 1>()[0];
        debug_assert_eq!(version, Self::VERSION);

        Ok(Self {
            batch_index: u64::from_be_bytes(slice.array_slice::<1, 8>()),
            l1_message_popped: u64::from_be_bytes(slice.array_slice::<9, 8>()),
            total_l1_message_popped: u64::from_be_bytes(slice.array_slice::<17, 8>()),
            data_hash: H256(slice.array_slice::<25, 32>()),
            blob_versioned_hash: H256(slice.array_slice::<57, 32>()),
            parent_batch_hash: H256(slice.array_slice::<89, 32>()),
            last_block_timestamp: u64::from_be_bytes(slice.array_slice::<121, 8>()),
            blob_data_proof: H512(slice.array_slice::<129, 64>()),
        })
    }

    #[must_use]
    pub fn compute_batch_hash(&self) -> H256 {
        let mut hasher = Keccak256::new();

        hasher.update([Self::VERSION]);
        hasher.update(self.batch_index.to_be_bytes());
        hasher.update(self.l1_message_popped.to_be_bytes());
        hasher.update(self.total_l1_message_popped.to_be_bytes());
        hasher.update(self.data_hash);
        hasher.update(self.blob_versioned_hash);
        hasher.update(self.parent_batch_hash);
        hasher.update(self.last_block_timestamp.to_be_bytes());
        hasher.update(self.blob_data_proof);

        hasher.finalize().into()
    }
}
