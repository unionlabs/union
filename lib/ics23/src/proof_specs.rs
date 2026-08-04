use std::borrow::Cow;

use unionlabs::{
    bounded::BoundedUsize,
    cosmos::ics23::{
        hash_op::HashOp,
        inner_spec::{InnerSpec, PositiveI32AsUsize},
        leaf_op::LeafOp,
        length_op::LengthOp,
        proof_spec::ProofSpec,
    },
    primitives::Bytes,
    result_unwrap,
};

pub const IAVL_PROOF_SPEC: ProofSpec = ProofSpec {
    leaf_spec: LeafOp {
        hash: HashOp::Sha256,
        prehash_key: HashOp::NoHash,
        prehash_value: HashOp::Sha256,
        length: LengthOp::VarProto,
        prefix: Bytes::new_static(&[0]),
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
        empty_child: Bytes::new_static(&[]),
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
        prefix: Bytes::new_static(&[0]),
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
        empty_child: Bytes::new_static(&[]),
        hash: HashOp::Sha256,
    },
    max_depth: None,
    min_depth: None,
    prehash_key_before_comparison: false,
};

pub const BPTREE_PROOF_SPEC: ProofSpec = ProofSpec {
    leaf_spec: LeafOp {
        hash: HashOp::Sha256,
        prehash_key: HashOp::NoHash,
        prehash_value: HashOp::Sha256,
        length: LengthOp::VarProto,
        prefix: Bytes::new_static(&[0]),
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
        empty_child: Bytes::new_static(&[
            0xdb, 0xc1, 0xb4, 0xc9, 0x00, 0xff, 0xe4, 0x8d, 0x57, 0x5b, 0x5d, 0xa5, 0xc6, 0x38,
            0x04, 0x01, 0x25, 0xf6, 0x5d, 0xb0, 0xfe, 0x3e, 0x24, 0x49, 0x4b, 0x76, 0xea, 0x98,
            0x64, 0x57, 0xd9, 0x86,
        ]),
        hash: HashOp::Sha256,
    },
    max_depth: Some(result_unwrap!(
        BoundedUsize::<1, { i32::MAX as usize }>::new_const(60)
    )),
    min_depth: Some(result_unwrap!(
        BoundedUsize::<1, { i32::MAX as usize }>::new_const(5)
    )),
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
