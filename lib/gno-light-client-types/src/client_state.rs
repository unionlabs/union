use unionlabs::{
    cosmos::ics23::proof_spec::ProofSpec, google::protobuf::duration::Duration,
    ibc::core::client::height::Height,
};

use crate::Fraction;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    pub chain_id: String,
    pub trust_level: Fraction,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
    pub frozen_height: Option<Height>,
    pub latest_height: Height,
    pub proof_specs: Vec<ProofSpec>,
    // TODO: Remove? We don't use it?
    pub upgrade_path: Vec<String>,
    pub realm: String,
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use hex_literal::hex;
    use ics23::ibc_api::SDK_SPECS;
    use unionlabs::{
        cosmos::ics23::{
            hash_op::HashOp, inner_spec::InnerSpec, leaf_op::LeafOp, length_op::LengthOp,
        },
        encoding::{Bincode, Json},
        primitives::Bytes,
        test_utils::{assert_codec_iso, assert_codec_iso_bytes},
    };

    use super::*;

    fn mk_client_state() -> ClientState {
        ClientState {
            chain_id: "oogabooga".to_string(),
            trust_level: Fraction {
                numerator: 1,
                denominator: NonZeroU64::new(3).unwrap(),
            },
            trusting_period: Duration::new(12, 345).unwrap(),
            unbonding_period: Duration::new(67, 890).unwrap(),
            max_clock_drift: Duration::new(543, 21).unwrap(),
            frozen_height: Some(Height::default()),
            latest_height: Height::new(1337),
            proof_specs: [ProofSpec {
                leaf_spec: LeafOp {
                    hash: HashOp::Sha256,
                    prehash_key: HashOp::Sha512,
                    prehash_value: HashOp::Keccak256,
                    length: LengthOp::VarProto,
                    prefix: Bytes::new_static(&[1, 2, 3]),
                },
                inner_spec: InnerSpec {
                    child_order: [0.try_into().unwrap()].into_iter().collect(),
                    child_size: 123.try_into().unwrap(),
                    min_prefix_length: 456.try_into().unwrap(),
                    max_prefix_length: 789.try_into().unwrap(),
                    empty_child: Bytes::new_static(&[10, 11, 12]),
                    hash: HashOp::Bitcoin,
                },
                max_depth: None,
                min_depth: None,
                prehash_key_before_comparison: false,
            }]
            .to_vec(),
            upgrade_path: ["upgrade".to_owned(), "path".to_owned()].to_vec(),
            realm: "gno.land/r/oogabooga".to_owned(),
        }
    }

    #[test]
    fn bincode_iso() {
        assert_codec_iso::<_, Bincode>(&mk_client_state());
    }

    #[test]
    fn json_iso() {
        assert_codec_iso::<_, Json>(&mk_client_state());
    }

    #[test]
    fn bincode_bytes_iso() {
        // TODO: Use an actual gno client state once there's a client live
        // voyager rpc client-state union-testnet-10 6 --height 1206751
        let bz = hex!(
            "0700000000000000" "676e6f6c616e64" // chain_id
            "0100000000000000" "0300000000000000" // trust_level
            "0030e25c622e00000000000000000000" // trusting_period
            "00c0afd6913600000000000000000000" // unbonding_period
            "0070c9b28b0000000000000000000000" // max_clock_drift
            "00" // frozen_height
            "01" "0500000000000000" "19bf100000000000" // latest_height

            "0200000000000000" // proof_specs

            // leaf_spec
            "01000000" // hash
            "00000000" // prehash_key
            "01000000" // prehash_value
            "01000000" // length
            "0100000000000000" "00" // prefix
            // inner_spec
            "0200000000000000" "0000000000000000" "0100000000000000" // child_order
            "2100000000000000" // child_size
            "0400000000000000" // min_prefix_length
            "0c00000000000000" // max_prefix_length
            "0000000000000000" // empty_child
            "01000000" // hash
            "00" // max_depth
            "00" // min_depth
            "00" // prehash_key_before_comparison

            // leaf_spec
            "01000000" // hash
            "00000000" // prehash_key
            "01000000" // prehash_value
            "01000000" // length
            "0100000000000000" "00" // prefix
            // inner_spec
            "0200000000000000" "0000000000000000" "0100000000000000" // child_order
            "2000000000000000" // child_size
            "0100000000000000" // min_prefix_length
            "0100000000000000" // max_prefix_length
            "0000000000000000" // empty_child
            "01000000" // hash
            "00" // max_depth
            "00" // min_depth
            "00" // prehash_key_before_comparison

            "0200000000000000" // upgrade_path
            "0700000000000000" "75706772616465"
            "1000000000000000" "75706772616465644942435374617465"
            "1400000000000000" "676e6f2e6c616e642f722f6f6f6761626f6f6761" // realm
        );

        assert_codec_iso_bytes::<_, Bincode>(
            &ClientState {
                chain_id: "gnoland".to_owned(),
                trust_level: Fraction {
                    numerator: 1,
                    denominator: NonZeroU64::new(3).unwrap(),
                },
                trusting_period: Duration::new(51000, 0).unwrap(),
                unbonding_period: Duration::new(60000, 0).unwrap(),
                max_clock_drift: Duration::new(600, 0).unwrap(),
                frozen_height: None,
                latest_height: Height::new_with_revision(5, 1097497),
                proof_specs: SDK_SPECS.to_vec(),
                upgrade_path: ["upgrade".to_owned(), "upgradedIBCState".to_owned()].to_vec(),
                realm: "gno.land/r/oogabooga".to_owned(),
            },
            &bz,
        );
    }
}
