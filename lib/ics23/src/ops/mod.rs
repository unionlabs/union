pub mod hash_op;
pub mod inner_op;
pub mod leaf_op;
pub mod length_op;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ValidateIavlOpsError {
    #[error("height ({height}) is smaller than the minimum height ({min_height})")]
    HeightTooShort { height: i64, min_height: i64 },
    #[error("min_height {min_height} is too big")]
    MinHeightTooBig { min_height: usize },
    #[error("size ({0}) is expected to be non-negative integer")]
    NegativeSize(i64),
    #[error("version ({0}) is expected to be non-negative integer")]
    NegativeVersion(i64),
    #[error(transparent)]
    Decode(#[from] prost::DecodeError),
}

// Read a varint from the provided buffer, returning the value and the remaining buffer.
pub(crate) fn read_varint(mut buffer: &[u8]) -> Result<(&[u8], i64), prost::DecodeError> {
    let ux: u64 = prost::encoding::decode_varint(&mut buffer)?;
    // reinterpret the bits of the u64 as an i64
    // reference implementation did some hacky bit fiddling but rustc is smart enough to optimize this to shr and neg
    #[allow(clippy::cast_possible_wrap)] // u64::MAX / 2 == i64::MAX
    let x = (ux / 2) as i64 * (if ux % 2 == 0 { 1 } else { -1 });

    Ok((buffer, x))
}

/// Validates the prefix against the iavl spec. Returns the remaining length of the input prefix buffer.
pub(crate) fn validate_iavl_ops(
    prefix: &[u8],
    min_height: usize,
) -> Result<usize, ValidateIavlOpsError> {
    let (buffer, height) = read_varint(prefix)?;
    match i64::try_from(min_height) {
        Ok(min_height) => {
            if height < min_height {
                return Err(ValidateIavlOpsError::HeightTooShort { height, min_height });
            }
        }
        Err(_) => {
            return Err(ValidateIavlOpsError::MinHeightTooBig { min_height });
        }
    };

    let (buffer, size) = read_varint(buffer)?;
    if size < 0 {
        return Err(ValidateIavlOpsError::NegativeSize(size));
    }

    let (buffer, version) = read_varint(buffer)?;
    if version < 0 {
        return Err(ValidateIavlOpsError::NegativeVersion(size));
    }

    Ok(buffer.len())
}
