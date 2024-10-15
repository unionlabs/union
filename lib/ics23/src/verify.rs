use std::borrow::{Borrow, Cow};

use unionlabs::cosmos::ics23::{
    existence_proof::ExistenceProof,
    hash_op::HashOp,
    inner_op::InnerOp,
    inner_spec::{InnerSpec, PositiveI32AsUsize},
    non_existence_proof::NonExistenceProof,
    proof_spec::ProofSpec,
};

use crate::{
    existence_proof::{self, CalculateRootError, SpecMismatchError},
    ops::hash_op::{do_hash, HashError},
};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
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
    #[error(
        "calculated and given root doesn't match ({calculated_root}, {given_root})",
        calculated_root = serde_utils::to_hex(calculated_root),
        given_root = serde_utils::to_hex(given_root)
    )]
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
    #[error("neighbor search failure ({0})")]
    NeighborSearch(NeighborSearchError),
    #[error(transparent)]
    Hash(#[from] HashError),
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum VerifyMembershipError {
    #[error("existence proof verification failed, ({0})")]
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
    let left_ops = |left: &ExistenceProof| -> Result<(), VerifyError> {
        verify_existence_proof(left, spec, root, &left.key, &left.value)?;

        if key_for_comparison(spec, key)? <= key_for_comparison(spec, &left.key)? {
            return Err(VerifyError::KeyIsNotRightOfLeftProof);
        }

        if !is_right_most(&spec.inner_spec, &left.path).map_err(VerifyError::NeighborSearch)? {
            return Err(VerifyError::RightProofMissing);
        }

        Ok(())
    };

    let right_ops = |right: &ExistenceProof| -> Result<(), VerifyError> {
        verify_existence_proof(right, spec, root, &right.key, &right.value)?;

        if key_for_comparison(spec, key)? >= key_for_comparison(spec, &right.key)? {
            return Err(VerifyError::KeyIsNotLeftOfRightProof);
        }

        if !is_left_most(&spec.inner_spec, &right.path).map_err(VerifyError::NeighborSearch)? {
            return Err(VerifyError::LeftProofMissing);
        }

        Ok(())
    };

    match (&non_existence_proof.left, &non_existence_proof.right) {
        (None, Some(right)) => right_ops(right)?,
        (Some(left), None) => left_ops(left)?,
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
        if Some(spec.empty_child.borrow()) != op.prefix.get(from..from + spec.child_size.inner()) {
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
        .find(|(_, &elem)| elem.inner() == branch)
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
            existence_proof_key: existence_proof.key.to_vec(),
        });
    }

    if value != &existence_proof.value[..] {
        return Err(VerifyError::ValueAndExistenceProofValueMismatch {
            value: value.into(),
            existence_proof_value: existence_proof.value.to_vec(),
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

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use unionlabs::{
        cosmos::ics23::commitment_proof::CommitmentProof,
        encoding::{DecodeAs, EncodeAs as _, Proto},
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
        let proof = hex!("0adb030a14303142424373615a55715146735259436c6a5767121e76616c75655f666f725f303142424373615a55715146735259436c6a57671a090801180120012a0100222708011201011a20cb3131cd98b069efcc0e8c7e68da47370adbff32266d7fcd1b0580fdf3961266222708011201011a2021d1205c1f8537205e8fb4b176f960b459d9131669968d59c456442f7673b68b222708011201011a20b82a0e7f4434b3cedb87ea83eb5a70c7dc664c77b2fe21c6245f315e58fdf745222708011201011a20bf0657a0e6fbd8f2043eb2cf751561adcf50547d16201224133eeb8d38145229222708011201011a206d47c03df91a4a0252055d116439d34b5b73f3a24d5cb3cf0d4b08caa540cac4222708011201011a20d5d2926993fa15c7410ac4ee1f1d81afddfb0ab5f6f4706b05f407bc01638149222708011201011a20540719b26a7301ad012ac45ebe716679e5595e5570d78be9b6da8d8591afb374222708011201011a20fccaaa9950730e80b9ccf75ad2cfeab26ae750b8bd6ac1ff1c7a7502f3c64be2222708011201011a20ecb61a6d70accb79c2325fb0b51677ed1561c91af5e10578c8294002fbb3c21e222708011201011a201b3bc1bd8d08af9f6199de84e95d646570cbd9b306a632a5acf617cbd7d1ab0a");
        let root = hex!("c569a38a5775bbda2051c34ae00894186f837c39d11dca55495b9aed14f17ddf");
        let key = hex!("303142424373615a55715146735259436c6a5767");
        let value = hex!("76616c75655f666f725f303142424373615a55715146735259436c6a5767");

        assert_eq!(ensure_existent(&proof, &root, &key, &value), Ok(()));
    }

    #[test]
    fn verify_membership_middle() {
        let proof = hex!("0ad1030a14513334656d766f39447145585735325257523835121e76616c75655f666f725f513334656d766f394471455857353252575238351a090801180120012a010022250801122101e231d775380f2d663651e213cc726660e2ce0a2f2e9ee12cbb7df32294104a8c222708011201011a2014af194c63500236e52cc290ab24244fab39a520ece7e20fa93f4c9ff80c6626222508011221017966d2ead34418db2eaa04c0dffb9316805e8a0d421d1270c8954c35ee3221382225080112210172339e20a49bb16795a99bd905b47f99c45e5e5a9e6b7fb223dc8fe6751e1bda222708011201011a2053dd1ecc25ff906a0ef4db37ee068f3d8ad6d1d49913eefb847a675a681c5ffa222708011201011a20de90f9951a19497be7e389e02aa79e26faf77080e740e8743249a17a537f287d22250801122101ad4e53e981afc5a71e34ab0c4ffbccf1b468414d9d0939bd08edbd2461bc944a222708011201011a209b4cf89c3995b9dd66d58ab088846b2c6b59c52c6d10ec1d759ca9e9aa5eef5c222508011221013928a078bd66ab3949f5b1846b6d354dbdc1968a416607c7d91555ca26716667222708011201011a20d2d82cf8915b9ae6f92c7eae343e37d312ace05e654ce47acdf57d0a5490b873");
        let root = hex!("494b16e3a64a85df143b2881bdd3ec94c3f8e18b343e8ff9c2d61afd05d040c8");
        let key = hex!("513334656d766f39447145585735325257523835");
        let value = hex!("76616c75655f666f725f513334656d766f39447145585735325257523835");

        assert_eq!(ensure_existent(&proof, &root, &key, &value), Ok(()));
    }

    #[test]
    fn verify_membership_right() {
        let proof = hex!("0aab020a147a785a4e6b534c64634d655657526c7658456644121e76616c75655f666f725f7a785a4e6b534c64634d655657526c76584566441a090801180120012a0100222508011221012634b831468dbafb1fc61a979c348ff8462da9a7d550191a6afc916ade16cc9922250801122101ab814d419bfc94ee9920d0ce993ce5da011e43613daf4b6f302855760083d7dd222508011221015a1568c73eaeaba567a6b2b2944b0e9a0228c931884cb5942f58ed835b8a7ac522250801122101a171412db5ee84835ef247768914e835ff80b7711e4aa8060871c2667ec3ea2922250801122101f9c2491884de24fb61ba8f358a56b306a8989bd35f1f8a4c8dabce22f703cc14222508011221012f12a6aa6270eff8a1628052938ff5e36cfcc5bf2eaedc0941ee46398ebc7c38");
        let root = hex!("f54227f1a7d90aa2bf7931066196fd3072b7fe6b1fbd49d1e26e85a90d9541bb");
        let key = hex!("7a785a4e6b534c64634d655657526c7658456644");
        let value = hex!("76616c75655f666f725f7a785a4e6b534c64634d655657526c7658456644");

        assert_eq!(ensure_existent(&proof, &root, &key, &value), Ok(()));
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_left.json
    #[test]
    fn verify_non_membership_left() {
        let proof = hex!("12e4030a04010101011adb030a143032615465366472483456706f4f583245507137121e76616c75655f666f725f3032615465366472483456706f4f5832455071371a090801180120012a0100222708011201011a20b843481496dc10561056b63ec8f726f3357395b610355b25082f5768b2073e91222708011201011a20d5281fdd872060e89173d4de1100fa6c96f778467df66abb10cf3b1f5821f182222708011201011a20eb981020433d929c6275ad772accf2e6aa916db97e31d2f26d0b6b07b444bbef222708011201011a204a40e813132aff60b64ba9d109548ab39459ad48a203ab8d3455dd842a7ab1da222708011201011a208f354a84ce1476e0b9cca92e65301a6435b1f242c2f53f943b764a4f326a71c7222708011201011a20ac6451617a6406005035dddad36657fde5312cc4d67d69ca1464611847c10cfb222708011201011a2023c1d1dd62002a0e2efcc679196589a4337234dcd209cb449cc3ac10773b60e0222708011201011a203b11c267328ba761ddc630dd5ef7642aeda05f180539fe93c0ca57729705bc46222708011201011a205ff2e1933be704539463c264b157ff2b8d9960813bd36c69c5208d57e3b1e07e222708011201011a20c4a79e6c0cbf60fb8e5bf940db4c444b7e442951b69c840db38cf28c8aa008be");
        let root = hex!("4e2e78d2da505b7d0b00fda55a4b048eed9a23a7f7fc3d801f20ce4851b442aa");
        let key = hex!("01010101");

        assert_eq!(ensure_non_existent(&proof, &root, &key), Ok(()));
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_middle.json
    #[test]
    fn verify_non_membership_middle() {
        let proof = hex!("12c0070a14544f31483668784a4b667136547a56767649ffff12cf030a14544f31483668784a4b667136547a567676497747121e76616c75655f666f725f544f31483668784a4b667136547a5676764977471a090801180120012a01002225080112210143e19cb5e5dab017734caa78a2e2bccbb4797b7dc5a91abeab630c66fa6b162522250801122101b575404a1bb42b0fef8ae7f217af88aec769f7d66b5bc4b2913e74d651365473222508011221017c22dc50e866f9a1dce517ea01621161cecd70f4bdcd024b5a392746a1c8dc2622250801122101578105344f2c98c323ba0b8ca31e75aaa2b865cc389681e300b14d1c20713796222708011201011a20895c070c14546ecef7f5cb3a4bda1fd436a0ff99190f90bd037cbeaf52b2ffc1222708011201011a20f7571fca06ac4387c3eae5469c152427b797abb55fa98727eacbd5c1c91b5fb4222508011221015056e6472f8e5c5c9b8881c5f0e49601e9eca31f3e1766aa69c2dc9c6d9112be222708011201011a206c74439556c5edb5aa693af410d3718dbb613d37799f2f4e8ff304a8bfe3351b22250801122101253014334c7b8cd78436979554f7890f3dc1c971925ea31b48fc729cd179c701222708011201011a20b81c19ad4b5d8d15f716b91519bf7ad3d6e2289f9061fd2592a8431ea97806fe1ad5030a14544f433344683150664f76657538585166635778121e76616c75655f666f725f544f433344683150664f766575385851666357781a090801180120012a0100222708011201011a20415d4cfaed0bfc98ac32acc219a8517bfa1983a15cc742e8b2f860167484bd46222708011201011a2098d853d9cc0ee1d2162527f660f2b90ab55b13e5534f1b7753ec481d7901d3ec222708011201011a20b5113e6000c5411b7cfa6fd09b6752a43de0fcd3951ed3b154d162deb53224a2222708011201011a208ce18cd72cc83511cb8ff706433f2fa4208c85b9f4c8d0ed71a614f24b89ae6c22250801122101c611244fe6b5fda4257615902eb24c14efcd9708c7c875d1ac5e867767aa1eab222708011201011a20f7571fca06ac4387c3eae5469c152427b797abb55fa98727eacbd5c1c91b5fb4222508011221015056e6472f8e5c5c9b8881c5f0e49601e9eca31f3e1766aa69c2dc9c6d9112be222708011201011a206c74439556c5edb5aa693af410d3718dbb613d37799f2f4e8ff304a8bfe3351b22250801122101253014334c7b8cd78436979554f7890f3dc1c971925ea31b48fc729cd179c701222708011201011a20b81c19ad4b5d8d15f716b91519bf7ad3d6e2289f9061fd2592a8431ea97806fe");
        let root = hex!("4bf28d948566078c5ebfa86db7471c1541eab834f539037075b9f9e3b1c72cfc");
        let key = hex!("544f31483668784a4b667136547a56767649ffff");

        assert_eq!(ensure_non_existent(&proof, &root, &key), Ok(()));
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_right.json
    #[test]
    fn verify_non_membership_right() {
        let proof = hex!("12a9030a04ffffffff12a0030a147a774e4d4a456f7932674253586277666e63504a121e76616c75655f666f725f7a774e4d4a456f7932674253586277666e63504a1a090801180120012a01002225080112210178a215355c17371583418df95773476b347a853f6eae317677721e0c24e78ad2222508011221015e2cf893e7cd70251eb4debd855c8c9a92f6e0a1fd931cf41e0575846ab174e822250801122101414bae883f8133f0201a2791dafeaef3daa24a6631b3f9402de3a4dc658fd035222508011221012e2829beee266a814af4db08046f4575b011e5ec9d2d93c1510c3cc7d8219edc22250801122101f8286597078491ae0ef61264c218c6e167e4e03f1de47945d9ba75bb41deb81a22250801122101dea6a53098d11ce2138cbcae26b392959f05d7e1e24b9547584571012280f289222508011221010a8e535094d18b2120c38454b445d9accf3f1b255690e6f3d48164ae73b4c775222508011221012cbb518f52ec1f8e26dd36587f29a6890a11c0dd3f94e7a28546e695f296d3a722250801122101839d9ddd9dadf41c0ecfc3f7e20f57833b8fb5bcb703bef4f97910bbe5b579b9");
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
                hash: unionlabs::hash::H256::new(root),
            },
            &[b"ibc".to_vec(), key.to_vec()],
            ConnectionEnd {
                client_id: ClientId::new(0),
                client_type: "08-wasm".to_owned(),
                versions: vec![Version {
                    identifier: "1".to_string(),
                    features: vec![Order::Unordered],
                }],
                state: unionlabs::ibc::core::connection::state::State::Tryopen,
                counterparty: unionlabs::ibc::core::connection::counterparty::Counterparty {
                    client_id: ClientId::new(0),
                    client_type: "cometbls".to_owned(),
                    connection_id: Some(ConnectionId::new(0)),
                    prefix: unionlabs::ibc::core::commitment::merkle_prefix::MerklePrefix {
                        key_prefix: b"ibc".to_vec(),
                    },
                },
                delay_period: 0,
            }
            .encode_as::<Proto>(),
        )
        .unwrap();
    }
}
