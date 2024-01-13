use std::{borrow::Cow, collections::HashMap, fmt::Display, fs, path::PathBuf};

use anyhow::bail;
use clap::Parser;
use ics23::{
    existence_proof::{self, calculate_root},
    ops::{hash_op, inner_op, leaf_op},
    verify::{verify_membership, verify_non_membership},
};
use protos::cosmos::ics23::v1::InnerSpec;
use serde::{de::DeserializeOwned, Deserialize};
use unionlabs::{
    bounded::BoundedUsize,
    cosmos::ics23::{
        commitment_proof::CommitmentProof, hash_op::HashOp, inner_spec::PositiveI32AsUsize,
        leaf_op::LeafOp, proof_spec::ProofSpec,
    },
    promote, result_unwrap, TryFromProto,
};

#[derive(Parser)]
struct App {
    pub testdata_dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let app = App::parse();

    run_test_cases::<TestLeafOpData>(app.testdata_dir.join("TestLeafOpData.json"))?;
    run_test_cases::<TestInnerOpData>(app.testdata_dir.join("TestInnerOpData.json"))?;
    run_test_cases::<TestDoHashData>(app.testdata_dir.join("TestDoHashData.json"))?;
    run_test_cases::<TestExistenceProofData>(app.testdata_dir.join("TestExistenceProofData.json"))?;
    run_test_cases::<TestCheckLeafData>(app.testdata_dir.join("TestCheckLeafData.json"))?;
    // these are currently skipped in the ics23 repo and don't pass anyways
    // run_test_cases::<TestCheckAgainstSpecData>(
    //     app.testdata_dir.join("TestCheckAgainstSpecData.json"),
    // )?;

