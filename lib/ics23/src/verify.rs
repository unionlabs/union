use unionlabs::cosmos::ics23::{
    batch_entry::BatchEntry, commitment_proof::CommitmentProof, existence_proof::ExistenceProof,
    proof_spec::ProofSpec,
};

use crate::{commitment_proof, existence_proof};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum VerifyMembershipError {
    #[error("existence proof verification failed, ({0})")]
    ExistenceProofVerify(existence_proof::VerifyError),
    #[error("proof does not exist")]
    ProofDoesNotExist,
}

pub fn verify_membership(
    spec: &ProofSpec,
    root: &[u8],
    proof: CommitmentProof,
    key: &[u8],
    value: &[u8],
) -> Result<(), VerifyMembershipError> {
    let proof = commitment_proof::decompress(proof);

    if let Some(ep) = get_exist_proof_for_key(proof, key) {
        existence_proof::verify(&ep, spec, root, key, value)
            .map_err(VerifyMembershipError::ExistenceProofVerify)
    } else {
        Err(VerifyMembershipError::ProofDoesNotExist)
    }
}

fn get_exist_proof_for_key(proof: CommitmentProof, key: &[u8]) -> Option<ExistenceProof> {
    match proof {
        CommitmentProof::Exist(exist) => {
            if exist.key.as_slice() == key {
                return Some(exist);
            }

            None
        }
        CommitmentProof::Batch(batch) => {
            for sub in batch.entries {
                match sub {
                    BatchEntry::Exist(exist) => {
                        if exist.key.as_slice() == key {
                            return Some(exist);
                        }
                    }
                    _ => {}
                }
            }

            None
        }
        _ => None,
    }
}
