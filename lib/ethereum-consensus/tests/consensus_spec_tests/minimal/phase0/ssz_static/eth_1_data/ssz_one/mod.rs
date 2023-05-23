// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::ssz_static::Eth1DataTestCase;
use ethereum_consensus::phase0::minimal as spec;
use ssz_rs::prelude::*;

#[test]
fn test_case_0() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_0",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_1() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_1",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_10() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_10",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_11() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_11",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_12() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_12",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_13() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_13",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_14() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_14",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_15() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_15",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_16() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_16",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_17() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_17",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_18() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_18",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_19() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_19",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_2() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_2",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_20() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_20",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_21() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_21",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_22() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_22",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_23() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_23",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_24() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_24",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_25() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_25",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_26() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_26",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_27() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_27",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_28() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_28",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_29() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_29",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_3() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_3",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_4() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_4",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_5() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_5",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_6() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_6",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_7() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_7",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_8() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_8",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}

#[test]
fn test_case_9() {
    let test_case = Eth1DataTestCase::from(
        "consensus-spec-tests/tests/minimal/phase0/ssz_static/Eth1Data/ssz_one/case_9",
    );

    test_case.execute(|encoding| {
        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
        let serialized = ssz_rs::serialize(&data).unwrap();
        let root = data.hash_tree_root().unwrap();
        (serialized, root)
    });
}
