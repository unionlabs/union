use unionlabs::cosmos::ics23::non_existence_proof::NonExistenceProof;

use crate::existence_proof;

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

pub fn verify(
    non_existence_proof: &NonExistenceProof,
    spec: &ProofSpec,
    root: &[u8],
    key: &[u8],
) -> Result<(), VerifyError> {
    if let Some(left) = non_existence_proof.left {
        existence_proof::verify(&left, spec, root, &left.key, &left.value)?;
    }

    Ok(())
}
