use unionlabs::cosmos::ics23::{existence_proof::ExistenceProof, proof_spec::ProofSpec};

use crate::{inner_op, leaf_op, proof_specs::IAVL_PROOF_SPEC};

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

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum VerifyError {
    #[error("spec mismatch ({0})")]
    SpecMismatch(SpecMismatchError),
    #[error("key and existence proof value doesn't match ({key:?}, {existence_proof_key:?})")]
    KeyAndExistenceProofKeyMismatch {
        key: Vec<u8>,
        existence_proof_key: Vec<u8>,
    },
    #[error(
        "value and existence proof value doesn't match ({value:?}, {existence_proof_value:?})"
    )]
    ValueAndExistenceProofValueMismatch {
        value: Vec<u8>,
        existence_proof_value: Vec<u8>,
    },
    #[error("root calculation ({0})")]
    RootCalculation(CalculateRootError),
    #[error("calculated and given root doesn't match ({calculated_root:?}, {given_root:?})")]
    CalculatedAndGivenRootMismatch {
        calculated_root: Vec<u8>,
        given_root: Vec<u8>,
    },
}

pub fn check_against_spec(
    existence_proof: &ExistenceProof,
    spec: &ProofSpec,
    iavl_spec: &ProofSpec,
) -> Result<(), SpecMismatchError> {
    leaf_op::check_against_spec(&existence_proof.leaf, spec, iavl_spec)
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
        inner_op::check_against_spec(inner, spec, index as i32 + 1, iavl_spec)
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

fn calculate(
    existence_proof: &ExistenceProof,
    spec: Option<&ProofSpec>,
) -> Result<Vec<u8>, CalculateRootError> {
    let mut res = leaf_op::apply(
        &existence_proof.leaf,
        &existence_proof.key,
        &existence_proof.value,
    )
    .map_err(CalculateRootError::LeafOpHash)?;

    for step in &existence_proof.path {
        res = inner_op::apply(step, res).map_err(CalculateRootError::InnerOpHash)?;

        if let Some(proof_spec) = spec {
            if res.len() > proof_spec.inner_spec.child_size as usize
                && proof_spec.inner_spec.child_size >= 32
            {
                return Err(CalculateRootError::InnerOpHashAndSpecMismatch);
            }
        }
    }

    Ok(res)
}

/// Verify does all checks to ensure this proof proves this key, value -> root
/// and matches the spec.
pub fn verify(
    existence_proof: &ExistenceProof,
    spec: &ProofSpec,
    root: &[u8],
    key: &[u8],
    value: &[u8],
) -> Result<(), VerifyError> {
    check_against_spec(existence_proof, spec, &IAVL_PROOF_SPEC)
        .map_err(VerifyError::SpecMismatch)?;

    if key != existence_proof.key {
        return Err(VerifyError::KeyAndExistenceProofKeyMismatch {
            key: key.into(),
            existence_proof_key: existence_proof.key.clone(),
        });
    }

    if value != existence_proof.value {
        return Err(VerifyError::ValueAndExistenceProofValueMismatch {
            value: value.into(),
            existence_proof_value: existence_proof.value.clone(),
        });
    }

    let calc = calculate(existence_proof, Some(spec)).map_err(VerifyError::RootCalculation)?;

    if root != calc {
        return Err(VerifyError::CalculatedAndGivenRootMismatch {
            calculated_root: calc,
            given_root: root.into(),
        });
    }

    Ok(())
}