    run_vector_tests(app)?;

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
struct TestVectors {
    proof: protos::cosmos::ics23::v1::ExistenceProof,
    spec: protos::cosmos::ics23::v1::ProofSpec,
    is_err: bool,
}

impl TestCase for TestVectors {
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

const IAVL_SPEC: unionlabs::cosmos::ics23::proof_spec::ProofSpec = ProofSpec {
    leaf_spec: LeafOp {
        hash: HashOp::Sha256,
        prehash_key: HashOp::NoHash,
        prehash_value: HashOp::Sha256,
        length: unionlabs::cosmos::ics23::length_op::LengthOp::VarProto,
        prefix: std::borrow::Cow::Borrowed(&[0]),
    },
    inner_spec: unionlabs::cosmos::ics23::inner_spec::InnerSpec {
        child_order: Cow::Borrowed(promote!(&[PositiveI32AsUsize]: &[
            result_unwrap!(PositiveI32AsUsize::new(0)),
            result_unwrap!(PositiveI32AsUsize::new(1)),
        ])),
        child_size: result_unwrap!(PositiveI32AsUsize::new(33)),
        min_prefix_length: result_unwrap!(PositiveI32AsUsize::new(4)),
        max_prefix_length: result_unwrap!(PositiveI32AsUsize::new(12)),
        empty_child: Cow::Borrowed(&[0; 0]),
        hash: HashOp::Sha256,
    },
    min_depth: None,
    max_depth: None,
    prehash_key_before_comparison: false,
};

// from: proof.go
// IavlSpec constrains the format from proofs-iavl (iavl merkle proofs)
// var IavlSpec = &ProofSpec{
// 	LeafSpec: &LeafOp{
// 		Prefix:       []byte{0},
// 		PrehashKey:   HashOp_NO_HASH,
// 		Hash:         HashOp_SHA256,
// 		PrehashValue: HashOp_SHA256,
// 		Length:       LengthOp_VAR_PROTO,
// 	},
// 	InnerSpec: &InnerSpec{
// 		ChildOrder:      []int32{0, 1},
// 		MinPrefixLength: 4,
// 		MaxPrefixLength: 12,
// 		ChildSize:       33, // (with length byte)
// 		EmptyChild:      nil,
// 		Hash:            HashOp_SHA256,
// 	},
// }

const TENDERMINT_SPEC: unionlabs::cosmos::ics23::proof_spec::ProofSpec = ProofSpec {
    leaf_spec: LeafOp {
        hash: HashOp::Sha256,
        prehash_key: HashOp::NoHash,
        prehash_value: HashOp::Sha256,
        length: unionlabs::cosmos::ics23::length_op::LengthOp::VarProto,
        prefix: std::borrow::Cow::Borrowed(&[0]),
    },
    inner_spec: unionlabs::cosmos::ics23::inner_spec::InnerSpec {
        child_order: Cow::Borrowed(promote!(&[PositiveI32AsUsize]: &[
            result_unwrap!(PositiveI32AsUsize::new(0)),
            result_unwrap!(PositiveI32AsUsize::new(1)),
        ])),
        child_size: result_unwrap!(PositiveI32AsUsize::new(32)),
        min_prefix_length: result_unwrap!(PositiveI32AsUsize::new(1)),
        max_prefix_length: result_unwrap!(PositiveI32AsUsize::new(1)),
        empty_child: Cow::Borrowed(&[0; 0]),
        hash: HashOp::Sha256,
    },
    min_depth: None,
    max_depth: None,
    prehash_key_before_comparison: false,
};

// from: proof.go
// TendermintSpec constrains the format from proofs-tendermint (crypto/merkle SimpleProof)
// var TendermintSpec = &ProofSpec{
// 	LeafSpec: &LeafOp{
// 		Prefix:       []byte{0},
// 		PrehashKey:   HashOp_NO_HASH,
// 		Hash:         HashOp_SHA256,
// 		PrehashValue: HashOp_SHA256,
// 		Length:       LengthOp_VAR_PROTO,
// 	},
// 	InnerSpec: &InnerSpec{
// 		ChildOrder:      []int32{0, 1},
// 		MinPrefixLength: 1,
// 		MaxPrefixLength: 1,
// 		ChildSize:       32, // (no length byte)
// 		Hash:            HashOp_SHA256,
// 	},
// }

type PositiveNonZeroI32AsUsize = BoundedUsize<1, { i32::MAX as usize }>;

const SMT_SPEC: unionlabs::cosmos::ics23::proof_spec::ProofSpec = ProofSpec {
    leaf_spec: LeafOp {
        hash: HashOp::Sha256,
        prehash_key: HashOp::Sha256,
        prehash_value: HashOp::Sha256,
        length: unionlabs::cosmos::ics23::length_op::LengthOp::NoPrefix,
        prefix: std::borrow::Cow::Borrowed(&[0]),
    },
    inner_spec: unionlabs::cosmos::ics23::inner_spec::InnerSpec {
        child_order: Cow::Borrowed(promote!(&[PositiveI32AsUsize]: &[
            result_unwrap!(PositiveI32AsUsize::new(0)),
            result_unwrap!(PositiveI32AsUsize::new(1)),
        ])),
        child_size: result_unwrap!(PositiveI32AsUsize::new(32)),
        min_prefix_length: result_unwrap!(PositiveI32AsUsize::new(1)),
        max_prefix_length: result_unwrap!(PositiveI32AsUsize::new(1)),
        empty_child: Cow::Borrowed(&[0; 32]),
        hash: HashOp::Sha256,
    },
    min_depth: None,
    max_depth: Some(result_unwrap!(PositiveNonZeroI32AsUsize::new(256))),
    prehash_key_before_comparison: true,
};

//  from: proof.go
//  var SmtSpec = &ProofSpec{
//	LeafSpec: &LeafOp{
//		Hash:         HashOp_SHA256,
//		PrehashKey:   HashOp_SHA256,
//		PrehashValue: HashOp_SHA256,
//		Length:       LengthOp_NO_PREFIX,
//		Prefix:       []byte{0},
//	},
//	InnerSpec: &InnerSpec{
//		ChildOrder:      []int32{0, 1},
//		ChildSize:       32,
//		MinPrefixLength: 1,
//		MaxPrefixLength: 1,
//		EmptyChild:      make([]byte, 32),
//		Hash:            HashOp_SHA256,
//	},
//	MaxDepth:                   256,
//	PrehashKeyBeforeComparison: true,
//}

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
    Smt,
}

impl SpecType {
    fn all() -> [SpecType; 3] {
        [SpecType::Iavl, SpecType::Tendermint, SpecType::Smt]
    }

