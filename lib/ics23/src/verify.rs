use unionlabs::cosmos::ics23::{
    existence_proof::ExistenceProof, inner_op::InnerOp, inner_spec::InnerSpec,
    non_existence_proof::NonExistenceProof, proof_spec::ProofSpec,
};

use crate::{
    existence_proof::{self, CalculateRootError, SpecMismatchError},
    hash_op::do_hash_or_noop,
};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum VerifyError {
    #[error("spec mismatch ({0})")]
    SpecMismatch(SpecMismatchError),
    #[error("key and existence proof value doesn't match ({key:?}, {existence_proof_key:?})")]
    KeyAndExistenceProofKeyMismatch {
        key: Vec<u8>,
        existence_proof_key: Vec<u8>,
    },
    #[error(
        "value and existence proof value doesn't match ({value:?}, {existence_proof_value:?})"
    )]
    ValueAndExistenceProofValueMismatch {
        value: Vec<u8>,
        existence_proof_value: Vec<u8>,
    },
    #[error("root calculation ({0})")]
    RootCalculation(CalculateRootError),
    #[error("calculated and given root doesn't match ({calculated_root:?}, {given_root:?})")]
    CalculatedAndGivenRootMismatch {
        calculated_root: Vec<u8>,
        given_root: Vec<u8>,
    },
    #[error("key is not left of right proof")]
    KeyIsNotLeftOfRightProof,
    #[error("key is not right of left proof")]
    KeyIsNotRightOfLeftProof,
    #[error("left proof missing, right proof must be left-most")]
    LeftProofMissing,
    #[error("right proof missing, left proof must be right-most")]
    RightProofMissing,
    #[error("both left and right proofs are missing")]
    BothProofsMissing,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum VerifyMembershipError {
    #[error("existence proof verification failed, ({0})")]
    ExistenceProofVerify(VerifyError),
    #[error("proof does not exist")]
    ProofDoesNotExist,
}

pub fn verify_non_membership(
    spec: &ProofSpec,
    root: &[u8],
    proof: &NonExistenceProof,
    key: &[u8],
) -> Result<(), VerifyMembershipError> {
    verify_non_existence(&proof, spec, root, key)
        .map_err(VerifyMembershipError::ExistenceProofVerify)
}

pub fn verify_membership(
    spec: &ProofSpec,
    root: &[u8],
    proof: &ExistenceProof,
    key: &[u8],
    value: &[u8],
) -> Result<(), VerifyMembershipError> {
    // TODO(aeryz): push proof.key and proof.value checks to here
    verify_existence_proof(proof, spec, root, key, value)
        .map_err(VerifyMembershipError::ExistenceProofVerify)
}

fn verify_non_existence(
    non_existence_proof: &NonExistenceProof,
    spec: &ProofSpec,
    root: &[u8],
    key: &[u8],
) -> Result<(), VerifyError> {
    let key_for_comparison = |spec: &ProofSpec, key: &[u8]| -> Vec<u8> {
        // TODO(aeryz): we don't have prehash_key_before_comparison, why?
        do_hash_or_noop(spec.leaf_spec.prehash_key, key)
    };

    let left_ops = |left: &ExistenceProof| -> Result<(), VerifyError> {
        verify_existence_proof(&left, spec, root, &left.key, &left.value)?;

        if key_for_comparison(spec, key) == key_for_comparison(spec, &left.key) {
            return Err(VerifyError::KeyIsNotRightOfLeftProof);
        }

        if !is_right_most(&spec.inner_spec, &left.path) {
            return Err(VerifyError::RightProofMissing);
        }

        Ok(())
    };

    let right_ops = |right: &ExistenceProof| -> Result<(), VerifyError> {
        verify_existence_proof(&right, spec, root, &right.key, &right.value)?;

        if key_for_comparison(spec, key) == key_for_comparison(spec, &right.key) {
            return Err(VerifyError::KeyIsNotLeftOfRightProof);
        }

        if !is_left_most(&spec.inner_spec, &right.path) {
            return Err(VerifyError::LeftProofMissing);
        }

        Ok(())
    };

    match (&non_existence_proof.left, &non_existence_proof.right) {
        (None, Some(right)) => right_ops(right)?,
        (Some(left), None) => left_ops(left)?,
        (Some(left), Some(right)) => {
            if !is_left_neighbor(&spec.inner_spec, &left.path, &right.path) {
                return Err(VerifyError::RightProofMissing);
            }
        }
        (None, None) => return Err(VerifyError::BothProofsMissing),
    }

    Ok(())
}

fn is_left_neighbor(spec: &InnerSpec, left: &[InnerOp], right: &[InnerOp]) -> bool {
    let Some((mut topleft, mut left)) = left.split_last() else {
        return false;
    };

    let Some((mut topright, mut right)) = right.split_last() else {
        return false;
    };

    while topleft.prefix == topright.prefix && topleft.suffix == topright.suffix {
        (topleft, left) = if let Some(l) = left.split_last() {
            l
        } else {
            return false;
        };

        (topright, right) = if let Some(r) = right.split_last() {
            r
        } else {
            return false;
        };
    }

    if !is_left_step(spec, topleft, topright)
        || !is_right_most(spec, left)
        || !is_left_most(spec, right)
    {
        return false;
    }

    true
}

