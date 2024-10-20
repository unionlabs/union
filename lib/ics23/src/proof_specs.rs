use std::borrow::Cow;

use unionlabs::{
    cosmos::ics23::{
        hash_op::HashOp,
        inner_spec::{InnerSpec, PositiveI32AsUsize},
        leaf_op::LeafOp,
        length_op::LengthOp,
        proof_spec::ProofSpec,
    },
    result_unwrap,
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
        child_order: Cow::Borrowed(
            const {
                &[
                    result_unwrap!(PositiveI32AsUsize::new_const(0)),
                    result_unwrap!(PositiveI32AsUsize::new_const(1)),
                ]
            },
        ),
        child_size: result_unwrap!(PositiveI32AsUsize::new_const(33)),
        min_prefix_length: result_unwrap!(PositiveI32AsUsize::new_const(4)),
        max_prefix_length: result_unwrap!(PositiveI32AsUsize::new_const(12)),
        empty_child: Cow::Borrowed(&[]),
        hash: HashOp::Sha256,
    },
    max_depth: None,
    min_depth: None,
    prehash_key_before_comparison: false,
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
        child_order: Cow::Borrowed(
            const {
                &[
                    result_unwrap!(PositiveI32AsUsize::new_const(0)),
                    result_unwrap!(PositiveI32AsUsize::new_const(1)),
                ]
            },
        ),
        child_size: result_unwrap!(PositiveI32AsUsize::new_const(32)),
        min_prefix_length: result_unwrap!(PositiveI32AsUsize::new_const(1)),
        max_prefix_length: result_unwrap!(PositiveI32AsUsize::new_const(1)),
        empty_child: Cow::Borrowed(&[]),
        hash: HashOp::Sha256,
    },
    max_depth: None,
    min_depth: None,
    prehash_key_before_comparison: false,
};

#[must_use]
pub fn compatible(lhs: &ProofSpec, rhs: &ProofSpec) -> bool {
    lhs.leaf_spec.hash == rhs.leaf_spec.hash
        && lhs.leaf_spec.prehash_key == rhs.leaf_spec.prehash_key
        && lhs.leaf_spec.prehash_value == rhs.leaf_spec.prehash_value
        && lhs.leaf_spec.length == rhs.leaf_spec.length
        && lhs.inner_spec.hash == rhs.inner_spec.hash
        && lhs.inner_spec.min_prefix_length == rhs.inner_spec.min_prefix_length
        && lhs.inner_spec.max_prefix_length == rhs.inner_spec.max_prefix_length
        && lhs.inner_spec.child_size == rhs.inner_spec.child_size
        && lhs.inner_spec.child_order.len() == rhs.inner_spec.child_order.len()
}
