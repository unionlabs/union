use std::borrow::Cow;

use unionlabs::cosmos::ics23::{
    hash_op::HashOp, inner_spec::InnerSpec, leaf_op::LeafOp, length_op::LengthOp,
    proof_spec::ProofSpec,
};

pub const IAVL_PROOF_SPEC: ProofSpec = ProofSpec {
    leaf_spec: LeafOp {
        hash: HashOp::Sha256,
        prehash_key: HashOp::NoHash,
        prehash_value: HashOp::Sha256,
        length: LengthOp::VarProto,
        prefix: Cow::Borrowed(&[0]),
    },
    inner_spec: InnerSpec {
        child_order: Cow::Borrowed(&[0, 1]),
        child_size: 33,
        min_prefix_length: 4,
        max_prefix_length: 12,
        empty_child: Cow::Borrowed(&[]),
        hash: HashOp::Sha256,
    },
    max_depth: 0,
    min_depth: 0,
};

pub const TENDERMINT_PROOF_SPEC: ProofSpec = ProofSpec {
    leaf_spec: LeafOp {
        hash: HashOp::Sha256,
        prehash_key: HashOp::NoHash,
        prehash_value: HashOp::Sha256,
        length: LengthOp::VarProto,
        prefix: Cow::Borrowed(&[0]),
    },
    inner_spec: InnerSpec {
        child_order: Cow::Borrowed(&[0, 1]),
        child_size: 32,
        min_prefix_length: 1,
        max_prefix_length: 1,
        empty_child: Cow::Borrowed(&[]),
        hash: HashOp::Sha256,
    },
    max_depth: 0,
    min_depth: 0,
};
