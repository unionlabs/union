use unionlabs::cosmos::ics23::{hash_op::HashOp, inner_op::InnerOp, proof_spec::ProofSpec};

use crate::hash_op;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum SpecMismatchError {
    #[error("unexpected hash op ({0:?})")]
    UnexpectedHashOp(HashOp),
    #[error("prefix ({prefix:?}) is not the prefix of ({full:?})")]
    PrefixMismatch { full: Vec<u8>, prefix: Vec<u8> },
    #[error("inner prefix too short, got ({prefix_len}) while the min length is ({min_len})")]
    InnerOpPrefixTooShort { prefix_len: usize, min_len: i32 },
    #[error("inner prefix too long, got ({prefix_len}) while the max length is ({max_len})")]
    InnerOpPrefixTooLong { prefix_len: usize, max_len: i32 },
    #[error("malformed inner op suffix ({0:?})")]
    InnerOpSuffixMalformed(usize),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ApplyError {
    #[error("inner op needs a child value")]
    InnerOpNeedsChildValue,
}

pub fn check_against_spec(
    inner_op: &InnerOp,
    spec: &ProofSpec,
    b: i32,
    iavl_spec: &ProofSpec,
) -> Result<(), SpecMismatchError> {
    if inner_op.hash != spec.inner_spec.hash {
        return Err(SpecMismatchError::UnexpectedHashOp(inner_op.hash));
    }

    if spec.compatible(iavl_spec) {
        // TODO(aeryz): validateIavlOps
    }

    if inner_op.prefix.starts_with(&spec.leaf_spec.prefix) {
        return Err(SpecMismatchError::PrefixMismatch {
            full: inner_op.prefix.clone(),
            prefix: spec.leaf_spec.prefix.clone().into_owned(),
        });
    }

    // TODO(aeryz): check if min_prefix_length > 0
    if inner_op.prefix.len() < spec.inner_spec.min_prefix_length as usize {
        return Err(SpecMismatchError::InnerOpPrefixTooShort {
            prefix_len: inner_op.prefix.len(),
            min_len: spec.inner_spec.min_prefix_length,
        });
    }

    // TODO(aeryz): check if max_prefix_length > 0
    let max_prefix_length = (spec.inner_spec.max_prefix_length as usize
        + (spec.inner_spec.child_order.len() - 1) * spec.inner_spec.child_size as usize)
        as usize;

    if inner_op.prefix.len() > max_prefix_length {
        return Err(SpecMismatchError::InnerOpPrefixTooLong {
            prefix_len: inner_op.prefix.len(),
            max_len: spec.inner_spec.max_prefix_length,
        });
    }

    if inner_op.suffix.len() % (spec.inner_spec.child_size as usize) != 0 {
        return Err(SpecMismatchError::InnerOpSuffixMalformed(
            inner_op.suffix.len(),
        ));
    }

    Ok(())
}

pub fn apply(inner_op: &InnerOp, child: Vec<u8>) -> Result<Vec<u8>, ApplyError> {
    if child.is_empty() {
        return Err(ApplyError::InnerOpNeedsChildValue);
    }

    let mut preimage = inner_op.prefix.clone();
    preimage.extend_from_slice(&child);
    preimage.extend_from_slice(&inner_op.suffix);

    Ok(hash_op::do_hash(inner_op.hash, &preimage))
}