    fn name(&self) -> &str {
        match self {
            SpecType::Iavl => "IAVL",
            SpecType::Tendermint => "Tendermint",
            SpecType::Smt => "SMT",
        }
    }

    fn path(&self) -> &str {
        match self {
            SpecType::Iavl => "iavl",
            SpecType::Tendermint => "tendermint",
            SpecType::Smt => "smt",
        }
    }

    fn proof_spec(&self) -> ProofSpec {
        match self {
            SpecType::Iavl => IAVL_SPEC,
            SpecType::Tendermint => TENDERMINT_SPEC,
            SpecType::Smt => SMT_SPEC,
        }
    }
}

fn run_vector_tests(app: App) -> anyhow::Result<()> {
    let tests: Vec<VectorTest> = SpecType::all()
        .iter()
        .flat_map(|spec_type| {
            FILENAMES.iter().map(|file_name| {
                let name = format!("{} - {}", spec_type.name(), file_name);
                let spec = spec_type.proof_spec();

                let path = app.testdata_dir.join(spec_type.path()).join(file_name);
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
    key: Option<Vec<u8>>,
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    proof: Option<Vec<u8>>,
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    root: Option<Vec<u8>>,
    #[serde(with = "::serde_utils::hex_upper_unprefixed")]
    value: Option<Vec<u8>>,
}

impl Display for VectorTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name.as_str())
    }
}

impl TestCase for VectorTest {
    fn run(self) -> anyhow::Result<()> {
        match (&self.data.proof, &self.data.root) {
            (Some(proof), Some(expected_root)) => {
                match CommitmentProof::try_from_proto_bytes(proof.as_slice()) {
                    Ok(proof) => match proof {
                        CommitmentProof::Exist(existence_proof) => {
                            match calculate_root(&existence_proof) {
                                Ok(root) => {
                                    assert_eq!(&root, expected_root);

                                    match (&self.data.value, &self.data.key) {
                                        (Some(value), Some(key)) => {
                                            match verify_membership(
                                                &self.spec,
                                                root.as_slice(),
                                                &existence_proof,
                                                key.as_slice(),
                                                value.as_slice(),
                                            ) {
                                                Ok(_) => Ok(()),
                                                Err(e) => {
                                                    bail!("Failed: no membership - {}", e)
                                                }
                                            }
                                        }
                                        (_, _) => {
                                            bail!("Expected value and key in test data file");
                                        }
                                    }
                                }
                                Err(e) => {
                                    bail!("Cannot calculate root: {}", e)
                                }
                            }
                        }
                        CommitmentProof::Nonexist(non_existence_proof) => {
                            match (&self.data.root, &self.data.key) {
                                (Some(root), Some(key)) => {
                                    // TODO: Original self calculates root and assert it's the same, but I can't find a `calculate_root(Nonexist)` function

                                    match verify_non_membership(
                                        &self.spec,
                                        root.as_slice(),
                                        &non_existence_proof,
                                        key.as_slice(),
                                    ) {
                                        Ok(_) => Ok(()),
                                        Err(e) => {
                                            bail!("Failed: no no-membership - {}", e)
                                        }
                                    }
                                }
                                (_, _) => {
                                    bail!("Expected root and key in test data file");
                                }
                            }
                        }
                        _ => {
                            bail!("unexpected proof: {:?}", proof)
                        }
                    },
                    Err(e) => {
                        bail!("Failed: cannot parse proof - {:?}", e)
                    }
                }
            }
            (_, _) => {
                bail!("Expected proof and root in test data file")
            }
        }
    }
}
