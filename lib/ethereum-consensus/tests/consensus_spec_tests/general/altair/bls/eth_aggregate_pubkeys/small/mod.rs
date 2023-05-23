// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

use crate::spec_test_runners::bls::EthAggregatePubkeysTestCase;
use crate::test_utils::TestCase;

#[test]
fn test_eth_aggregate_pubkeys_empty_list() {
    let  test_case = EthAggregatePubkeysTestCase::from("consensus-spec-tests/tests/general/altair/bls/eth_aggregate_pubkeys/small/eth_aggregate_pubkeys_empty_list");

    test_case.execute();
}

#[test]
fn test_eth_aggregate_pubkeys_infinity_pubkey() {
    let  test_case = EthAggregatePubkeysTestCase::from("consensus-spec-tests/tests/general/altair/bls/eth_aggregate_pubkeys/small/eth_aggregate_pubkeys_infinity_pubkey");

    test_case.execute();
}

#[test]
fn test_eth_aggregate_pubkeys_valid_e_235_e_92_e_3_a_313_f_43() {
    let  test_case = EthAggregatePubkeysTestCase::from("consensus-spec-tests/tests/general/altair/bls/eth_aggregate_pubkeys/small/eth_aggregate_pubkeys_valid_e235e92e3a313f43");

    test_case.execute();
}

#[test]
fn test_eth_aggregate_pubkeys_valid_ea_0_e_3_cc_74_e_1_de_899() {
    let  test_case = EthAggregatePubkeysTestCase::from("consensus-spec-tests/tests/general/altair/bls/eth_aggregate_pubkeys/small/eth_aggregate_pubkeys_valid_ea0e3cc74e1de899");

    test_case.execute();
}

#[test]
fn test_eth_aggregate_pubkeys_valid_f_15974_ec_693571_cf() {
    let  test_case = EthAggregatePubkeysTestCase::from("consensus-spec-tests/tests/general/altair/bls/eth_aggregate_pubkeys/small/eth_aggregate_pubkeys_valid_f15974ec693571cf");

    test_case.execute();
}

#[test]
fn test_eth_aggregate_pubkeys_valid_pubkeys() {
    let  test_case = EthAggregatePubkeysTestCase::from("consensus-spec-tests/tests/general/altair/bls/eth_aggregate_pubkeys/small/eth_aggregate_pubkeys_valid_pubkeys");

    test_case.execute();
}

#[test]
fn test_eth_aggregate_pubkeys_x_40_pubkey() {
    let  test_case = EthAggregatePubkeysTestCase::from("consensus-spec-tests/tests/general/altair/bls/eth_aggregate_pubkeys/small/eth_aggregate_pubkeys_x40_pubkey");

    test_case.execute();
}

#[test]
fn test_eth_aggregate_pubkeys_zero_pubkey() {
    let  test_case = EthAggregatePubkeysTestCase::from("consensus-spec-tests/tests/general/altair/bls/eth_aggregate_pubkeys/small/eth_aggregate_pubkeys_zero_pubkey");

    test_case.execute();
}
