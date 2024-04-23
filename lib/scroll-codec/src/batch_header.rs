use enumorph::Enumorph;
use sha3::{Digest, Keccak256};
use unionlabs::{
    errors::{ExpectedLength, InvalidLength},
    hash::H256,
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
    pub(crate) fn decode(bz: impl AsRef<[u8]>) -> Result<Self, BatchHeaderDecodeError> {
        match bz.as_ref().first() {
            Some(&BatchHeaderV0::VERSION) => Ok(BatchHeaderV0::decode(bz)?.into()),
            Some(&BatchHeaderV1::VERSION) => Ok(BatchHeaderV1::decode(bz)?.into()),
            Some(version) => Err(BatchHeaderDecodeError::UnknownVersion(*version)),
            None => Err(BatchHeaderDecodeError::EmptyBytes),
        }
    }

    #[must_use]
    pub(crate) fn batch_index(&self) -> u64 {
        match self {
            BatchHeader::V0(batch_header) => batch_header.batch_index,
            BatchHeader::V1(batch_header) => batch_header.batch_index,
        }
    }

    #[must_use]
    pub(crate) fn compute_batch_hash(&self) -> H256 {
        match self {
            BatchHeader::V0(batch_header) => batch_header.compute_batch_hash(),
            BatchHeader::V1(batch_header) => batch_header.compute_batch_hash(),
        }
    }

    #[must_use]
    pub(crate) fn total_l1_message_popped(&self) -> u64 {
        match self {
            BatchHeader::V0(batch_header) => batch_header.total_l1_message_popped,
            BatchHeader::V1(batch_header) => batch_header.total_l1_message_popped,
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
    pub skipped_l1_message_bitmap: Vec<H256>,
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

impl BatchHeaderV0 {
    const VERSION: u8 = 0;
    const FIXED_LENGTH: usize = 89;

    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    fn decode(bz: impl AsRef<[u8]>) -> Result<Self, BatchHeaderV0DecodeError> {
        let bz = bz.as_ref();

        let slice: [u8; Self::FIXED_LENGTH] = bz
            .get(..Self::FIXED_LENGTH)
            .ok_or(BatchHeaderV0DecodeError::LengthTooSmall(InvalidLength {
                expected: ExpectedLength::Gte(Self::FIXED_LENGTH),
                found: bz.len(),
            }))?
            .try_into()
            .expect("range bound is array len; qed;");

        let l1_message_popped = u64::from_be_bytes(slice.array_slice::<9, 8>());

        let expected_len = Self::FIXED_LENGTH
            + usize::try_from(((l1_message_popped + 255) / 256) * 32)
                .map_err(|_| BatchHeaderV0DecodeError::TooManyL1Messages(l1_message_popped))?;

        if bz.len() != expected_len {
            return Err(BatchHeaderV0DecodeError::IncorrectBitmapLength(
                InvalidLength {
                    expected: ExpectedLength::Exact(expected_len),
                    found: bz.len(),
                },
            ));
        }

        let version = slice.array_slice::<0, 1>()[0];
        debug_assert_eq!(version, Self::VERSION);

        assert!(bz[Self::FIXED_LENGTH..].len() % 32 == 0);

        Ok(Self {
            batch_index: u64::from_be_bytes(slice.array_slice::<1, 8>()),
            l1_message_popped,
            total_l1_message_popped: u64::from_be_bytes(slice.array_slice::<17, 8>()),
            data_hash: H256(slice.array_slice::<25, 32>()),
            parent_batch_hash: H256(slice.array_slice::<57, 32>()),
            skipped_l1_message_bitmap: bz[Self::FIXED_LENGTH..]
                .chunks(32)
                .map(|x| H256(x.try_into().expect("chunk size is 32; qed")))
                .collect(),
        })
    }

    /// <https://github.com/scroll-tech/scroll/blob/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/libraries/codec/BatchHeaderV0Codec.sol#L206>
    #[must_use]
    pub fn compute_batch_hash(&self) -> H256 {
        let mut hasher = Keccak256::new();

        hasher.update([Self::VERSION]);
        hasher.update(self.batch_index.to_be_bytes());
        hasher.update(self.l1_message_popped.to_be_bytes());
        hasher.update(self.total_l1_message_popped.to_be_bytes());
        hasher.update(self.data_hash);
        hasher.update(self.parent_batch_hash);

        for bitmap_limb in &self.skipped_l1_message_bitmap {
            hasher.update(bitmap_limb);
        }

        hasher.finalize().into()
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
    pub skipped_l1_message_bitmap: Vec<H256>,
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

impl BatchHeaderV1 {
    const VERSION: u8 = 1;
    const FIXED_LENGTH: usize = 121;

    fn decode(bz: impl AsRef<[u8]>) -> Result<Self, BatchHeaderV1DecodeError> {
        let bz = bz.as_ref();

        let slice: [u8; Self::FIXED_LENGTH] = bz
            .get(..Self::FIXED_LENGTH)
            .ok_or(BatchHeaderV1DecodeError::LengthTooSmall(InvalidLength {
                expected: ExpectedLength::Gte(Self::FIXED_LENGTH),
                found: bz.len(),
            }))?
            .try_into()
            .expect("range bound is array len; qed;");

        let l1_message_popped = u64::from_be_bytes(slice.array_slice::<9, 8>());

        let expected_len = Self::FIXED_LENGTH
            + usize::try_from(((l1_message_popped + 255) / 256) * 32)
                .map_err(|_| BatchHeaderV1DecodeError::TooManyL1Messages(l1_message_popped))?;

        if bz.len() != expected_len {
            return Err(BatchHeaderV1DecodeError::IncorrectBitmapLength(
                InvalidLength {
                    expected: ExpectedLength::Exact(expected_len),
                    found: bz.len(),
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
            skipped_l1_message_bitmap: bz[Self::FIXED_LENGTH..]
                .chunks(32)
                .map(|x| H256(x.try_into().expect("chunk size is 32; qed")))
                .collect(),
        })
    }

    /// <https://github.com/scroll-tech/scroll/blob/71f88b04f5a69196138c8cec63a75cf1f0ba2d99/contracts/src/libraries/codec/BatchHeaderV1Codec.sol#L224>
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

        for bitmap_limb in &self.skipped_l1_message_bitmap {
            hasher.update(bitmap_limb);
        }

        hasher.finalize().into()
    }
}
