pub mod hash_op;
pub mod inner_op;
pub mod leaf_op;
pub mod length_op;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ValidateIavlOpsError {
    #[error("height ({height}) is smaller than the min. height ({min_height})")]
    HeightTooShort { height: i64, min_height: i32 },
    #[error("size ({0}) is expected to be non-negative integer")]
    NegativeSize(i64),
    #[error("version ({0}) is expected to be non-negative integer")]
    NegativeVersion(i64),
    #[error("min. height cannot be negative")]
    NegativeMinHeight(i32),
    #[error("prost decode ({0:?})")]
    Decode(prost::DecodeError),
}

pub(crate) fn read_varint<'a>(mut buffer: &'a [u8]) -> Result<(&'a [u8], i64), prost::DecodeError> {
    let ux = prost::encoding::decode_varint(&mut buffer)?;
    let mut x = (ux >> 1) as i64;
    if ux & 1 != 0 {
        x = !x;
    }
    Ok((buffer, x))
}

pub(crate) fn validate_iavl_ops(
    prefix: &[u8],
    min_height: i32,
) -> Result<usize, ValidateIavlOpsError> {
    let mut prefix = prefix.to_vec();

    if min_height < 0 {
        return Err(ValidateIavlOpsError::NegativeMinHeight(min_height));
    }

    let (mut buffer, height) = read_varint(&mut prefix).map_err(ValidateIavlOpsError::Decode)?;
    if height < min_height as i64 {
        return Err(ValidateIavlOpsError::HeightTooShort { height, min_height });
    }

    let (mut buffer, size) = read_varint(&mut buffer).map_err(ValidateIavlOpsError::Decode)?;
    if size < 0 {
        return Err(ValidateIavlOpsError::NegativeSize(size));
    }

    let (buffer, version) = read_varint(&mut buffer).map_err(ValidateIavlOpsError::Decode)?;
    if version < 0 {
        return Err(ValidateIavlOpsError::NegativeVersion(size));
    }

    Ok(buffer.len())
}
