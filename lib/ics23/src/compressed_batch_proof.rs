use unionlabs::cosmos::ics23::{
    batch_proof::BatchProof, compressed_batch_proof::CompressedBatchProof,
};

use super::compressed_batch_entry;

pub fn decompress(compressed_batch_proof: CompressedBatchProof) -> BatchProof {
    let lookup = compressed_batch_proof.lookup_inners;

    BatchProof {
        entries: compressed_batch_proof
            .entries
            .into_iter()
            .map(|entry| compressed_batch_entry::decompress(entry, &lookup))
            .collect(),
    }
}
