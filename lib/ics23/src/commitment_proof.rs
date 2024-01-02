use unionlabs::cosmos::ics23::commitment_proof::CommitmentProof;

use crate::compressed_batch_proof;

pub fn decompress(commitment_proof: CommitmentProof) -> CommitmentProof {
    match commitment_proof {
        CommitmentProof::CompressedBatch(comp) => {
            CommitmentProof::Batch(compressed_batch_proof::decompress(comp))
        }
        proof => proof,
    }
}
