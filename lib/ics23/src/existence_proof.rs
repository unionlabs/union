use unionlabs::cosmos::ics23::{existence_proof::ExistenceProof, proof_spec::ProofSpec};

use crate::{inner_op, leaf_op};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum SpecMismatchError {
    #[error("leaf spec mismatch ({0})")]
    LeafSpecMismatch(super::leaf_op::SpecMismatchError),
    #[error("inner op spec mismatch ({0})")]
    InnerOpSpecMismatch(super::inner_op::SpecMismatchError),
    #[error("inner path depth too short, got ({path_len}) while the min depth is ({min_depth})")]
    InnerDepthTooShort { path_len: usize, min_depth: i32 },
    #[error("inner path depth too long, got ({path_len}) while the max depth is ({max_depth})")]
    InnerDepthTooLong { path_len: usize, max_depth: i32 },
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CalculateRootError {
    #[error("leaf op hash ({0})")]
    LeafOpHash(super::leaf_op::ApplyError),
    #[error("inner op hash ({0})")]
    InnerOpHash(super::inner_op::ApplyError),
    #[error("inner op hash does not match the spec")]
    InnerOpHashAndSpecMismatch,
}

pub fn check_against_spec(
    existence_proof: &ExistenceProof,
    spec: &ProofSpec,
) -> Result<(), SpecMismatchError> {
    leaf_op::check_against_spec(&existence_proof.leaf, spec)
        .map_err(SpecMismatchError::LeafSpecMismatch)?;

    if spec.min_depth > 0 && existence_proof.path.len() < spec.min_depth as usize {
        return Err(SpecMismatchError::InnerDepthTooShort {
            path_len: existence_proof.path.len(),
            min_depth: spec.min_depth,
        });
    }
    if spec.max_depth > 0 && existence_proof.path.len() > spec.max_depth as usize {
        return Err(SpecMismatchError::InnerDepthTooLong {
            path_len: existence_proof.path.len(),
            max_depth: spec.max_depth,
        });
    }

    for (index, inner) in existence_proof.path.iter().enumerate() {
        inner_op::check_against_spec(inner, spec, index as i32 + 1)
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

pub(crate) fn calculate(
    existence_proof: &ExistenceProof,
    spec: Option<&ProofSpec>,
) -> Result<Vec<u8>, CalculateRootError> {
    let res = leaf_op::apply(
        &existence_proof.leaf,
        &existence_proof.key,
        &existence_proof.value,
    )
    .map_err(CalculateRootError::LeafOpHash)?;

    existence_proof.path.iter().try_fold(res, |res, step| {
        let res = inner_op::apply(step, res).map_err(CalculateRootError::InnerOpHash)?;

        if let Some(proof_spec) = spec {
            if res.len() > proof_spec.inner_spec.child_size as usize
                && proof_spec.inner_spec.child_size >= 32
            {
                return Err(CalculateRootError::InnerOpHashAndSpecMismatch);
            }
        }

        Ok(res)
    })
}
