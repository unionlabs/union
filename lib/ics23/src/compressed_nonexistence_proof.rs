use unionlabs::cosmos::ics23::{
    compressed_non_existence_proof::CompressedNonExistenceProof, inner_op::InnerOp,
    non_existence_proof::NonExistenceProof,
};

use super::compressed_existence_proof;

pub fn decompress(
    compressed_nonexistence_proof: CompressedNonExistenceProof,
    lookup: &Vec<InnerOp>,
) -> NonExistenceProof {
    NonExistenceProof {
        key: compressed_nonexistence_proof.key,
        left: compressed_existence_proof::decompress(compressed_nonexistence_proof.left, lookup),
        right: compressed_existence_proof::decompress(compressed_nonexistence_proof.right, lookup),
    }
}
