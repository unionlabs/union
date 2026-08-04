use std::borrow::{Borrow, Cow};

use unionlabs::{
    cosmos::ics23::{
        existence_proof::ExistenceProof,
        hash_op::HashOp,
        inner_op::InnerOp,
        inner_spec::{InnerSpec, PositiveI32AsUsize},
        non_existence_proof::NonExistenceProof,
        proof_spec::ProofSpec,
    },
    primitives::Bytes,
};

use crate::{
    existence_proof::{self, CalculateRootError, SpecMismatchError},
    ops::hash_op::{HashError, do_hash},
};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VerifyError {
    #[error("spec mismatch ({0})")]
    SpecMismatch(SpecMismatchError),
    #[error(
        "key and existence proof key mismatch (key: {key}, existence_proof_key: {existence_proof_key})"
    )]
    KeyAndExistenceProofKeyMismatch {
        key: Bytes,
        existence_proof_key: Bytes,
    },
    #[error(
        "value and existence proof value mismatch (value: {value}, existence_proof_value: {existence_proof_value})"
    )]
    ValueAndExistenceProofValueMismatch {
        value: Bytes,
        existence_proof_value: Bytes,
    },
    #[error("root calculation ({0})")]
    RootCalculation(CalculateRootError),
    #[error(
        "calculated and given root doesn't match (calculated_root: {calculated_root}, given_root: {given_root})"
    )]
    CalculatedAndGivenRootMismatch {
        calculated_root: Bytes,
        given_root: Bytes,
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
    #[error("neighbor search failure ({0})")]
    NeighborSearch(NeighborSearchError),
    #[error(transparent)]
    Hash(#[from] HashError),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VerifyMembershipError {
    #[error("existence proof verification failed ({0})")]
    ExistenceProofVerify(VerifyError),
    #[error("proof does not exist")]
    ProofDoesNotExist,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum NeighborSearchError {
    #[error("invalid branch {branch} (order length: {order_len})")]
    InvalidBranch { branch: usize, order_len: usize },
    #[error("branch ({branch}) not found in ({order:?})")]
    BranchNotFoundInOrder {
        branch: usize,
        order: Vec<PositiveI32AsUsize>,
    },
    #[error("cannot find any valid spacing for this node")]
    CannotFindValidSpacing,
    #[error("invalid path provided for proof")]
    InvalidPath,
}

/// Implements ICS-23 verifyNonMembership: verifies a proof that a path has not been set to any value in a commitment.
pub fn verify_non_membership(
    spec: &ProofSpec,
    root: &[u8],
    proof: &NonExistenceProof,
    key: &[u8],
) -> Result<(), VerifyMembershipError> {
    verify_non_existence(proof, spec, root, key)
        .map_err(VerifyMembershipError::ExistenceProofVerify)
}

/// Implements ICS-23 verifyMembership: verifies a proof that a path has been set to a particular value in a commitment.
pub fn verify_membership(
    spec: &ProofSpec,
    root: &[u8],
    proof: &ExistenceProof,
    key: &[u8],
    value: &[u8],
) -> Result<(), VerifyMembershipError> {
    verify_existence_proof(proof, spec, root, key, value)
        .map_err(VerifyMembershipError::ExistenceProofVerify)
}

fn verify_non_existence(
    non_existence_proof: &NonExistenceProof,
    spec: &ProofSpec,
    root: &[u8],
    key: &[u8],
) -> Result<(), VerifyError> {
    let left_key = if let Some(left) = &non_existence_proof.left {
        verify_existence_proof(left, spec, root, &left.key, &left.value)?;
        left.key.clone()
    } else {
        Default::default()
    };

    let right_key = if let Some(right) = &non_existence_proof.right {
        verify_existence_proof(right, spec, root, &right.key, &right.value)?;
        right.key.clone()
    } else {
        Default::default()
    };

    if left_key.is_empty() && right_key.is_empty() {
        return Err(VerifyError::BothProofsMissing);
    }

    if !right_key.is_empty()
        && key_for_comparison(spec, key)? >= key_for_comparison(spec, &right_key)?
    {
        return Err(VerifyError::KeyIsNotLeftOfRightProof);
    }

    if !left_key.is_empty()
        && key_for_comparison(spec, key)? <= key_for_comparison(spec, &left_key)?
    {
        return Err(VerifyError::KeyIsNotRightOfLeftProof);
    }

    match (&non_existence_proof.left, &non_existence_proof.right) {
        (Some(left), None) => {
            if !is_right_most(&spec.inner_spec, &left.path).map_err(VerifyError::NeighborSearch)? {
                return Err(VerifyError::RightProofMissing);
            }
        }
        (None, Some(right)) => {
            if !is_left_most(&spec.inner_spec, &right.path).map_err(VerifyError::NeighborSearch)? {
                return Err(VerifyError::LeftProofMissing);
            }
        }
        (Some(left), Some(right)) => {
            if !is_left_neighbor(&spec.inner_spec, &left.path, &right.path)
                .map_err(VerifyError::NeighborSearch)?
            {
                return Err(VerifyError::RightProofMissing);
            }
        }
        (None, None) => return Err(VerifyError::BothProofsMissing),
    }

    Ok(())
}

fn key_for_comparison<'a>(spec: &ProofSpec, key: &'a [u8]) -> Result<Cow<'a, [u8]>, HashError> {
    if !spec.prehash_key_before_comparison {
        return Ok(Cow::Borrowed(key));
    }
    if spec.leaf_spec.prehash_key == HashOp::NoHash {
        Ok(Cow::Borrowed(key))
    } else {
        Ok(Cow::Owned(do_hash(spec.leaf_spec.prehash_key, key)?))
    }
}

/// returns true if `right` is the next possible path right of `left`
///
/// Find the common suffix from the Left.Path and Right.Path and remove it. We have LPath and RPath now, which must be neighbors.
/// Validate that LPath[len-1] is the left neighbor of RPath[len-1]
/// For step in LPath[0..len-1], validate step is right-most node
/// For step in RPath[0..len-1], validate step is left-most node
fn is_left_neighbor(
    spec: &InnerSpec,
    left: &[InnerOp],
    right: &[InnerOp],
) -> Result<bool, NeighborSearchError> {
    let (mut top_left, mut left) = left.split_last().ok_or(NeighborSearchError::InvalidPath)?;
    let (mut top_right, mut right) = right.split_last().ok_or(NeighborSearchError::InvalidPath)?;

    while top_left.prefix == top_right.prefix && top_left.suffix == top_right.suffix {
        (top_left, left) = left.split_last().ok_or(NeighborSearchError::InvalidPath)?;
        (top_right, right) = right.split_last().ok_or(NeighborSearchError::InvalidPath)?;
    }

    if !is_left_step(spec, top_left, top_right)?
        || !is_right_most(spec, left)?
        || !is_left_most(spec, right)?
    {
        return Ok(false);
    }

    Ok(true)
}

/// assumes left and right have common parents
/// checks if left is exactly one slot to the left of right
fn is_left_step(
    spec: &InnerSpec,
    left: &InnerOp,
    right: &InnerOp,
) -> Result<bool, NeighborSearchError> {
    let left_idx = order_from_padding(spec, left)?;

    let right_idx = order_from_padding(spec, right)?;

    Ok(right_idx == left_idx + 1)
}

/// returns true if this is the right-most path in the tree, excluding placeholder (empty child) nodes
fn is_right_most(spec: &InnerSpec, path: &[InnerOp]) -> Result<bool, NeighborSearchError> {
    let (min_prefix, max_prefix, suffix) = get_padding(spec, spec.child_order.len() - 1)?;

    for step in path {
        if !has_padding(step, min_prefix, max_prefix, suffix)
            && !right_branches_are_empty(spec, step)?
        {
            return Ok(false);
        }
    }

    Ok(true)
}

/// returns true if this is the left-most path in the tree, excluding placeholder (empty child) nodes
fn is_left_most(spec: &InnerSpec, path: &[InnerOp]) -> Result<bool, NeighborSearchError> {
    let (min_prefix, max_prefix, suffix) = get_padding(spec, 0)?;

    for step in path {
        if !has_padding(step, min_prefix, max_prefix, suffix)
            && !left_branches_are_empty(spec, step)?
        {
            return Ok(false);
        }
    }

    Ok(true)
}