fn is_left_step(spec: &InnerSpec, left: &InnerOp, right: &InnerOp) -> bool {
    let Ok(leftidx) = order_from_padding(spec, left) else {
        // TODO(aeryz)
        panic!("err")
    };

    let Ok(rightidx) = order_from_padding(spec, right) else {
        // TODO(aeryz)
        panic!("err")
    };

    return rightidx == leftidx + 1;
}

fn is_right_most(spec: &InnerSpec, path: &[InnerOp]) -> bool {
    let (min_prefix, max_prefix, suffix) = get_padding(spec, (spec.child_order.len() - 1) as i32);

    for step in path {
        if !has_padding(step, min_prefix, max_prefix, suffix)
            && !right_branches_are_empty(spec, step)
        {
            return false;
        }
    }

    true
}

fn is_left_most(spec: &InnerSpec, path: &[InnerOp]) -> bool {
    let (min_prefix, max_prefix, suffix) = get_padding(spec, 0);

    for step in path {
        if !has_padding(step, min_prefix, max_prefix, suffix)
            && !left_branches_are_empty(spec, step)
        {
            return false;
        }
    }

    true
}

fn right_branches_are_empty(spec: &InnerSpec, op: &InnerOp) -> bool {
    let Ok(idx) = order_from_padding(spec, op) else {
        return false;
    };

    let right_branches = (spec.child_order.len() as i32) - 1 - idx;
    if right_branches == 0 {
        return false;
    }

    if (op.suffix.len() as i32) != right_branches * spec.child_size {
        return false;
    }

    for i in 0..right_branches {
        let idx = get_position(&spec.child_order, i);
        let from = (idx * spec.child_size) as usize;
        if spec.empty_child != &op.suffix[from..from + (spec.child_size as usize)] {
            return false;
        }
    }

    true
}

fn left_branches_are_empty(spec: &InnerSpec, op: &InnerOp) -> bool {
    let Ok(left_branches) = order_from_padding(spec, op) else {
        return false;
    };

    if left_branches == 0 {
        return false;
    }

    let actual_prefix = (op.prefix.len() as i32) - left_branches * spec.child_size;
    if actual_prefix < 0 {
        return false;
    }

    for i in 0..left_branches {
        let idx = get_position(&spec.child_order, i);
        let from = (actual_prefix + idx * spec.child_size) as usize;
        if spec.empty_child != &op.prefix[from..from + (spec.child_size as usize)] {
            return false;
        }
    }

    true
}

fn order_from_padding(spec: &InnerSpec, inner: &InnerOp) -> Result<i32, ()> {
    let branch = (0..spec.child_order.len())
        .find(|&branch| {
            let (minp, maxp, suffix) = get_padding(spec, branch as i32);
            has_padding(inner, minp, maxp, suffix)
        })
        .map(|branch| branch as i32);

    branch.ok_or(())
}

fn has_padding(op: &InnerOp, min_prefix: i32, max_prefix: i32, suffix: i32) -> bool {
    if (op.prefix.len() as i32) < min_prefix || (op.prefix.len() as i32) > max_prefix {
        return false;
    }

    (op.suffix.len() as i32) == suffix
}

fn get_padding(spec: &InnerSpec, branch: i32) -> (i32, i32, i32) {
    let idx = get_position(&spec.child_order, branch);

    let prefix = idx * spec.child_size;
    let min_prefix = prefix + spec.min_prefix_length;
    let max_prefix = prefix + spec.max_prefix_length;

    let suffix = (spec.child_order.len() as i32 - 1 - idx) * spec.child_size;

    (min_prefix, max_prefix, suffix)
}

fn get_position(order: &[i32], branch: i32) -> i32 {
    if branch < 0 || branch as usize >= order.len() {
        // TODO(aeryz):
        panic!("invalid branch")
    }

    match order.iter().enumerate().find(|(_, &elem)| elem == branch) {
        Some((i, _)) => i as i32,
        None => panic!("branch not found"), // TODO(aeryz)
    }
}

/// Verify does all checks to ensure this proof proves this key, value -> root
/// and matches the spec.
fn verify_existence_proof(
    existence_proof: &ExistenceProof,
    spec: &ProofSpec,
    root: &[u8],
    key: &[u8],
    value: &[u8],
) -> Result<(), VerifyError> {
    existence_proof::check_against_spec(existence_proof, spec)
        .map_err(VerifyError::SpecMismatch)?;

    if key != existence_proof.key {
        return Err(VerifyError::KeyAndExistenceProofKeyMismatch {
            key: key.into(),
            existence_proof_key: existence_proof.key.clone(),
        });
    }

    if value != existence_proof.value {
        return Err(VerifyError::ValueAndExistenceProofValueMismatch {
            value: value.into(),
            existence_proof_value: existence_proof.value.clone(),
        });
    }

    let calc = existence_proof::calculate(existence_proof, Some(spec))
        .map_err(VerifyError::RootCalculation)?;

    if root != calc {
        return Err(VerifyError::CalculatedAndGivenRootMismatch {
            calculated_root: calc,
            given_root: root.into(),
        });
    }

    Ok(())
}
