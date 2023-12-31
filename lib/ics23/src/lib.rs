#![feature(error_in_core)]

use unionlabs::cosmos::ics23::{
    hash_op::HashOp, inner_spec::InnerSpec, leaf_op::LeafOp, length_op::LengthOp,
    proof_spec::ProofSpec,
};

pub mod commitment_proof;
pub mod compressed_batch_entry;
pub mod compressed_batch_proof;
pub mod compressed_existence_proof;
pub mod compressed_nonexistence_proof;
pub mod existence_proof;
pub mod hash_op;
pub mod inner_op;
pub mod leaf_op;
pub mod length_op;

pub fn iavl_spec() -> ProofSpec {
    ProofSpec {
        leaf_spec: LeafOp {
            hash: HashOp::Sha256,
            prehash_key: HashOp::NoHash,
            prehash_value: HashOp::Sha256,
            length: LengthOp::VarProto,
            prefix: vec![0],
        },
        inner_spec: InnerSpec {
            child_order: vec![0, 1],
            child_size: 33,
            min_prefix_length: 4,
            max_prefix_length: 12,
            empty_child: vec![],
            hash: HashOp::Sha256,
        },
        max_depth: 0,
        min_depth: 0,
    }
}

pub fn tendermint_proof_spec() -> ProofSpec {
    ProofSpec {
        leaf_spec: LeafOp {
            hash: HashOp::Sha256,
            prehash_key: HashOp::NoHash,
            prehash_value: HashOp::Sha256,
            length: LengthOp::VarProto,
            prefix: vec![0],
        },
        inner_spec: InnerSpec {
            child_order: vec![0, 1],
            child_size: 32,
            min_prefix_length: 1,
            max_prefix_length: 1,
            empty_child: vec![],
            hash: HashOp::Sha256,
        },
        max_depth: 0,
        min_depth: 0,
    }
}
