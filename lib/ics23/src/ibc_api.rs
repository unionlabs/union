use unionlabs::{
    cosmos::ics23::{commitment_proof::CommitmentProof, proof_spec::ProofSpec},
    ibc::core::commitment::{
        merkle_path::MerklePath, merkle_proof::MerkleProof, merkle_root::MerkleRoot,
    },
};

pub use crate::proof_specs::{IAVL_PROOF_SPEC, TENDERMINT_PROOF_SPEC};
use crate::{existence_proof, verify};

pub const SDK_SPECS: [&'static ProofSpec; 2] = [&IAVL_PROOF_SPEC, &TENDERMINT_PROOF_SPEC];

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum VerifyMembershipError {
    #[error("root calculation ({0})")]
    RootCalculation(existence_proof::CalculateRootError),
    #[error("{0}")]
    InnerVerification(verify::VerifyMembershipError),
    #[error("invalid root top level")] // TODO(aeryz): beautify
    InvalidRoot,
    #[error("expected the size of proofs to be ({expected}), got ({got})")]
    InvalidProofsLength { expected: usize, got: usize },
    #[error("expected the size of key path to be ({expected}), got ({got})")]
    InvalidKeyPathLength { expected: usize, got: usize },
    #[error("proof type is expected to be `Exist`")]
    InvalidProofType,
    #[error("could not retrieve the key due to invalid indexing")]
    InvalidIndexing,
}

pub fn verify_membership(
    proof: MerkleProof,
    specs: &[&ProofSpec],
    consensus_root: &MerkleRoot,
    path: MerklePath,
    value: Vec<u8>,
) -> Result<(), VerifyMembershipError> {
    if proof.proofs.len() != specs.len() {
        return Err(VerifyMembershipError::InvalidProofsLength {
            expected: specs.len(),
            got: proof.proofs.len(),
        });
    }

    if path.key_path.len() != specs.len() {
        return Err(VerifyMembershipError::InvalidKeyPathLength {
            expected: specs.len(),
            got: path.key_path.len(),
        });
    }

    verify_chained_membership_proof(
        consensus_root.hash.as_ref(),
        specs,
        proof.proofs,
        path,
        value,
        0,
    )
}

fn verify_chained_membership_proof(
    root: &[u8],
    specs: &[&ProofSpec],
    proofs: Vec<CommitmentProof>,
    keys: MerklePath,
    value: Vec<u8>,
    index: usize,
) -> Result<(), VerifyMembershipError> {
    proofs
        .into_iter()
        .skip(index)
        .enumerate()
        .try_fold(value, |value, (i, proof)| {
            let CommitmentProof::Exist(ref existence_proof) = proof else {
                return Err(VerifyMembershipError::InvalidProofType);
            };

            let subroot = existence_proof::calculate_root(existence_proof)
                .map_err(VerifyMembershipError::RootCalculation)?;

            let key = keys
                .key_path
                .get(keys.key_path.len() - 1 - i)
                .ok_or(VerifyMembershipError::InvalidIndexing)?;

            verify::verify_membership(specs[index], &subroot, proof, key.as_bytes(), &value)
                .map_err(VerifyMembershipError::InnerVerification)?;

            Ok(subroot)
        })
        .and_then(|value| {
            if value.as_slice() == root {
                Ok(())
            } else {
                Err(VerifyMembershipError::InvalidRoot)
            }
        })
}
