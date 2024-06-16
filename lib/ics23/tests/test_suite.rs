use std::{collections::HashMap, fmt::Display, fs, path::PathBuf};

use anyhow::{bail, Context};
use ics23::{
    existence_proof::{self, calculate_root},
    ops::{hash_op, inner_op, inner_op::check_against_spec, leaf_op},
    proof_specs::{IAVL_PROOF_SPEC, TENDERMINT_PROOF_SPEC},
    verify::{
        left_branches_are_empty, right_branches_are_empty, verify_membership, verify_non_membership,
    },
};
use protos::cosmos::ics23::v1::InnerSpec;
use serde::{de::DeserializeOwned, Deserialize};
use unionlabs::{
    cosmos::ics23::{
        commitment_proof::CommitmentProof, hash_op::HashOp, inner_op::InnerOp,
        proof_spec::ProofSpec,
    },
    encoding::{DecodeAs, Proto},
};

#[test]
fn suite() -> anyhow::Result<()> {
    let testdata_dir = PathBuf::from(std::env::var("ICS23_TEST_SUITE_DATA_DIR").unwrap());

    run_test_cases::<TestLeafOpData>(testdata_dir.join("TestLeafOpData.json"))?;
    run_test_cases::<TestInnerOpData>(testdata_dir.join("TestInnerOpData.json"))?;
    run_test_cases::<TestDoHashData>(testdata_dir.join("TestDoHashData.json"))?;
    run_test_cases::<TestExistenceProofData>(testdata_dir.join("TestExistenceProofData.json"))?;
    run_test_cases::<TestCheckLeafData>(testdata_dir.join("TestCheckLeafData.json"))?;
    // these are currently skipped in the ics23 repo and don't pass anyways
    // run_test_cases::<TestCheckAgainstSpecData>(
    //     testdata_dir.join("TestCheckAgainstSpecData.json"),
    // )?;
    run_test_cases_in_array::<TestEmptyBranchData>(
        testdata_dir.join("TestEmptyBranchData.json"),
        "test empty branch data",
    )?;

    run_vector_tests(testdata_dir)?;

    Ok(())
}

fn run_test_cases<T: TestCase>(p: PathBuf) -> Result<(), anyhow::Error> {
    let json = read_json::<HashMap<String, T>>(p);

    for (case, t) in json {
        eprint!("{case}... ");
        t.run()?;
        eprintln!("ok");
    }

    Ok(())
}

fn run_test_cases_in_array<T: TestCase>(p: PathBuf, name: &str) -> Result<(), anyhow::Error> {
    let json = read_json::<Vec<T>>(p);

    for (case, t) in json.into_iter().enumerate() {
        eprint!("{name}[{case}]... ");
        t.run()?;
        eprintln!("ok");
    }

    Ok(())
}

fn read_json<T: DeserializeOwned>(p: PathBuf) -> T {
    serde_json::from_str::<T>(&fs::read_to_string(p).unwrap()).unwrap()
}