/// returns true if the padding bytes correspond to all empty siblings
/// on the right side of a branch, ie. it's a valid placeholder on a rightmost path
pub fn right_branches_are_empty(
    spec: &InnerSpec,
    op: &InnerOp,
) -> Result<bool, NeighborSearchError> {
    let idx = order_from_padding(spec, op)?;

    let right_branches = spec.child_order.len() - 1 - idx;
    if right_branches == 0 {
        return Ok(false);
    }

    if op.suffix.len() != right_branches * spec.child_size.inner() {
        return Ok(false);
    }

    for i in 0..right_branches {
        let idx = get_position(&spec.child_order, i)?;
        let from = (idx * spec.child_size.inner()) as usize;

        let Some(suffix) = op.suffix.get(from..(from + spec.child_size.inner())) else {
            return Ok(false);
        };

        if spec.empty_child != suffix {
            return Ok(false);
        }
    }

    Ok(true)
}

/// returns true if the padding bytes correspond to all empty siblings
/// on the left side of a branch, ie. it's a valid placeholder on a leftmost path
pub fn left_branches_are_empty(
    spec: &InnerSpec,
    op: &InnerOp,
) -> Result<bool, NeighborSearchError> {
    let left_branches = order_from_padding(spec, op)?;

    if left_branches == 0 {
        return Ok(false);
    }

    // NOTE: Reference implementation checks `actual_prefix < 0` with signed integers
    let Some(actual_prefix) = op
        .prefix
        .len()
        .checked_sub(left_branches * spec.child_size.inner())
    else {
        return Ok(false);
    };

    for i in 0..left_branches {
        let idx = get_position(&spec.child_order, i)?;
        let from = actual_prefix + (idx * spec.child_size.inner());
        if Some(spec.empty_child.borrow().as_ref())
            != op.prefix.get(from..from + spec.child_size.inner())
        {
            return Ok(false);
        }
    }

    Ok(true)
}

/// will look at the proof and determine which order it is...
/// So we can see if it is branch 0, 1, 2 etc... to determine neighbors
fn order_from_padding(spec: &InnerSpec, inner: &InnerOp) -> Result<usize, NeighborSearchError> {
    for branch in 0..spec.child_order.len() {
        let (minp, maxp, suffix) = get_padding(spec, branch)?;
        if has_padding(inner, minp, maxp, suffix) {
            return Ok(branch);
        }
    }

    Err(NeighborSearchError::CannotFindValidSpacing)
}

/// checks if an op has the expected padding
fn has_padding(op: &InnerOp, min_prefix: usize, max_prefix: usize, suffix: usize) -> bool {
    if op.prefix.len() < min_prefix || op.prefix.len() > max_prefix {
        return false;
    }

    op.suffix.len() == suffix
}

/// determines prefix and suffix with the given spec and position in the tree
fn get_padding(
    spec: &InnerSpec,
    branch: usize,
) -> Result<(usize, usize, usize), NeighborSearchError> {
    let idx = get_position(&spec.child_order, branch)?;

    let prefix = idx * spec.child_size.inner();
    let min_prefix = prefix + spec.min_prefix_length.inner();
    let max_prefix = prefix + spec.max_prefix_length.inner();

    let suffix = (spec.child_order.len() - 1 - idx) * spec.child_size.inner();

    Ok((min_prefix, max_prefix, suffix))
}

