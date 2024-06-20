use std::borrow::Cow;

use unionlabs::cosmos::ics23::{
    hash_op::HashOp, leaf_op::LeafOp, length_op::LengthOp, proof_spec::ProofSpec,
};

use crate::{
    ops::{hash_op, hash_op::HashError, length_op, validate_iavl_ops, ValidateIavlOpsError},
    proof_specs::{self, IAVL_PROOF_SPEC},
};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SpecMismatchError {
    #[error("unexpected hash op ({0:?})")]
    UnexpectedHashOp(HashOp),
    #[error("unexpected prehash key ({0:?})")]
    UnexpectedPrehashKey(HashOp),
    #[error("unexpected prehash value ({0:?})")]
    UnexpectedPrehashValue(HashOp),
    #[error("unexpected length op ({0:?})")]
    UnexpectedLengthOp(LengthOp),
    #[error("bad prefix remaining {0} bytes after reading")]
    BadPrefix(usize),
    #[error(
        "prefix ({prefix}) is not the prefix of ({full})",
        prefix = serde_utils::to_hex(prefix),
        full = serde_utils::to_hex(full)
    )]
    PrefixMismatch { full: Vec<u8>, prefix: Vec<u8> },
    #[error("validate iavl ops ({0})")]
    ValidateIavlOps(ValidateIavlOpsError),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ApplyError {
    #[error("key needed")]
    KeyNeeded,
    #[error("value needed")]
    ValueNeeded,
    #[error("apply leaf ({0})")]
    LeafData(length_op::ApplyError),
    #[error(transparent)]
    Hash(#[from] HashError),
}

pub fn check_against_spec(leaf_op: &LeafOp, spec: &ProofSpec) -> Result<(), SpecMismatchError> {
    let leaf_spec = &spec.leaf_spec;

    if proof_specs::compatible(spec, &IAVL_PROOF_SPEC) {
        match validate_iavl_ops(&leaf_op.prefix, 0) {
            Ok(remaining_bytes) => {
                if remaining_bytes > 0 {
                    return Err(SpecMismatchError::BadPrefix(remaining_bytes));
                }
            }
            Err(e) => return Err(SpecMismatchError::ValidateIavlOps(e)),
        }
    }

    if leaf_op.hash != leaf_spec.hash {
        return Err(SpecMismatchError::UnexpectedHashOp(leaf_spec.hash));
    }

    if leaf_op.prehash_key != leaf_spec.prehash_key {
        return Err(SpecMismatchError::UnexpectedPrehashKey(
            leaf_spec.prehash_key,
        ));
    }

    if leaf_op.prehash_value != leaf_spec.prehash_value {
        return Err(SpecMismatchError::UnexpectedPrehashValue(
            leaf_spec.prehash_value,
        ));
    }

    if leaf_op.length != leaf_spec.length {
        return Err(SpecMismatchError::UnexpectedLengthOp(leaf_spec.length));
    }

    if !leaf_op.prefix.starts_with(&leaf_spec.prefix) {
        return Err(SpecMismatchError::PrefixMismatch {
            full: leaf_op.prefix.to_vec(),
            prefix: leaf_spec.prefix.to_vec(),
        });
    }

    Ok(())
}

/// Calculate the leaf hash given the key and value being proven
pub fn apply(leaf_op: &LeafOp, key: &[u8], value: &[u8]) -> Result<Vec<u8>, ApplyError> {
    if key.is_empty() {
        return Err(ApplyError::KeyNeeded);
    }

    if value.is_empty() {
        return Err(ApplyError::ValueNeeded);
    }

    let p_key = prepare_data(leaf_op, leaf_op.prehash_key, key)?;
    let p_value = prepare_data(leaf_op, leaf_op.prehash_value, value)?;

    let data = leaf_op
        .prefix
        .iter()
        .chain(p_key.iter())
        .chain(p_value.iter())
        .copied()
        .collect::<Vec<_>>();

    Ok(hash_op::do_hash(leaf_op.hash, &data)?)
}

fn prepare_data<'a>(
    leaf_op: &LeafOp,
    hash_op: HashOp,
    data: &'a [u8],
) -> Result<Cow<'a, [u8]>, ApplyError> {
    let hashed_data = if hash_op == HashOp::NoHash {
        Cow::Borrowed(data)
    } else {
        Cow::Owned(hash_op::do_hash(hash_op, data)?)
    };
    length_op::apply(&leaf_op.length, hashed_data).map_err(ApplyError::LeafData)
}
