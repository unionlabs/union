use unionlabs::cosmos::ics23::{
    batch_entry::BatchEntry, compressed_batch_entry::CompressedBatchEntry, inner_op::InnerOp,
};

use crate::{compressed_existence_proof, compressed_nonexistence_proof};

pub fn decompress(
    compressed_batch_entry: CompressedBatchEntry,
    lookup: Vec<InnerOp>,
) -> BatchEntry {
    match compressed_batch_entry {
        CompressedBatchEntry::Exist(exist) => {
            BatchEntry::Exist(compressed_existence_proof::decompress(exist, lookup))
        }
        CompressedBatchEntry::Nonexist(nonexist) => {
            BatchEntry::Nonexist(compressed_nonexistence_proof::decompress(nonexist, lookup))
        }
    }
}
