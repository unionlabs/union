use unionlabs::cosmos::ics23::{
    compressed_existence_proof::CompressedExistenceProof, existence_proof::ExistenceProof,
    inner_op::InnerOp,
};

// TODO(aeryz): Should this function be fallible? We do unsafe indexing here
// by assuming that the indices in the path will always be within the bounds.
// Is this a correct assumption?
pub fn decompress(
    compressed_existence_proof: CompressedExistenceProof,
    lookup: &Vec<InnerOp>,
) -> ExistenceProof {
    ExistenceProof {
        key: compressed_existence_proof.key,
        value: compressed_existence_proof.value,
        leaf: compressed_existence_proof.leaf,
        path: compressed_existence_proof
            .path
            .iter()
            .map(|i| lookup[i.inner() as usize].clone())
            .collect(),
    }
}
