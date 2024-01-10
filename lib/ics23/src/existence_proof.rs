use unionlabs::cosmos::ics23::{existence_proof::ExistenceProof, proof_spec::ProofSpec};

use crate::ops::{inner_op, leaf_op};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum SpecMismatchError {
    #[error("leaf spec mismatch ({0})")]
    LeafSpecMismatch(leaf_op::SpecMismatchError),
    #[error("inner op spec mismatch ({0})")]
    InnerOpSpecMismatch(inner_op::SpecMismatchError),
    #[error("inner path depth too short, got ({path_len}) while the min depth is ({min_depth})")]
    InnerDepthTooShort { path_len: usize, min_depth: usize },
    #[error("inner path depth too long, got ({path_len}) while the max depth is ({max_depth})")]
    InnerDepthTooLong { path_len: usize, max_depth: usize },
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CalculateRootError {
    #[error("leaf op hash ({0})")]
    LeafOpHash(leaf_op::ApplyError),
    #[error("inner op hash ({0})")]
    InnerOpHash(inner_op::ApplyError),
    #[error("inner op hash does not match the spec")]
    InnerOpHashAndSpecMismatch,
}

pub fn check_against_spec(
    existence_proof: &ExistenceProof,
    spec: &ProofSpec,
) -> Result<(), SpecMismatchError> {
    leaf_op::check_against_spec(&existence_proof.leaf, spec)
        .map_err(SpecMismatchError::LeafSpecMismatch)?;

    if let Some(min_depth) = spec.min_depth {
        if existence_proof.path.len() < min_depth.inner() {
            return Err(SpecMismatchError::InnerDepthTooShort {
                path_len: existence_proof.path.len(),
                min_depth: min_depth.inner(),
            });
        }
    }

    if let Some(max_depth) = spec.max_depth {
        if existence_proof.path.len() < max_depth.inner() {
            return Err(SpecMismatchError::InnerDepthTooLong {
                path_len: existence_proof.path.len(),
                max_depth: max_depth.inner(),
            });
        }
    }

    for (index, inner) in existence_proof.path.iter().enumerate() {
        inner_op::check_against_spec(inner, spec, index)
            .map_err(SpecMismatchError::InnerOpSpecMismatch)?;
    }

    Ok(())
}

/// Calculate determines the root hash that matches the given proof.
/// You must validate the result is what you have in a header.
/// Returns error if the calculations cannot be performed.
pub fn calculate_root(existence_proof: &ExistenceProof) -> Result<Vec<u8>, CalculateRootError> {
    calculate(existence_proof, None)
}

/// Calculates the hash of the given proof, validating against the given spec if provided.
pub(crate) fn calculate(
    existence_proof: &ExistenceProof,
    spec: Option<&ProofSpec>,
) -> Result<Vec<u8>, CalculateRootError> {
    let leaf_hash = leaf_op::apply(
        &existence_proof.leaf,
        &existence_proof.key,
        &existence_proof.value,
    )
    .map_err(CalculateRootError::LeafOpHash)?;

    existence_proof
        .path
        .iter()
        .try_fold(leaf_hash, |res, step| {
            let leaf_hash = inner_op::apply(step, &res).map_err(CalculateRootError::InnerOpHash)?;

            if let Some(proof_spec) = spec {
                if leaf_hash.len() > proof_spec.inner_spec.child_size.inner()
                    // REVIEW: WHy is this >= 32 check here? Taken directly from https://github.com/cosmos/ics23/blob/master/go/proof.go#L140
                    && proof_spec.inner_spec.child_size.inner() >= 32
                {
                    return Err(CalculateRootError::InnerOpHashAndSpecMismatch);
                }
            }

            Ok(leaf_hash)
        })
}
