use enumorph::Enumorph;

use crate::{
    errors::{ExpectedLength, InvalidLength},
    hash::H256,
    uint::U256,
    ByteArrayExt,
};

#[derive(Debug, Clone, PartialEq, Enumorph)]
pub enum BatchHeader {
    V0(BatchHeaderV0),
    V1(BatchHeaderV1),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BatchHeaderDecodeError {
    #[error("error decoding a v0 batch header")]
    V0(#[from] BatchHeaderV0DecodeError),
    #[error("error decoding a v1 batch header")]
    V1(#[from] BatchHeaderV1DecodeError),
    #[error("the provided bytes were empty")]
    EmptyBytes,
    #[error("unknown batch header version {0}")]
    UnknownVersion(u8),
}

impl BatchHeader {
    pub fn decode(bz: impl AsRef<[u8]>) -> Result<Self, BatchHeaderDecodeError> {
        match bz.as_ref().first() {
            Some(&BatchHeaderV0::VERSION) => Ok(BatchHeaderV0::decode(bz)?.into()),
            Some(&BatchHeaderV1::VERSION) => Ok(BatchHeaderV1::decode(bz)?.into()),
            Some(version) => Err(BatchHeaderDecodeError::UnknownVersion(*version)),
            None => Err(BatchHeaderDecodeError::EmptyBytes),
        }
    }
}

/// @dev Below is the encoding for `BatchHeader` V0, total 89 + ceil(l1MessagePopped / 256) * 32 bytes.
/// ```text
///   * Field                   Bytes       Type        Index   Comments
///   * version                 1           uint8       0       The batch version
///   * batchIndex              8           uint64      1       The index of the batch
///   * l1MessagePopped         8           uint64      9       Number of L1 messages popped in the batch
///   * totalL1MessagePopped    8           uint64      17      Number of total L1 messages popped after the batch
///   * dataHash                32          bytes32     25      The data hash of the batch
///   * parentBatchHash         32          bytes32     57      The parent batch hash
///   * skippedL1MessageBitmap  dynamic     uint256[]   89      A bitmap to indicate which L1 messages are skipped in the batch
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BatchHeaderV0 {
    /// The index of the batch
    pub batch_index: u64,
    /// Number of L1 messages popped in the batch
    pub l1_message_popped: u64,
    /// Number of total L1 messages popped after the batch
    pub total_l1_message_popped: u64,
    /// The data hash of the batch
    pub data_hash: H256,
    /// The parent batch hash
    pub parent_batch_hash: H256,
    /// A bitmap to indicate which L1 messages are skipped in the batch
    pub skipped_l1_message_bitmap: Vec<U256>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BatchHeaderV0DecodeError {
    #[error("input length too small")]
    LengthTooSmall(#[source] InvalidLength),
    #[error("incorrect bitmap length")]
    IncorrectBitmapLength(#[source] InvalidLength),
    #[error("l1 message count ({0}) is greater than usize::MAX ({})", usize::MAX)]
    TooManyL1Messages(u64),
}

const BATCH_HEADER_V0_FIXED_LENGTH: usize = 89;

impl BatchHeaderV0 {
    const VERSION: u8 = 0;

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    fn decode(bz: impl AsRef<[u8]>) -> Result<Self, BatchHeaderV0DecodeError> {
        let slice: [u8; BATCH_HEADER_V0_FIXED_LENGTH] = bz
            .as_ref()
            .get(..BATCH_HEADER_V0_FIXED_LENGTH)
            .ok_or_else(|| {
                BatchHeaderV0DecodeError::LengthTooSmall(InvalidLength {
                    expected: ExpectedLength::Gte(BATCH_HEADER_V0_FIXED_LENGTH),
                    found: bz.as_ref().len(),
                })
            })?
            .try_into()
            .expect("range bound is array len; qed;");

        let l1_message_popped = u64::from_be_bytes(slice.array_slice::<9, 8>());

        let expected_len = BATCH_HEADER_V0_FIXED_LENGTH
            + usize::try_from(((l1_message_popped + 255) / 256) * 32)
                .map_err(|_| BatchHeaderV0DecodeError::TooManyL1Messages(l1_message_popped))?;

        if bz.as_ref().len() != expected_len {
            return Err(BatchHeaderV0DecodeError::IncorrectBitmapLength(
                InvalidLength {
                    expected: ExpectedLength::Exact(expected_len),
                    found: bz.as_ref().len(),
                },
            ));
        }

        let version = slice.array_slice::<0, 1>()[0];
        debug_assert_eq!(version, Self::VERSION);

        Ok(Self {
            batch_index: u64::from_be_bytes(slice.array_slice::<1, 8>()),
            l1_message_popped,
            total_l1_message_popped: u64::from_be_bytes(slice.array_slice::<17, 8>()),
            data_hash: H256(slice.array_slice::<25, 32>()),
            parent_batch_hash: H256(slice.array_slice::<57, 32>()),
            skipped_l1_message_bitmap: bz
                .as_ref()
                .chunks(32)
                .map(U256::try_from_big_endian)
                .collect::<Result<_, _>>()
                .expect("chunk size is 32; qed"),
        })
    }
}

/// @dev Below is the encoding for `BatchHeader` V1, total 121 + ceil(l1MessagePopped / 256) * 32 bytes.
/// ```text
///   * Field                   Bytes       Type        Index   Comments
///   * version                 1           uint8       0       The batch version
///   * batchIndex              8           uint64      1       The index of the batch
///   * l1MessagePopped         8           uint64      9       Number of L1 messages popped in the batch
///   * totalL1MessagePopped    8           uint64      17      Number of total L1 messages popped after the batch
///   * dataHash                32          bytes32     25      The data hash of the batch
///   * blobVersionedHash       32          bytes32     57      The versioned hash of the blob with this batch’s data
///   * parentBatchHash         32          bytes32     89      The parent batch hash
///   * skippedL1MessageBitmap  dynamic     uint256[]   121     A bitmap to indicate which L1 messages are skipped in the batch
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct BatchHeaderV1 {
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
    /// A bitmap to indicate which L1 messages are skipped in the batch
    pub skipped_l1_message_bitmap: Vec<U256>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BatchHeaderV1DecodeError {
    #[error("input length too small")]
    LengthTooSmall(#[source] InvalidLength),
    #[error("incorrect bitmap length")]
    IncorrectBitmapLength(#[source] InvalidLength),
    #[error("l1 message count ({0}) is greater than usize::MAX ({})", usize::MAX)]
    TooManyL1Messages(u64),
}

const BATCH_HEADER_V1_FIXED_LENGTH: usize = 121;

impl BatchHeaderV1 {
    const VERSION: u8 = 1;

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    fn decode(bz: impl AsRef<[u8]>) -> Result<Self, BatchHeaderV1DecodeError> {
        let slice: [u8; BATCH_HEADER_V1_FIXED_LENGTH] = bz
            .as_ref()
            .get(..BATCH_HEADER_V1_FIXED_LENGTH)
            .ok_or_else(|| {
                BatchHeaderV1DecodeError::LengthTooSmall(InvalidLength {
                    expected: ExpectedLength::Gte(BATCH_HEADER_V1_FIXED_LENGTH),
                    found: bz.as_ref().len(),
                })
            })?
            .try_into()
            .expect("range bound is array len; qed;");

        let l1_message_popped = u64::from_be_bytes(slice.array_slice::<9, 8>());

        let expected_len = BATCH_HEADER_V1_FIXED_LENGTH
            + usize::try_from(((l1_message_popped + 255) / 256) * 32)
                .map_err(|_| BatchHeaderV1DecodeError::TooManyL1Messages(l1_message_popped))?;

        if bz.as_ref().len() != expected_len {
            return Err(BatchHeaderV1DecodeError::IncorrectBitmapLength(
                InvalidLength {
                    expected: ExpectedLength::Exact(expected_len),
                    found: bz.as_ref().len(),
                },
            ));
        }

        let version = slice.array_slice::<0, 1>()[0];
        debug_assert_eq!(version, Self::VERSION);

        Ok(Self {
            batch_index: u64::from_be_bytes(slice.array_slice::<1, 8>()),
            l1_message_popped,
            total_l1_message_popped: u64::from_be_bytes(slice.array_slice::<17, 8>()),
            data_hash: H256(slice.array_slice::<25, 32>()),
            blob_versioned_hash: H256(slice.array_slice::<57, 32>()),
            parent_batch_hash: H256(slice.array_slice::<89, 32>()),
            skipped_l1_message_bitmap: bz
                .as_ref()
                .chunks(32)
                .map(U256::try_from_big_endian)
                .collect::<Result<_, _>>()
                .expect("chunk size is 32; qed"),
        })
    }
}
