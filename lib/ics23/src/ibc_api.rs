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
    // TODO(aeryz): print hex
    #[error("calculated root ({calculated:?}) does not match the given ({given:?}) value")]
    InvalidRoot { given: Vec<u8>, calculated: Vec<u8> },
    #[error("expected the size of proofs to be ({expected}), got ({got})")]
    InvalidProofsLength { expected: usize, got: usize },
    #[error("expected the size of key path to be ({expected}), got ({got})")]
    InvalidKeyPathLength { expected: usize, got: usize },
    #[error("proof type is expected to be `Exist`")]
    InvalidProofType,
    #[error("could not retrieve the key due to invalid indexing")]
    InvalidIndexing,
    #[error("nonexistence proof has empty left and right proof")]
    EmptyNonExistenceProof,
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

pub fn verify_non_membership(
    proof: MerkleProof,
    specs: &[&ProofSpec],
    consensus_root: &MerkleRoot,
    path: MerklePath,
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

    let CommitmentProof::Nonexist(nonexist) = &proof.proofs[0] else {
        return Err(VerifyMembershipError::InvalidProofType);
    };

    let subroot = if let Some(ep) = &nonexist.left {
        existence_proof::calculate_root(ep)
    } else if let Some(ep) = &nonexist.right {
        existence_proof::calculate_root(ep)
    } else {
        return Err(VerifyMembershipError::EmptyNonExistenceProof);
    }
    .map_err(VerifyMembershipError::RootCalculation)?;

    let key = path
        .key_path
        .get(path.key_path.len() - 1)
        .ok_or(VerifyMembershipError::InvalidIndexing)?;

    verify::verify_non_membership(specs[0], &subroot, &nonexist, key.as_bytes())
        .map_err(VerifyMembershipError::InnerVerification)?;

    verify_chained_membership_proof(
        consensus_root.hash.as_ref(),
        specs,
        proof.proofs,
        path,
        subroot,
        1,
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

            verify::verify_membership(
                specs[index],
                &subroot,
                existence_proof,
                key.as_bytes(),
                &value,
            )
            .map_err(VerifyMembershipError::InnerVerification)?;

            Ok(subroot)
        })
        .and_then(|value| {
            if value.as_slice() == root {
                Ok(())
            } else {
                Err(VerifyMembershipError::InvalidRoot {
                    given: value,
                    calculated: root.into(),
                })
            }
        })
}
