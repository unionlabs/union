use unionlabs::cosmos::ics23::{hash_op::HashOp, inner_op::InnerOp, proof_spec::ProofSpec};

use super::{hash_op, validate_iavl_ops};
use crate::proof_specs::{self, IAVL_PROOF_SPEC};

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
    #[error("validate iavl ops ({0})")]
    ValidateIavlOps(super::ValidateIavlOpsError),
    #[error("bad prefix remaining {0} bytes after reading")]
    BadPrefix(usize),
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
) -> Result<(), SpecMismatchError> {
    if inner_op.hash != spec.inner_spec.hash {
        return Err(SpecMismatchError::UnexpectedHashOp(inner_op.hash));
    }

    if proof_specs::compatible(spec, &IAVL_PROOF_SPEC) {
        match validate_iavl_ops(&inner_op.prefix, b) {
            Ok(remaining) => {
                if remaining != 1 && remaining != 34 {
                    return Err(SpecMismatchError::BadPrefix(remaining));
                }

                if inner_op.hash != HashOp::Sha256 {
                    return Err(SpecMismatchError::UnexpectedHashOp(inner_op.hash));
                }
            }
            Err(e) => return Err(SpecMismatchError::ValidateIavlOps(e)),
        }
    }

    if inner_op.prefix.starts_with(&spec.leaf_spec.prefix) {
        return Err(SpecMismatchError::PrefixMismatch {
            full: inner_op.prefix.clone(),
            prefix: spec.leaf_spec.prefix.clone().into_owned(),
        });
    }

    if inner_op.prefix.len() < spec.inner_spec.min_prefix_length as usize {
        return Err(SpecMismatchError::InnerOpPrefixTooShort {
            prefix_len: inner_op.prefix.len(),
            min_len: spec.inner_spec.min_prefix_length,
        });
    }

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