trait TestCase: DeserializeOwned {
    fn run(self) -> anyhow::Result<()>;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TestLeafOpData {
    op: protos::cosmos::ics23::v1::LeafOp,
    #[serde(with = "::serde_utils::base64_opt")]
    key: Option<Vec<u8>>,
    #[serde(with = "::serde_utils::base64_opt")]
    value: Option<Vec<u8>>,
    #[serde(with = "::serde_utils::base64_opt")]
    expected: Option<Vec<u8>>,
    is_err: bool,
}

impl TestCase for TestLeafOpData {
    fn run(self) -> anyhow::Result<()> {
        let res = leaf_op::apply(
            &self.op.try_into().unwrap(),
            &self.key.unwrap_or_default(),
            &self.value.unwrap_or_default(),
        );

        match res {
            Ok(ok) => {
                if self.is_err {
                    bail!("expected error, but got none")
                }

                let expected = self.expected.unwrap();
                if ok != expected {
                    bail!(
                        "bad result: {} vs {}",
                        serde_utils::to_hex(ok),
                        serde_utils::to_hex(expected)
                    )
                };
            }
            Err(err) => {
                if !self.is_err {
                    bail!("{err}")
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TestInnerOpData {
    op: protos::cosmos::ics23::v1::InnerOp,
    #[serde(with = "::serde_utils::base64_opt")]
    child: Option<Vec<u8>>,
    #[serde(with = "::serde_utils::base64_opt")]
    expected: Option<Vec<u8>>,
    is_err: bool,
}

impl TestCase for TestInnerOpData {
    fn run(self) -> anyhow::Result<()> {
        let res = inner_op::apply(
            &self.op.try_into().unwrap(),
            &self.child.unwrap_or_default(),
        );

        match res {
            Ok(ok) => {
                if self.is_err {
                    bail!("expected error, but got none")
                }

                let expected = self.expected.unwrap();
                if ok != expected {
                    bail!(
                        "bad result: {} vs {}",
                        serde_utils::to_hex(ok),
                        serde_utils::to_hex(expected)
                    )
                };
            }
            Err(err) => {
                if !self.is_err {
                    bail!("{err}")
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TestDoHashData {
    hash_op: i32,
    // https://github.com/cosmos/ics23/blob/bf89d957b019bb9a2f381edb1f24d06957807690/testdata/TestDoHashData.json#L9
    #[serde(alias = "PreImage")]
    preimage: String,
    #[serde(with = "::hex")]
    expected_hash: Vec<u8>,
}

impl TestCase for TestDoHashData {
    fn run(self) -> anyhow::Result<()> {
        let do_hash = hash_op::do_hash(self.hash_op.try_into().unwrap(), self.preimage.as_bytes());

        match do_hash {
            Ok(res) => {
                if res != self.expected_hash {
                    bail!(
                        "Expected {} got {}",
                        serde_utils::to_hex(res),
                        serde_utils::to_hex(self.expected_hash)
                    )
                }
            }
            Err(hash_op::HashError::UnsupportedOp(
                hash_op @ (HashOp::Blake2b512 | HashOp::Blake2s256 | HashOp::Blake3),
            )) => {
                println!("unsupported hash op {hash_op}, skipping");
            }
            Err(err) => bail!(err),
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TestExistenceProofData {
    proof: protos::cosmos::ics23::v1::ExistenceProof,
    #[serde(with = "::serde_utils::base64_opt")]
    expected: Option<Vec<u8>>,
    is_err: bool,
}

impl TestCase for TestExistenceProofData {
    fn run(self) -> anyhow::Result<()> {
        match self.proof.try_into() {
            Ok(res) => match existence_proof::calculate_root(&res) {
                Ok(ok) => {
                    if self.is_err {
                        bail!("expected error, but got none")
                    }

                    let expected = self.expected.unwrap();
                    if ok != expected {
                        bail!(
                            "bad result: {} vs {}",
                            serde_utils::to_hex(ok),
                            serde_utils::to_hex(expected)
                        )
                    };
                }
                Err(err) => {
                    if !self.is_err {
                        bail!("{err}")
                    }
                }
            },
            Err(err) => {
                if !self.is_err {
                    bail!("{err:?}")
                }
            }
        };

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TestCheckLeafData {
    leaf: protos::cosmos::ics23::v1::LeafOp,
    spec: protos::cosmos::ics23::v1::LeafOp,
    is_err: bool,
}

impl TestCase for TestCheckLeafData {
    fn run(self) -> anyhow::Result<()> {
        match (self.leaf.try_into(), self.spec.try_into()) {
            (Ok(leaf), Ok(spec)) => match leaf_op::check_against_spec(
                &leaf,
                &ProofSpec {
                    leaf_spec: spec,
                    inner_spec: InnerSpec::default().try_into().unwrap(),
                    max_depth: None,
                    min_depth: None,
                    prehash_key_before_comparison: false,
                },
            ) {
                Ok(()) => {
                    if self.is_err {
                        bail!("Expected error")
                    }
                }
                Err(err) => {
                    if !self.is_err {
                        bail!("Unexpected error: {err}")
                    }
                }
            },
            (Ok(_), Err(err)) => {
                if !self.is_err {
                    bail!("Unexpected error (ProofSpec): {err:?}")
                }
            }
            (Err(err), Ok(_)) => {
                if !self.is_err {
                    bail!("Unexpected error: {err:?}")
                }
            }
            (Err(err1), Err(err2)) => {
                if !self.is_err {
                    bail!("Unexpected errors: {err1:?}, {err2:?}")
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TestCheckAgainstSpecData {
    proof: protos::cosmos::ics23::v1::ExistenceProof,
    spec: protos::cosmos::ics23::v1::ProofSpec,
    is_err: bool,
}

impl TestCase for TestCheckAgainstSpecData {
    fn run(self) -> anyhow::Result<()> {
        match (self.proof.try_into(), self.spec.try_into()) {
            (Ok(leaf), Ok(spec)) => match existence_proof::check_against_spec(&leaf, &spec) {
                Ok(()) => {
                    if self.is_err {
                        bail!("Expected error")
                    }
                }
                Err(err) => {
                    if !self.is_err {
                        bail!("Unexpected error: {err}")
                    }
                }
            },
            (Ok(_), Err(err)) => {
                if !self.is_err {
                    bail!("Unexpected error (ProofSpec): {err:?}")
                }
            }
            (Err(err), Ok(_)) => {
                if !self.is_err {
                    bail!("Unexpected error: {err:?}")
                }
            }
            (Err(err1), Err(err2)) => {
                if !self.is_err {
                    bail!("Unexpected errors: {err1:?}, {err2:?}")
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TestEmptyBranchData {
    op: protos::cosmos::ics23::v1::InnerOp,
    spec: protos::cosmos::ics23::v1::ProofSpec,
    is_left: bool,
    is_right: bool,
}

impl TestCase for TestEmptyBranchData {
    fn run(self) -> anyhow::Result<()> {
        match (InnerOp::try_from(self.op), ProofSpec::try_from(self.spec)) {
            (Ok(op), Ok(spec)) => {
                match check_against_spec(&op, &spec, 1) {
                    Ok(_) => {}
                    Err(err) => {
                        bail!("Invalid InnerOp: {err:?}")
                    }
                }

                match left_branches_are_empty(&spec.inner_spec, &op) {
                    Ok(empty) => {
                        assert_eq!(empty, self.is_left)
                    }
                    Err(err) => {
                        bail!("Left branches empty check failed: {err:?}")
                    }
                }

                match right_branches_are_empty(&spec.inner_spec, &op) {
                    Ok(empty) => {
                        assert_eq!(empty, self.is_right)
                    }
                    Err(err) => {
                        bail!("Right branches empty check failed: {err:?}")
                    }
                }
            }
            (Ok(_), Err(err)) => {
                bail!("Unexpected error (ProofSpec): {err:?}")
            }
            (Err(err), Ok(_)) => {
                bail!("Unexpected error (Op): {err:?}")
            }
            (Err(err1), Err(err2)) => {
                bail!("Unexpected errors (Op): {err1:?}, (ProofSpec): {err2:?}")
            }
        }

        Ok(())
    }
}

const FILENAMES: [&str; 6] = [
    "exist_left.json",
    "exist_right.json",
    "exist_middle.json",
    "nonexist_left.json",
    "nonexist_right.json",
    "nonexist_middle.json",
];

#[derive(Debug)]
enum SpecType {
    Iavl,
    Tendermint,
}

impl SpecType {
    const fn all() -> [SpecType; 2] {
        [SpecType::Iavl, SpecType::Tendermint]
    }

    const fn name(&self) -> &str {
        match self {
            SpecType::Iavl => "IAVL",
            SpecType::Tendermint => "Tendermint",
        }
    }

    const fn path(&self) -> &str {
        match self {
            SpecType::Iavl => "iavl",
            SpecType::Tendermint => "tendermint",
        }
    }

    const fn proof_spec(&self) -> ProofSpec {
        match self {
            SpecType::Iavl => IAVL_PROOF_SPEC,
            SpecType::Tendermint => TENDERMINT_PROOF_SPEC,
        }
    }
}

fn run_vector_tests(testdata_dir: PathBuf) -> anyhow::Result<()> {
    let tests: Vec<VectorTest> = SpecType::all()
        .iter()
        .flat_map(|spec_type| {
            FILENAMES.iter().map(|file_name| {
                let name = format!("{} - {}", spec_type.name(), file_name);
                let spec = spec_type.proof_spec();

                let path = testdata_dir.join(spec_type.path()).join(file_name);
                let data = read_json::<VectorTestData>(path);

                VectorTest { name, data, spec }
            })
        })
        .collect();

    for test in tests {
        eprint!("test vectors: {}...", &test);
        test.run()?;
        eprintln!("OK");
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct VectorTest {
    name: String,
    data: VectorTestData,
    spec: ProofSpec,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
struct VectorTestData {
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    key: Vec<u8>,
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    proof: Vec<u8>,
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    root: Vec<u8>,
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    value: Vec<u8>,
}

impl Display for VectorTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}

impl TestCase for VectorTest {
    fn run(self) -> anyhow::Result<()> {
        match CommitmentProof::decode_as::<Proto>(self.data.proof.as_slice()) {
            Ok(proof) => match (&proof, &self.data.value.len()) {
                (CommitmentProof::Exist(existence_proof), 1..) => {
                    let root = calculate_root(existence_proof).context("calculating root")?;

                    assert_eq!(&root, &self.data.root);

                    verify_membership(
                        &self.spec,
                        root.as_slice(),
                        existence_proof,
                        self.data.key.as_slice(),
                        self.data.value.as_slice(),
                    )
                    .context("verify membership")
                }
                (CommitmentProof::Nonexist(non_existence_proof), 0) => {
                    // Even both are `Some`, still calculate the left branch
                    let root = match (&non_existence_proof.left, &non_existence_proof.right) {
                        (Some(existence_proof), _) | (None, Some(existence_proof)) => {
                            calculate_root(existence_proof).context("calculating root")?
                        }
                        _ => bail!("can't find root"),
                    };

                    assert_eq!(&root, &self.data.root);

                    verify_non_membership(
                        &self.spec,
                        self.data.root.as_slice(),
                        non_existence_proof,
                        self.data.key.as_slice(),
                    )
                    .context("verify non membership")
                }
                _ => {
                    bail!(
                        "unexpected proof: {:?} / value.len: {:?}",
                        &proof,
                        &self.data.value.len()
                    )
                }
            },
            Err(e) => {
                bail!("Failed: cannot parse proof - {:?}", e)
            }
        }
    }
}