/// checks where the branch is in the order and returns
/// the index of this branch
fn get_position(order: &[PositiveI32AsUsize], branch: usize) -> Result<usize, NeighborSearchError> {
    // NOTE: Reference implementation checks `branch < 0` as well as it uses signed integers
    if branch >= order.len() {
        return Err(NeighborSearchError::InvalidBranch {
            branch,
            order_len: order.len(),
        });
    }

    match order
        .iter()
        .enumerate()
        .find(|(_, elem)| elem.inner() == branch)
    {
        Some((i, _)) => Ok(i),
        None => Err(NeighborSearchError::BranchNotFoundInOrder {
            branch,
            order: order.to_vec(),
        }),
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

    if key != &existence_proof.key[..] {
        return Err(VerifyError::KeyAndExistenceProofKeyMismatch {
            key: key.into(),
            existence_proof_key: existence_proof.key.clone(),
        });
    }

    if value != &existence_proof.value[..] {
        return Err(VerifyError::ValueAndExistenceProofValueMismatch {
            value: value.into(),
            existence_proof_value: existence_proof.value.clone(),
        });
    }

    let calc = existence_proof::calculate(existence_proof, Some(spec))
        .map_err(VerifyError::RootCalculation)?;

    if root != calc {
        return Err(VerifyError::CalculatedAndGivenRootMismatch {
            calculated_root: calc.into(),
            given_root: root.into(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::{
        cosmos::ics23::commitment_proof::CommitmentProof,
        encoding::{Bincode, DecodeAs, EncodeAs as _, Proto},
        ibc::core::{
            channel::order::Order,
            commitment::{merkle_proof::MerkleProof, merkle_root::MerkleRoot},
            connection::{connection_end::ConnectionEnd, version::Version},
        },
        id::{ClientId, ConnectionId},
    };

    use super::*;
    use crate::{ibc_api::SDK_SPECS, proof_specs::TENDERMINT_PROOF_SPEC};

    fn ensure_existent(
        proof: &[u8],
        root: &[u8],
        key: &[u8],
        value: &[u8],
    ) -> Result<(), VerifyMembershipError> {
        let CommitmentProof::Exist(commitment_proof) =
            CommitmentProof::decode_as::<Proto>(proof).unwrap()
        else {
            panic!("unexpected proof type");
        };

        super::verify_membership(&TENDERMINT_PROOF_SPEC, root, &commitment_proof, key, value)
    }

    fn ensure_non_existent(
        proof: &[u8],
        root: &[u8],
        key: &[u8],
    ) -> Result<(), VerifyMembershipError> {
        let CommitmentProof::Nonexist(commitment_proof) =
            CommitmentProof::decode_as::<Proto>(proof).unwrap()
        else {
            panic!("unexpected proof type");
        };

        verify_non_membership(&TENDERMINT_PROOF_SPEC, root, &commitment_proof, key)
    }

    #[test]
    fn verify_membership_left() {
        let proof = hex!(
            "0adb030a14303142424373615a55715146735259436c6a5767121e76616c75655f666f725f303142424373615a55715146735259436c6a57671a090801180120012a0100222708011201011a20cb3131cd98b069efcc0e8c7e68da47370adbff32266d7fcd1b0580fdf3961266222708011201011a2021d1205c1f8537205e8fb4b176f960b459d9131669968d59c456442f7673b68b222708011201011a20b82a0e7f4434b3cedb87ea83eb5a70c7dc664c77b2fe21c6245f315e58fdf745222708011201011a20bf0657a0e6fbd8f2043eb2cf751561adcf50547d16201224133eeb8d38145229222708011201011a206d47c03df91a4a0252055d116439d34b5b73f3a24d5cb3cf0d4b08caa540cac4222708011201011a20d5d2926993fa15c7410ac4ee1f1d81afddfb0ab5f6f4706b05f407bc01638149222708011201011a20540719b26a7301ad012ac45ebe716679e5595e5570d78be9b6da8d8591afb374222708011201011a20fccaaa9950730e80b9ccf75ad2cfeab26ae750b8bd6ac1ff1c7a7502f3c64be2222708011201011a20ecb61a6d70accb79c2325fb0b51677ed1561c91af5e10578c8294002fbb3c21e222708011201011a201b3bc1bd8d08af9f6199de84e95d646570cbd9b306a632a5acf617cbd7d1ab0a"
        );
        let root = hex!("c569a38a5775bbda2051c34ae00894186f837c39d11dca55495b9aed14f17ddf");
        let key = hex!("303142424373615a55715146735259436c6a5767");
        let value = hex!("76616c75655f666f725f303142424373615a55715146735259436c6a5767");

        assert_eq!(ensure_existent(&proof, &root, &key, &value), Ok(()));
    }

    #[test]
    fn verify_membership_middle() {
        let proof = hex!(
            "0ad1030a14513334656d766f39447145585735325257523835121e76616c75655f666f725f513334656d766f394471455857353252575238351a090801180120012a010022250801122101e231d775380f2d663651e213cc726660e2ce0a2f2e9ee12cbb7df32294104a8c222708011201011a2014af194c63500236e52cc290ab24244fab39a520ece7e20fa93f4c9ff80c6626222508011221017966d2ead34418db2eaa04c0dffb9316805e8a0d421d1270c8954c35ee3221382225080112210172339e20a49bb16795a99bd905b47f99c45e5e5a9e6b7fb223dc8fe6751e1bda222708011201011a2053dd1ecc25ff906a0ef4db37ee068f3d8ad6d1d49913eefb847a675a681c5ffa222708011201011a20de90f9951a19497be7e389e02aa79e26faf77080e740e8743249a17a537f287d22250801122101ad4e53e981afc5a71e34ab0c4ffbccf1b468414d9d0939bd08edbd2461bc944a222708011201011a209b4cf89c3995b9dd66d58ab088846b2c6b59c52c6d10ec1d759ca9e9aa5eef5c222508011221013928a078bd66ab3949f5b1846b6d354dbdc1968a416607c7d91555ca26716667222708011201011a20d2d82cf8915b9ae6f92c7eae343e37d312ace05e654ce47acdf57d0a5490b873"
        );
        let root = hex!("494b16e3a64a85df143b2881bdd3ec94c3f8e18b343e8ff9c2d61afd05d040c8");
        let key = hex!("513334656d766f39447145585735325257523835");
        let value = hex!("76616c75655f666f725f513334656d766f39447145585735325257523835");

        assert_eq!(ensure_existent(&proof, &root, &key, &value), Ok(()));
    }

    #[test]
    fn verify_membership_right() {
        let proof = hex!(
            "0aab020a147a785a4e6b534c64634d655657526c7658456644121e76616c75655f666f725f7a785a4e6b534c64634d655657526c76584566441a090801180120012a0100222508011221012634b831468dbafb1fc61a979c348ff8462da9a7d550191a6afc916ade16cc9922250801122101ab814d419bfc94ee9920d0ce993ce5da011e43613daf4b6f302855760083d7dd222508011221015a1568c73eaeaba567a6b2b2944b0e9a0228c931884cb5942f58ed835b8a7ac522250801122101a171412db5ee84835ef247768914e835ff80b7711e4aa8060871c2667ec3ea2922250801122101f9c2491884de24fb61ba8f358a56b306a8989bd35f1f8a4c8dabce22f703cc14222508011221012f12a6aa6270eff8a1628052938ff5e36cfcc5bf2eaedc0941ee46398ebc7c38"
        );
        let root = hex!("f54227f1a7d90aa2bf7931066196fd3072b7fe6b1fbd49d1e26e85a90d9541bb");
        let key = hex!("7a785a4e6b534c64634d655657526c7658456644");
        let value = hex!("76616c75655f666f725f7a785a4e6b534c64634d655657526c7658456644");

        assert_eq!(ensure_existent(&proof, &root, &key, &value), Ok(()));
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_left.json
    #[test]
    fn verify_non_membership_left() {
        let proof = hex!(
            "12e4030a04010101011adb030a143032615465366472483456706f4f583245507137121e76616c75655f666f725f3032615465366472483456706f4f5832455071371a090801180120012a0100222708011201011a20b843481496dc10561056b63ec8f726f3357395b610355b25082f5768b2073e91222708011201011a20d5281fdd872060e89173d4de1100fa6c96f778467df66abb10cf3b1f5821f182222708011201011a20eb981020433d929c6275ad772accf2e6aa916db97e31d2f26d0b6b07b444bbef222708011201011a204a40e813132aff60b64ba9d109548ab39459ad48a203ab8d3455dd842a7ab1da222708011201011a208f354a84ce1476e0b9cca92e65301a6435b1f242c2f53f943b764a4f326a71c7222708011201011a20ac6451617a6406005035dddad36657fde5312cc4d67d69ca1464611847c10cfb222708011201011a2023c1d1dd62002a0e2efcc679196589a4337234dcd209cb449cc3ac10773b60e0222708011201011a203b11c267328ba761ddc630dd5ef7642aeda05f180539fe93c0ca57729705bc46222708011201011a205ff2e1933be704539463c264b157ff2b8d9960813bd36c69c5208d57e3b1e07e222708011201011a20c4a79e6c0cbf60fb8e5bf940db4c444b7e442951b69c840db38cf28c8aa008be"
        );
        let root = hex!("4e2e78d2da505b7d0b00fda55a4b048eed9a23a7f7fc3d801f20ce4851b442aa");
        let key = hex!("01010101");

        assert_eq!(ensure_non_existent(&proof, &root, &key), Ok(()));
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_middle.json
    #[test]
    fn verify_non_membership_middle() {
        let proof = hex!(
            "12c0070a14544f31483668784a4b667136547a56767649ffff12cf030a14544f31483668784a4b667136547a567676497747121e76616c75655f666f725f544f31483668784a4b667136547a5676764977471a090801180120012a01002225080112210143e19cb5e5dab017734caa78a2e2bccbb4797b7dc5a91abeab630c66fa6b162522250801122101b575404a1bb42b0fef8ae7f217af88aec769f7d66b5bc4b2913e74d651365473222508011221017c22dc50e866f9a1dce517ea01621161cecd70f4bdcd024b5a392746a1c8dc2622250801122101578105344f2c98c323ba0b8ca31e75aaa2b865cc389681e300b14d1c20713796222708011201011a20895c070c14546ecef7f5cb3a4bda1fd436a0ff99190f90bd037cbeaf52b2ffc1222708011201011a20f7571fca06ac4387c3eae5469c152427b797abb55fa98727eacbd5c1c91b5fb4222508011221015056e6472f8e5c5c9b8881c5f0e49601e9eca31f3e1766aa69c2dc9c6d9112be222708011201011a206c74439556c5edb5aa693af410d3718dbb613d37799f2f4e8ff304a8bfe3351b22250801122101253014334c7b8cd78436979554f7890f3dc1c971925ea31b48fc729cd179c701222708011201011a20b81c19ad4b5d8d15f716b91519bf7ad3d6e2289f9061fd2592a8431ea97806fe1ad5030a14544f433344683150664f76657538585166635778121e76616c75655f666f725f544f433344683150664f766575385851666357781a090801180120012a0100222708011201011a20415d4cfaed0bfc98ac32acc219a8517bfa1983a15cc742e8b2f860167484bd46222708011201011a2098d853d9cc0ee1d2162527f660f2b90ab55b13e5534f1b7753ec481d7901d3ec222708011201011a20b5113e6000c5411b7cfa6fd09b6752a43de0fcd3951ed3b154d162deb53224a2222708011201011a208ce18cd72cc83511cb8ff706433f2fa4208c85b9f4c8d0ed71a614f24b89ae6c22250801122101c611244fe6b5fda4257615902eb24c14efcd9708c7c875d1ac5e867767aa1eab222708011201011a20f7571fca06ac4387c3eae5469c152427b797abb55fa98727eacbd5c1c91b5fb4222508011221015056e6472f8e5c5c9b8881c5f0e49601e9eca31f3e1766aa69c2dc9c6d9112be222708011201011a206c74439556c5edb5aa693af410d3718dbb613d37799f2f4e8ff304a8bfe3351b22250801122101253014334c7b8cd78436979554f7890f3dc1c971925ea31b48fc729cd179c701222708011201011a20b81c19ad4b5d8d15f716b91519bf7ad3d6e2289f9061fd2592a8431ea97806fe"
        );
        let root = hex!("4bf28d948566078c5ebfa86db7471c1541eab834f539037075b9f9e3b1c72cfc");
        let key = hex!("544f31483668784a4b667136547a56767649ffff");

        assert_eq!(ensure_non_existent(&proof, &root, &key), Ok(()));
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_right.json
    #[test]
    fn verify_non_membership_right() {
        let proof = hex!(
            "12a9030a04ffffffff12a0030a147a774e4d4a456f7932674253586277666e63504a121e76616c75655f666f725f7a774e4d4a456f7932674253586277666e63504a1a090801180120012a01002225080112210178a215355c17371583418df95773476b347a853f6eae317677721e0c24e78ad2222508011221015e2cf893e7cd70251eb4debd855c8c9a92f6e0a1fd931cf41e0575846ab174e822250801122101414bae883f8133f0201a2791dafeaef3daa24a6631b3f9402de3a4dc658fd035222508011221012e2829beee266a814af4db08046f4575b011e5ec9d2d93c1510c3cc7d8219edc22250801122101f8286597078491ae0ef61264c218c6e167e4e03f1de47945d9ba75bb41deb81a22250801122101dea6a53098d11ce2138cbcae26b392959f05d7e1e24b9547584571012280f289222508011221010a8e535094d18b2120c38454b445d9accf3f1b255690e6f3d48164ae73b4c775222508011221012cbb518f52ec1f8e26dd36587f29a6890a11c0dd3f94e7a28546e695f296d3a722250801122101839d9ddd9dadf41c0ecfc3f7e20f57833b8fb5bcb703bef4f97910bbe5b579b9"
        );
        let root = hex!("83952b0b17e64c862628bcc1277e7f8847589af794ed5a855339281d395ec04f");
        let key = hex!("ffffffff");

        assert_eq!(ensure_non_existent(&proof, &root, &key), Ok(()));
    }

    #[test]
    fn verify_chained_test() {
        let proof = MerkleProof::decode_as::<Proto>(&hex!("0aa5020aa2020a18636f6e6e656374696f6e732f636f6e6e656374696f6e2d3012460a0930382d7761736d2d3012140a0131120f4f524445525f554e4f524445524544180222210a0a636f6d6574626c732d30120c636f6e6e656374696f6e2d301a050a036962631a0c0801180120012a040002ca01222a080112260204ca012067b76c7b82d60ebee7f41dd11a02534c1a16efa70c217310356230dfd5ad0c2020222a080112260406aa0220fe0560ee5685e1c214bcb958f761a467858478ed4a2ddcf77cc0f27258248f9c20222c08011205060eaa02201a2120140ee5ef0cddcc422e389954ff959f52c905a7211e62e3a14f67199ad81e0322222a08011226081aaa02203d62d598ecb60b8721fb2ace147909fb3c61c54dc7b54e04d028cc21e10d505a200afc010af9010a036962631220552a1b22544e343a046985a0ae8cc625adc18a18b7669a64ae9e4c9ba6754f461a090801180120012a0100222708011201011a202cd8b50700950546180ad979135a8708c2ea2098fff6ade31b7e40eb5dcf7c05222508011221012cf3feea58fcdb48b73c2cdd1b018c90c4078f924385675a0e9457168cd47ff1222508011221016bd19d4e1e3d1d96827c449152c4bedc0d5d306e9696d3ca78983d6866891f3122250801122101a9788106a88704540fe0ead349d99096acaae60826863dd426a530b82570b757222708011201011a20a2fac4bcd28e2655f7985c9aad923140076c1764bd862ebfa999f8ed2bacfbf7")).unwrap();
        let root = hex!("88be092a61a8033111d4625bdbdc48c814b7258a2ec560e731b9fd17780e45ed");
        let key = b"connections/connection-0";

        crate::ibc_api::verify_membership(
            &proof,
            &SDK_SPECS,
            &MerkleRoot {
                hash: unionlabs::primitives::H256::new(root),
            },
            &[b"ibc".to_vec(), key.to_vec()],
            ConnectionEnd {
                client_id: ClientId::new("08-wasm", 0),
                versions: vec![Version {
                    identifier: "1".to_string(),
                    features: vec![Order::Unordered],
                }],
                state: unionlabs::ibc::core::connection::state::State::Tryopen,
                counterparty: unionlabs::ibc::core::connection::counterparty::Counterparty {
                    client_id: ClientId::new("cometbls", 0),
                    connection_id: Some(ConnectionId::new(0)),
                    prefix: unionlabs::ibc::core::commitment::merkle_prefix::MerklePrefix {
                        key_prefix: b"ibc".into(),
                    },
                },
                delay_period: 0,
            }
            .encode_as::<Proto>(),
        )
        .unwrap();
    }

    #[test]
    fn gno_fail() {
        let proof = MerkleProof::decode_as::<Bincode>(&hex!("02000000000000000000000063000000000000002f70762f766d3a676e6f2e6c616e642f722f636f72652f6962632f76312f636f72653a376466653735376563643635636264373932326139633031363165393335646437666462636330653939393638396337643331363333383936623166633630622000000000000000e2b195a591ad754086ca7eadcfe988ad192ce5f5cabec8bd7b780bd2649bbb3a0100000000000000010000000100000003000000000000000002360a00000000000000010000002500000000000000020436200f58f33dc5e4d0799b73d7ef734783d13f651906c81854ac8cd30885d3cc66bf20000000000000000001000000250000000000000004063620aa3ca7fcbf169ff01eadd4de4c08c9a98661427392db74e38b85932acc99d5b0200000000000000000010000000400000000000000060c362021000000000000002014568e372a3853f7f49eb200a26dd5e294ae3e596ecd373110df7a10c2f65e1c01000000250000000000000008123620d0acd3fc160691af97bd1c446ce3000cbca86f92ac27f5ef1e6d543f9157db5c2000000000000000000100000004000000000000000a2836202100000000000000209d11e960c794f090ddf72ad19559ec9b49dec5c06787f67342f507e8f6ad42960100000025000000000000000c5a3620510ff1a0ac96a57896c19ac3c4a8f0fc8d1a77b895d0a80e5caba670010adf922000000000000000000100000026000000000000000e9a013620c78d381f53be2c40962ed48f41efa2c9abe5027220b8acd60b2cffbfa4ade155200000000000000000010000000500000000000000109a0236202100000000000000201713c3e4543cc43a9468e6819fdffa6568c0525c5cb4591cd7aa1fe5d390455a010000000500000000000000129a0436202100000000000000200f08807ee42a537f73121fc34e4b69d969f47d947c8f1b8dc8f75f0d9308858001000000050000000000000014f20b362021000000000000002068a7435db84845a7894baf2ab8205ba81c9b4c9240c3e79106136bbdb04ef7910000000004000000000000006d61696e200000000000000006a283452e4d4b9c804299183910f190143886e1ab241bd50260dc32b71b146601000000000000000100000001000000010000000000000000010000000000000001000000210000000000000001ccb581a002a493db462c72bc97aac085192a8ffb6a45fa5ee3cf22ee89eb15740000000000000000")).unwrap();

        let root = hex!("812a65c1513f837fb0ae43d51219941484a852f69a3cfaa9930f6e3968b684fe");
        let key = b"/pv/vm:gno.land/r/core/ibc/v1/core:7dfe757ecd65cbd7922a9c0161e935dd7fdbcc0e999689c7d31633896b1fc60b";

        crate::ibc_api::verify_membership(
            &proof,
            &SDK_SPECS,
            &MerkleRoot {
                hash: unionlabs::primitives::H256::new(root),
            },
            &[b"main".to_vec(), key.to_vec()],
            hex!("e2b195a591ad754086ca7eadcfe988ad192ce5f5cabec8bd7b780bd2649bbb3a").into(),
        )
        .unwrap();
    }

    #[test]
    fn verify_non_membership_bptree_topaz() {
        use crate::ibc_api::GNO_SPECS;

        let CommitmentProof::Nonexist(nonexist) = CommitmentProof::decode_as::<Proto>(&hex!(
            "12c286010a27706b673a676e6f2e6c616e642f722f6e6f6e6578697374656e745f7265616c6d5f78797a31323312b10c0a21706b673a676e6f2e6c616e642f722f6d6f756c2f64656d6f2f68656c6c6f2f763212a7070a0568656c6c6f121d676e6f2e6c616e642f722f6d6f756c2f64656d6f2f68656c6c6f2f76321ab6040a09524541444d452e6d6412a804232060676e6f2e6c616e642f722f6d6f756c2f64656d6f2f68656c6c6f2f7631600a0a5f544f444f3a2064657363726962652074686973207061636b6167652e5f0a0a3c212d2d20424547494e20474e4f434f4e54524143545320464f4f544552202867656e65726174656420627920606d616b6520726561646d6573603b20646f206e6f7420656469742062656c6f7729202d2d3e0a0a2d2d2d0a0a50617274206f66202a2a5b6d6f756c2f676e6f2d636f6e7472616374735d2868747470733a2f2f6769746875622e636f6d2f6d6f756c2f676e6f2d636f6e747261637473292a2a20e28094206d6f756c27732076657273696f6e656420676e6f2e6c616e6420636f6e7472616374732e2053656520746865207265706f7369746f727920666f72207468652066756c6c20636174616c6f672c206275696c642f7465737420746f6f6c696e672c20616e642075736167652e0a0a3e20e29aa0efb88f202a2a446973636c61696d65723a2a2a2070726f76696465642061732d69732c20776974686f75742077617272616e74793b206e6f742073656375726974792d617564697465642e2046756c6c20646973636c61696d65723a205b444953434c41494d45525d2868747470733a2f2f6769746875622e636f6d2f6d6f756c2f676e6f2d636f6e7472616374732f626c6f622f6d61696e2f444953434c41494d45522e6d64292e0a0a3c212d2d20454e4420474e4f434f4e54524143545320464f4f544552202d2d3e0a1a85010a0c636f6e74726163742e676e6f12757061636b6167652068656c6c6f0a0a66756e632048656c6c6f282920737472696e67207b0a0972657475726e202248656c6c6f20576f726c6421220a7d0a0a66756e6320707269766174654d6574686f64282920737472696e67207b0a0972657475726e202249276d2070726976617465220a7d0a1a98010a0b676e6f6d6f642e746f6d6c1288016d6f64756c65203d2022676e6f2e6c616e642f722f6d6f756c2f64656d6f2f68656c6c6f2f7632220a676e6f203d2022302e39220a0a5b616464706b675d0a202063726561746f72203d202267316d616e6672656434376b7a647565633932307a38387766723634796c6b736d646365646c6635220a2020686569676874203d203233373030350a22230a132f676e6f2e4d656d5061636b61676554797065120c0a0a4d505573657250726f641a090801180120012a0100222708011201011a2077a24253e18ff379e0aa275b16cbe18aaba2deb9c8398d46aa52089b3786753d222708011201011a20826138bb91dc3ccad0c0140b75509f20b7d2cc4a39e7a1cb9bb59e1fe7d7d3fa22250801122101e8189f97ee74a4f03571949040d7cbae23d2bf3bd1988a59828a1a0e85551e5d222508011221018597810966c1ccb68dc7d389a5e41717b7cf0d2c1d419dcf693676fb7794b157222708011201011a2027a3d2373fd45bae98336b1841ef9d73d969c7f55e00eed79cb57a3f52f4ff20222508011221019bffc19b7904631aec365415461d3d9ca2a2c3456acd18c7c2e58989c3d33cf0222708011201011a200784679145f06662eda7160ad3ecd389672690afdee97e41386665610fecf43022250801122101863554fbad7fa41d7e300607c486345df74a9b7f34dbdcf1ba29d44b33d9208a222708011201011a20dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d98622250801122101d9cc44531049ff1e7a878857f7f44845e3008e78f69df2cb3412f57b08813a4422250801122101ba0aa49bb798366b36e79042adf626d7ccb65d3d2d0d50b089063d034bfc0d77222708011201011a20dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d9862225080112210156f4b8dd34f1c04b78192c86c910a120c1022ac255d090e8486814255da7e4a0222708011201011a20dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986222708011201011a20dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d9861ae2790a26706b673a676e6f2e6c616e642f722f6f6e626c6f632f6962632f756e696f6e2f61636365737312d5740a066163636573731222676e6f2e6c616e642f722f6f6e626c6f632f6962632f756e696f6e2f6163636573731adf230a0a6163636573732e676e6f12d0237061636b616765206163636573730a0a696d706f72742022676e6f2e6c616e642f702f6f6e626c6f632f6163636573732f6d616e61676572220a0a766172206163636573735374617465206d616e616765722e53746174650a0a2f2f2052656c61796572526f6c65206d6972726f727320556e696f6e206465706c6f79657227732052454c4159455220726f6c652069642e0a2f2f205265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6465706c6f7965722f7372632f6d61696e2e7273234c36350a636f6e73742052656c61796572526f6c65206d616e616765722e526f6c654964203d20310a0a2f2f204772616e74526f6c65206772616e747320726f6c65206d656d6265727368697020746f206163636f756e74207468726f756768207468652073686172656420616363657373207265616c6d2e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c36392d4c38370a66756e63204772616e74526f6c6528637572207265616c6d2c20726f6c65206d616e616765722e526f6c6549642c206163636f756e7420616464726573732920626f6f6c207b0a0961737365727443616e41646d696e526f6c6528302c206375722c20726f6c65290a0a096e65774d656d626572203a3d2061636365737353746174652e4772616e74526f6c6528726f6c652c206163636f756e74290a09616363657373203a3d2061636365737353746174652e526f6c65735b726f6c655d2e4d656d626572735b6163636f756e745d0a0a09656d6974526f6c654772616e7465644576656e7428726f6c652c206163636f756e742c206163636573732e53696e63652c206e65774d656d626572290a0a0972657475726e206e65774d656d6265720a7d0a0a2f2f205265766f6b65526f6c65207265766f6b657320726f6c65206d656d626572736869702066726f6d206163636f756e74207468726f756768207468652073686172656420616363657373207265616c6d2e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c38392d4c3130300a66756e63205265766f6b65526f6c6528637572207265616c6d2c20726f6c65206d616e616765722e526f6c6549642c206163636f756e7420616464726573732920626f6f6c207b0a0961737365727443616e41646d696e526f6c6528302c206375722c20726f6c65290a0a097265766f6b6564203a3d2061636365737353746174652e5265766f6b65526f6c6528726f6c652c206163636f756e74290a096966207265766f6b6564207b0a0909656d6974526f6c655265766f6b65644576656e7428726f6c652c206163636f756e74290a097d0a0a0972657475726e207265766f6b65640a7d0a0a2f2f2052656e6f756e6365526f6c65207265766f6b6573207468652063616c6c65722773206f776e20726f6c65206d656d6265727368697020616674657220636f6e6669726d6174696f6e2e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c3130322d4c3131350a66756e632052656e6f756e6365526f6c6528637572207265616c6d2c20726f6c65206d616e616765722e526f6c6549642c2063616c6c6572436f6e6669726d6174696f6e20616464726573732920626f6f6c207b0a096163636f756e74203a3d206375722e50726576696f757328292e4164647265737328290a0a097265766f6b6564203a3d2061636365737353746174652e52656e6f756e6365526f6c6528726f6c652c206163636f756e742c2063616c6c6572436f6e6669726d6174696f6e290a096966207265766f6b6564207b0a0909656d6974526f6c655265766f6b65644576656e7428726f6c652c206163636f756e74290a097d0a0a0972657475726e207265766f6b65640a7d0a0a2f2f204c6162656c526f6c6520656d69747320726f6c65206c6162656c206d6574616461746120666f7220616e20756e6c6f636b656420726f6c652e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c34382d4c36370a66756e63204c6162656c526f6c6528637572207265616c6d2c20726f6c65206d616e616765722e526f6c6549642c206c6162656c20737472696e6729207b0a0961737365727443616e4d616e61676554617267657428302c20637572290a0a096d616e616765722e52657175697265556e6c6f636b6564436f6e666967526f6c6528726f6c65290a0a09656d6974526f6c654c6162656c4576656e7428726f6c652c206c6162656c290a7d0a0a2f2f20536574526f6c6541646d696e2073657473207468652061646d696e20726f6c6520746861742063616e206772616e74206f72207265766f6b6520726f6c652e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c3131372d4c3132360a66756e6320536574526f6c6541646d696e28637572207265616c6d2c20726f6c65206d616e616765722e526f6c6549642c2061646d696e206d616e616765722e526f6c65496429207b0a0961737365727443616e4d616e61676554617267657428302c20637572290a0a0961636365737353746174652e536574526f6c6541646d696e28726f6c652c2061646d696e290a0a09656d6974526f6c6541646d696e4368616e6765644576656e7428726f6c652c2061646d696e290a7d0a0a2f2f205365744772616e7444656c61792073657473207468652064656c6179206265666f726520667574757265206772616e7473206f6620726f6c65206265636f6d65206163746976652e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c3133392d4c3134380a66756e63205365744772616e7444656c617928637572207265616c6d2c20726f6c65206d616e616765722e526f6c6549642c2064656c6179206d616e616765722e44656c617929207b0a0961737365727443616e4d616e61676554617267657428302c20637572290a0a0961636365737353746174652e5365744772616e7444656c617928726f6c652c2064656c6179290a0a09656d6974526f6c654772616e7444656c61794368616e6765644576656e7428726f6c652c2064656c61792c206d616e616765722e43757272656e7454696d65506f696e742829290a7d0a0a2f2f2053657446756e6374696f6e526f6c6520736574732074686520726f6c6520666f722073656c6563746f72206f6e207468652063616c6c696e67207265616c6d27732074617267657420706174682e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c3335332d4c3336370a66756e632053657446756e6374696f6e526f6c6528637572207265616c6d2c2073656c6563746f72206d616e616765722e53656c6563746f722c20726f6c65206d616e616765722e526f6c65496429207b0a0961737365727443616e4d616e61676554617267657428302c20637572290a0a09746172676574203a3d206375722e50726576696f757328292e506b675061746828290a0961636365737353746174652e53657454617267657446756e6374696f6e526f6c65287461726765742c2073656c6563746f722c20726f6c65290a0a09656d697454617267657446756e6374696f6e526f6c65557064617465644576656e74287461726765742c2073656c6563746f722c20726f6c65290a7d0a0a2f2f2053657446756e6374696f6e526f6c65732073657473207468652073616d6520726f6c6520666f722073656c6563746f7273206f6e207468652063616c6c696e67207265616c6d27732074617267657420706174682e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c3335332d4c3336370a66756e632053657446756e6374696f6e526f6c657328637572207265616c6d2c2073656c6563746f7273205b5d6d616e616765722e53656c6563746f722c20726f6c65206d616e616765722e526f6c65496429207b0a0961737365727443616e4d616e61676554617267657428302c20637572290a0a09746172676574203a3d206375722e50726576696f757328292e506b675061746828290a0961636365737353746174652e53657454617267657446756e6374696f6e526f6c6573287461726765742c2073656c6563746f72732c20726f6c65290a0a09666f722069203a3d2072616e67652073656c6563746f7273207b0a0909656d697454617267657446756e6374696f6e526f6c65557064617465644576656e74287461726765742c2073656c6563746f72735b695d2c20726f6c65290a097d0a7d0a0a2f2f20536574546172676574436c6f73656420736574732077686574686572207468652063616c6c696e67207265616c6d277320746172676574207061746820697320636c6f7365642e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c3435302d4c3438330a66756e6320536574546172676574436c6f73656428637572207265616c6d2c20636c6f73656420626f6f6c29207b0a0961737365727443616e4d616e61676554617267657428302c20637572290a0a09746172676574203a3d206375722e50726576696f757328292e506b675061746828290a0961636365737353746174652e536574546172676574436c6f736564287461726765742c20636c6f736564290a0a09656d6974546172676574436c6f7365644576656e74287461726765742c20636c6f736564290a7d0a1a95090a0a6173736572742e676e6f1286097061636b616765206163636573730a0a696d706f72742022676e6f2e6c616e642f702f6f6e626c6f632f6163636573732f6d616e61676572220a0a66756e63206173736572744973526c6d43757272656e74285f20696e742c20726c6d207265616c6d29207b0a0969662021726c6d2e497343757272656e742829207b0a090970616e69632845727253706f6f6665645265616c6d290a097d0a7d0a0a2f2f2041737365727443616e43616c6c2070616e69637320756e6c6573732063616c6c65722063616e2063616c6c2073656c6563746f72206f6e207468652063616c6c696e67207265616c6d27732074617267657420706174682e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765642f7372632f726573747269637465642e7273234c33382d4c36360a66756e632041737365727443616e43616c6c285f20696e742c20726c6d207265616c6d2c2073656c6563746f72206d616e616765722e53656c6563746f7229207b0a096173736572744973526c6d43757272656e7428302c20726c6d290a0a0963616c6c6572203a3d20726c6d2e50726576696f757328292e4164647265737328290a09746172676574203a3d20726c6d2e506b675061746828290a096966202161636365737353746174652e4973417574686f72697a65642863616c6c65722c207461726765742c2073656c6563746f7229207b0a090970616e6963284163636573734d616e61676572556e617574686f72697a656443616c6c4572726f722863616c6c65722c207461726765742c2073656c6563746f7229290a097d0a7d0a0a66756e632061737365727443616e4d616e616765546172676574285f20696e742c20726c6d207265616c6d29207b0a0963616c6c6572203a3d20726c6d2e50726576696f757328292e4164647265737328290a096966202161636365737353746174652e43616e4d616e6167655461726765742863616c6c6572292e496d6d656469617465207b0a090970616e6963284163636573734d616e61676572556e617574686f72697a65644163636f756e744572726f722863616c6c65722c206d616e616765722e41646d696e526f6c6529290a097d0a7d0a0a66756e632061737365727443616e41646d696e526f6c65285f20696e742c20726c6d207265616c6d2c20726f6c65206d616e616765722e526f6c65496429207b0a0963616c6c6572203a3d20726c6d2e50726576696f757328292e4164647265737328290a096966202161636365737353746174652e43616e41646d696e526f6c6528726f6c652c2063616c6c6572292e496d6d656469617465207b0a090970616e6963284163636573734d616e61676572556e617574686f72697a65644163636f756e744572726f722863616c6c65722c2061636365737353746174652e476574526f6c6541646d696e28726f6c652929290a097d0a7d0a1aab110a0c6465706c6f7965722e676e6f129a117061636b616765206163636573730a0a696d706f727420280a0922676e6f2e6c616e642f702f6f6e626c6f632f6163636573732f6d616e61676572220a0922676e6f2e6c616e642f702f6f6e626c6f632f6962632f756e696f6e2f7479706573220a290a0a636f6e737420280a09436f72655265616c6d50617468202020202020203d2022676e6f2e6c616e642f722f6f6e626c6f632f6962632f756e696f6e2f636f7265220a0944656661756c7441646d696e41646472657373203d206164647265737328226731716772373938307279376636727172396373366434357930396b357536647a686a6b736b736e22290a290a0a66756e6320696e69742829207b0a0972657365744163636573732844656661756c7441646d696e41646472657373290a7d0a0a2f2f207265736574416363657373206d6972726f727320746865206465706c6f792d74696d65207365747570207468617420556e696f6e20706572666f726d73207468726f756768206974730a2f2f206465706c6f79657220616674657220696e7374616e74696174696e67204163636573734d616e616765722e20476e6f20686173206e6f20696e7374616e7469617465206d6573736167652c20736f0a2f2f2074686973207265616c6d20757365732044656661756c7441646d696e416464726573732061732069747320696e697469616c2061646d696e20647572696e6720696e69742e0a66756e6320726573657441636365737328696e697469616c41646d696e206164647265737329207b0a096163636573735374617465203d206d616e616765722e4e6577537461746528290a0a092f2f204d6972726f727320696e697469616c2061646d696e20626f6f747374726170206f6e6c793a0a092f2f204f5a204163636573734d616e6167657220636f6e7374727563746f723a0a092f2f2068747470733a2f2f6769746875622e636f6d2f4f70656e5a657070656c696e2f6f70656e7a657070656c696e2d636f6e7472616374732f626c6f622f76352e362e312f636f6e7472616374732f6163636573732f6d616e616765722f4163636573734d616e616765722e736f6c234c3133332d4c3133360a092f2f20556e696f6e206163636573732d6d616e6167657220696e69743a0a092f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f6c69622e7273234c3134382d4c3135340a092f2f0a092f2f2052454c41594552206d656d62657273686970206973207365706172617465206465706c6f79657220706f6c69637920696e20556e696f6e20616e64206973206772616e7465642062790a092f2f204772616e74526f6c652c206e6f74206279204163636573734d616e6167657220696e697469616c697a6174696f6e3a0a092f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6465706c6f7965722f7372632f6d61696e2e7273234c313134332d4c313135320a0961636365737353746174652e4772616e74526f6c65286d616e616765722e41646d696e526f6c652c20696e697469616c41646d696e290a09616363657373203a3d2061636365737353746174652e526f6c65735b6d616e616765722e41646d696e526f6c655d2e4d656d626572735b696e697469616c41646d696e5d0a09656d6974526f6c654772616e7465644576656e74286d616e616765722e41646d696e526f6c652c20696e697469616c41646d696e2c206163636573732e53696e63652c2074727565290a097365747570526f6c657328290a7d0a0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6465706c6f7965722f7372632f6d61696e2e7273234c313737390a66756e63207365747570526f6c65732829207b0a0961636365737353746174652e53657454617267657446756e6374696f6e526f6c657328436f72655265616c6d506174682c2074797065732e52656c6179657253656c6563746f727328292c2052656c61796572526f6c65290a09666f72205f2c2073656c6563746f72203a3d2072616e67652074797065732e52656c6179657253656c6563746f72732829207b0a0909656d697454617267657446756e6374696f6e526f6c65557064617465644576656e7428436f72655265616c6d506174682c2073656c6563746f722c2052656c61796572526f6c65290a097d0a092f2f20556e696f6e206465706c6f796572206c6162656c7320616c6c2070726f64756374696f6e20726f6c65732e205765206f6e6c7920626f6f7473747261702052454c4159455220666f720a092f2f206e6f773b205041555345522f554e5041555345522f524154455f4c494d4954455220617265206164646564207768656e20746865697220726f6c6573206172652061646f707465642e0a092f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6465706c6f7965722f7372632f6d61696e2e7273234c313837312d4c313930380a096d616e616765722e52657175697265556e6c6f636b6564436f6e666967526f6c652852656c61796572526f6c65290a09656d6974526f6c654c6162656c4576656e742852656c61796572526f6c652c202252454c4159455222290a7d0a1ab40c0a0a6572726f72732e676e6f12a50c7061636b616765206163636573730a0a696d706f727420280a0922737472636f6e76220a0a0922676e6f2e6c616e642f702f6f6e626c6f632f6163636573732f6d616e61676572220a290a0a636f6e737420280a094572724163636573734d616e61676572556e617574686f72697a65644163636f756e74203d2022616363657373206d616e6167657220756e617574686f72697a6564206163636f756e74220a094572724163636573734d616e61676572556e617574686f72697a656443616c6c202020203d2022616363657373206d616e6167657220756e617574686f72697a65642063616c6c220a0945727253706f6f6665645265616c6d2020202020202020202020202020202020202020203d2022726c6d20646f6573206e6f74206d61746368207468652063757272656e742063726f7373696e67206672616d65220a290a0a2f2f204163636573734d616e61676572556e617574686f72697a65644163636f756e744572726f7220666f726d61747320616e206163636f756e742d726f6c6520617574686f72697a6174696f6e206572726f722e0a2f2f20556e696f6e207265666572656e6365733a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f6c69622f6163636573732d6d616e616765722d74797065732f7372632f6d616e616765722f6572726f722e7273234c32362d4c33300a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c3833322d4c3834300a66756e63204163636573734d616e61676572556e617574686f72697a65644163636f756e744572726f722863616c6c657220616464726573732c207265717569726564526f6c65206d616e616765722e526f6c6549642920737472696e67207b0a0972657475726e204572724163636573734d616e61676572556e617574686f72697a65644163636f756e74202b0a0909223a2022202b2063616c6c65722e537472696e672829202b0a090922206d757374206861766520726f6c652022202b20737472636f6e762e466f726d617455696e742875696e743634287265717569726564526f6c65292c203130290a7d0a0a2f2f204163636573734d616e61676572556e617574686f72697a656443616c6c4572726f7220666f726d6174732061207461726765742d73656c6563746f7220617574686f72697a6174696f6e206572726f722e0a2f2f20556e696f6e207265666572656e6365733a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f6c69622f6163636573732d6d616e616765722d74797065732f7372632f6d616e616765722f6572726f722e7273234c33322d4c33370a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c3532362d4c3533340a66756e63204163636573734d616e61676572556e617574686f72697a656443616c6c4572726f722863616c6c657220616464726573732c2074617267657420737472696e672c2073656c6563746f72206d616e616765722e53656c6563746f722920737472696e67207b0a0972657475726e204572724163636573734d616e61676572556e617574686f72697a656443616c6c202b0a0909223a2022202b2063616c6c65722e537472696e672829202b0a090922206973206e6f7420617574686f72697a656420746f2063616c6c2022202b20737472696e672873656c6563746f7229202b0a090922206f6e2022202b207461726765740a7d0a1ae50e0a0a6576656e74732e676e6f12d60e7061636b616765206163636573730a0a696d706f727420280a0922636861696e220a0922737472636f6e76220a0a0922676e6f2e6c616e642f702f6f6e626c6f632f6163636573732f6d616e61676572220a290a0a66756e6320656d6974526f6c654c6162656c4576656e7428726f6c65206d616e616765722e526f6c6549642c206c6162656c20737472696e6729207b0a09636861696e2e456d6974280a09096d616e616765722e4576656e7454797065526f6c654c6162656c2c0a09096d616e616765722e4174747269627574654b6579526f6c6549442c20726f6c652e537472696e6728292c0a09096d616e616765722e4174747269627574654b65794c6162656c2c206c6162656c2c0a09290a7d0a0a66756e6320656d6974526f6c654772616e7465644576656e7428726f6c65206d616e616765722e526f6c6549642c206163636f756e7420616464726573732c2073696e6365206d616e616765722e54696d65506f696e742c206e65774d656d62657220626f6f6c29207b0a09636861696e2e456d6974280a09096d616e616765722e4576656e7454797065526f6c654772616e7465642c0a09096d616e616765722e4174747269627574654b6579526f6c6549442c20726f6c652e537472696e6728292c0a09096d616e616765722e4174747269627574654b65794163636f756e742c206163636f756e742e537472696e6728292c0a09096d616e616765722e4174747269627574654b657953696e63652c2073696e63652e537472696e6728292c0a09096d616e616765722e4174747269627574654b65794e65774d656d6265722c20737472636f6e762e466f726d6174426f6f6c286e65774d656d626572292c0a09290a7d0a0a66756e6320656d6974526f6c655265766f6b65644576656e7428726f6c65206d616e616765722e526f6c6549642c206163636f756e74206164647265737329207b0a09636861696e2e456d6974280a09096d616e616765722e4576656e7454797065526f6c655265766f6b65642c0a09096d616e616765722e4174747269627574654b6579526f6c6549442c20726f6c652e537472696e6728292c0a09096d616e616765722e4174747269627574654b65794163636f756e742c206163636f756e742e537472696e6728292c0a09290a7d0a0a66756e6320656d6974526f6c6541646d696e4368616e6765644576656e7428726f6c65206d616e616765722e526f6c6549642c2061646d696e206d616e616765722e526f6c65496429207b0a09636861696e2e456d6974280a09096d616e616765722e4576656e7454797065526f6c6541646d696e4368616e6765642c0a09096d616e616765722e4174747269627574654b6579526f6c6549442c20726f6c652e537472696e6728292c0a09096d616e616765722e4174747269627574654b657941646d696e2c2061646d696e2e537472696e6728292c0a09290a7d0a0a66756e6320656d6974526f6c654772616e7444656c61794368616e6765644576656e7428726f6c65206d616e616765722e526f6c6549642c2064656c6179206d616e616765722e44656c61792c2073696e6365206d616e616765722e54696d65506f696e7429207b0a09636861696e2e456d6974280a09096d616e616765722e4576656e7454797065526f6c654772616e7444656c61794368616e6765642c0a09096d616e616765722e4174747269627574654b6579526f6c6549442c20726f6c652e537472696e6728292c0a09096d616e616765722e4174747269627574654b657944656c61792c2064656c61792e537472696e6728292c0a09096d616e616765722e4174747269627574654b657953696e63652c2073696e63652e537472696e6728292c0a09290a7d0a0a66756e6320656d6974546172676574436c6f7365644576656e742874617267657420737472696e672c20636c6f73656420626f6f6c29207b0a09636861696e2e456d6974280a09096d616e616765722e4576656e7454797065546172676574436c6f7365642c0a09096d616e616765722e4174747269627574654b65795461726765742c207461726765742c0a09096d616e616765722e4174747269627574654b6579436c6f7365642c20737472636f6e762e466f726d6174426f6f6c28636c6f736564292c0a09290a7d0a0a66756e6320656d697454617267657446756e6374696f6e526f6c65557064617465644576656e742874617267657420737472696e672c2073656c6563746f72206d616e616765722e53656c6563746f722c20726f6c65206d616e616765722e526f6c65496429207b0a09636861696e2e456d6974280a09096d616e616765722e4576656e745479706554617267657446756e6374696f6e526f6c65557064617465642c0a09096d616e616765722e4174747269627574654b65795461726765742c207461726765742c0a09096d616e616765722e4174747269627574654b657953656c6563746f722c20737472696e672873656c6563746f72292c0a09096d616e616765722e4174747269627574654b6579526f6c6549442c20726f6c652e537472696e6728292c0a09290a7d0a1a9a190a0b676574746572732e676e6f128a197061636b616765206163636573730a0a696d706f72742022676e6f2e6c616e642f702f6f6e626c6f632f6163636573732f6d616e61676572220a0a2f2f20486173526f6c652072657475726e7320726f6c65206d656d626572736869702073746174757320666f72206163636f756e742e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c313137322d4c313139370a66756e6320486173526f6c6528726f6c65206d616e616765722e526f6c6549642c206163636f756e74206164647265737329206d616e616765722e486173526f6c65526573756c74207b0a0972657475726e2061636365737353746174652e486173526f6c6528726f6c652c206163636f756e74290a7d0a0a2f2f20476574526f6c6541646d696e2072657475726e73207468652061646d696e20726f6c6520636f6e6669677572656420666f7220726f6c652e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c313131392d4c313132360a66756e6320476574526f6c6541646d696e28726f6c65206d616e616765722e526f6c65496429206d616e616765722e526f6c654964207b0a0972657475726e2061636365737353746174652e476574526f6c6541646d696e28726f6c65290a7d0a0a2f2f20476574526f6c654772616e7444656c61792072657475726e7320746865206772616e742064656c617920636f6e6669677572656420666f7220726f6c652e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c313133372d4c313134350a66756e6320476574526f6c654772616e7444656c617928726f6c65206d616e616765722e526f6c65496429206d616e616765722e44656c6179207b0a0972657475726e2061636365737353746174652e476574526f6c654772616e7444656c617928726f6c65290a7d0a0a2f2f2047657446756e6374696f6e526f6c652072657475726e73207468652073656c6563746f7220726f6c6520666f7220746172676574506174682e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c313039312d4c313130370a66756e632047657446756e6374696f6e526f6c65287461726765745061746820737472696e672c2073656c6563746f72206d616e616765722e53656c6563746f7229206d616e616765722e526f6c654964207b0a0972657475726e2047657454617267657446756e6374696f6e526f6c6528746172676574506174682c2073656c6563746f72290a7d0a0a2f2f2047657454617267657446756e6374696f6e526f6c652072657475726e73207468652073656c6563746f7220726f6c6520666f7220746172676574506174682e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c313039312d4c313130370a66756e632047657454617267657446756e6374696f6e526f6c65287461726765745061746820737472696e672c2073656c6563746f72206d616e616765722e53656c6563746f7229206d616e616765722e526f6c654964207b0a0972657475726e2061636365737353746174652e47657454617267657446756e6374696f6e526f6c6528746172676574506174682c2073656c6563746f72290a7d0a0a2f2f204973546172676574436c6f736564207265706f7274732077686574686572207461726765745061746820697320636c6f7365642e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c313038322d4c313038390a66756e63204973546172676574436c6f736564287461726765745061746820737472696e672920626f6f6c207b0a0972657475726e2061636365737353746174652e4973546172676574436c6f7365642874617267657450617468290a7d0a0a2f2f2043616e43616c6c207265706f72747320776865746865722063616c6c65722063616e2063616c6c2073656c6563746f72206f6e20746172676574506174682e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c313033332d4c313036380a66756e632043616e43616c6c287461726765745061746820737472696e672c2073656c6563746f72206d616e616765722e53656c6563746f722c2063616c6c6572206164647265737329206d616e616765722e43616e43616c6c526573756c74207b0a0972657475726e2061636365737353746174652e43616e43616c6c2863616c6c65722c20746172676574506174682c2073656c6563746f72290a7d0a0a2f2f204973417574686f72697a6564207265706f72747320696d6d6564696174652063616c6c20617574686f72697a6174696f6e20666f7220746172676574506174682e0a2f2f20556e696f6e207265666572656e63653a0a2f2f2068747470733a2f2f6769746875622e636f6d2f756e696f6e6c6162732f756e696f6e2f626c6f622f386366663066663334663662616134636462316534363530613038393835646430356465306335612f636f736d7761736d2f6163636573732d6d616e616765722f7372632f636f6e74726163742e7273234c313033332d4c313036380a66756e63204973417574686f72697a6564287461726765745061746820737472696e672c2073656c6563746f72206d616e616765722e53656c6563746f722c2063616c6c657220616464726573732920626f6f6c207b0a0972657475726e2061636365737353746174652e4973417574686f72697a65642863616c6c65722c20746172676574506174682c2073656c6563746f72290a7d0a0a2f2f2043616e41646d696e526f6c65207265706f72747320776865746865722063616c6c65722063616e2061646d696e697374657220726f6c652e0a66756e632043616e41646d696e526f6c6528726f6c65206d616e616765722e526f6c6549642c2063616c6c6572206164647265737329206d616e616765722e43616e43616c6c526573756c74207b0a0972657475726e2061636365737353746174652e43616e41646d696e526f6c6528726f6c652c2063616c6c6572290a7d0a0a2f2f2043616e4d616e616765546172676574207265706f72747320776865746865722063616c6c65722063616e206d616e6167652074617267657420636f6e66696775726174696f6e2e0a66756e632043616e4d616e6167655461726765742863616c6c6572206164647265737329206d616e616765722e43616e43616c6c526573756c74207b0a0972657475726e2061636365737353746174652e43616e4d616e6167655461726765742863616c6c6572290a7d0a1a9d010a0b676e6f6d6f642e746f6d6c128d016d6f64756c65203d2022676e6f2e6c616e642f722f6f6e626c6f632f6962632f756e696f6e2f616363657373220a676e6f203d2022302e39220a0a5b616464706b675d0a202063726561746f72203d2022673132767837646e3364717138396d7a3535307a77756e7667347177366570713733643963736179220a2020686569676874203d203137333337310a22230a132f676e6f2e4d656d5061636b61676554797065120c0a0a4d505573657250726f641a090801180120012a010022250801122101624fb4199892e99ea1c231bff7044cc13f3d37d4ec8ba131ceacbdc2e0dedc08222708011201011a20826138bb91dc3ccad0c0140b75509f20b7d2cc4a39e7a1cb9bb59e1fe7d7d3fa22250801122101e8189f97ee74a4f03571949040d7cbae23d2bf3bd1988a59828a1a0e85551e5d222508011221018597810966c1ccb68dc7d389a5e41717b7cf0d2c1d419dcf693676fb7794b157222708011201011a2027a3d2373fd45bae98336b1841ef9d73d969c7f55e00eed79cb57a3f52f4ff20222508011221019bffc19b7904631aec365415461d3d9ca2a2c3456acd18c7c2e58989c3d33cf0222708011201011a200784679145f06662eda7160ad3ecd389672690afdee97e41386665610fecf43022250801122101863554fbad7fa41d7e300607c486345df74a9b7f34dbdcf1ba29d44b33d9208a222708011201011a20dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d98622250801122101d9cc44531049ff1e7a878857f7f44845e3008e78f69df2cb3412f57b08813a4422250801122101ba0aa49bb798366b36e79042adf626d7ccb65d3d2d0d50b089063d034bfc0d77222708011201011a20dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d9862225080112210156f4b8dd34f1c04b78192c86c910a120c1022ac255d090e8486814255da7e4a0222708011201011a20dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986222708011201011a20dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986"
        ))
        .unwrap() else {
            panic!("expected a non-existence proof");
        };

        let CommitmentProof::Exist(main_store_exist) = CommitmentProof::decode_as::<Proto>(&hex!(
            "0a5a0a046d61696e1220858d767ec613cd00ab6c293614f89f2ec7da94778f9f178d17606af44a7d8c171a090801180120012a010022250801122101ccb581a002a493db462c72bc97aac085192a8ffb6a45fa5ee3cf22ee89eb1574"
        ))
        .unwrap() else {
            panic!("expected an existence proof");
        };

        let proof = MerkleProof {
            proofs: vec![
                CommitmentProof::Nonexist(nonexist),
                CommitmentProof::Exist(main_store_exist),
            ],
        };

        let root = hex!("30de965e9bcc6da505420edd144998fea570c7e2b77d44f62fdc9103f6b95806");
        let key = b"pkg:gno.land/r/nonexistent_realm_xyz123";

        crate::ibc_api::verify_non_membership(
            &proof,
            &GNO_SPECS,
            &MerkleRoot {
                hash: unionlabs::primitives::H256::new(root),
            },
            &[b"main".to_vec(), key.to_vec()],
        )
        .unwrap();
    }

    #[test]
    fn verify_non_membership_bptree_empty_child_fallback() {
        use crate::{ops, proof_specs::BPTREE_PROOF_SPEC};

        fn build_proof(embedded: [u8; 32]) -> (Vec<u8>, ExistenceProof) {
            let leaf = unionlabs::cosmos::ics23::leaf_op::LeafOp {
                hash: HashOp::Sha256,
                prehash_key: HashOp::NoHash,
                prehash_value: HashOp::Sha256,
                length: unionlabs::cosmos::ics23::length_op::LengthOp::VarProto,
                prefix: vec![0].into(),
            };
            let key = b"key-b".to_vec();
            let value = b"value-b".to_vec();
            let mut hash = ops::leaf_op::apply(&leaf, &key, &value).unwrap();

            // 4 ordinary branch-0 ops (arbitrary sibling content — shape alone
            // satisfies `has_padding`) plus 1 branch-1-shaped op carrying
            // `embedded` where a real left sibling hash would go.
            let path = [
                (vec![1u8], vec![0x11; 32]),
                (vec![1u8], vec![0x22; 32]),
                ([vec![1u8], embedded.to_vec()].concat(), vec![]),
                (vec![1u8], vec![0x33; 32]),
                (vec![1u8], vec![0x44; 32]),
            ];

            let mut inner_ops = Vec::new();
            for (prefix, suffix) in path {
                let inner_op = InnerOp {
                    hash: HashOp::Sha256,
                    prefix: prefix.into(),
                    suffix: suffix.into(),
                };
                hash = ops::inner_op::apply(&inner_op, &hash).unwrap();
                inner_ops.push(inner_op);
            }

            (
                hash,
                ExistenceProof {
                    key: key.into(),
                    value: value.into(),
                    leaf,
                    path: inner_ops,
                },
            )
        }

        let empty_child: [u8; 32] = BPTREE_PROOF_SPEC
            .inner_spec
            .empty_child
            .borrow()
            .as_ref()
            .try_into()
            .unwrap();

        // Correct placeholder: `left_branches_are_empty` recognizes it, the
        // leftmost path is accepted, and the queried key (left of "key-b") is
        // proven absent.
        let (root, right) = build_proof(empty_child);
        crate::verify::verify_non_membership(
            &BPTREE_PROOF_SPEC,
            &root,
            &NonExistenceProof {
                key: b"key-a".to_vec(),
                left: None,
                right: Some(right),
            },
            b"key-a",
        )
        .unwrap();

        // Same shape, wrong embedded bytes: `left_branches_are_empty` must
        // reject it since it no longer matches `empty_child`, and there is no
        // other way for a branch-1-shaped op to satisfy the branch-0 check.
        let (root, right) = build_proof([0xff; 32]);
        crate::verify::verify_non_membership(
            &BPTREE_PROOF_SPEC,
            &root,
            &NonExistenceProof {
                key: b"key-a".to_vec(),
                left: None,
                right: Some(right),
            },
            b"key-a",
        )
        .unwrap_err();
    }
}
