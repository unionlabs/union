use unionlabs::{
    cosmos::ics23::proof_spec::ProofSpec, google::protobuf::duration::Duration,
    ibc::core::client::height::Height, primitives::H256,
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
    pub upgrade_path: Vec<String>,
    /// For clients that connect to the cosmwasm implementation of ibc-union, the contract address of the IBC host is required in order to verify storage proofs. For clients connecting to IBC classic, this field is not required and can be ignored during client creation and migration.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "H256::is_zero")
    )]
    pub contract_address: H256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{
        cosmos::ics23::proof_spec::TryFromProofSpecError, errors::MissingField,
        google::protobuf::duration::DurationError, impl_proto_via_try_from_into, primitives::H256,
        required,
    };

    impl_proto_via_try_from_into!(ClientState => protos::ibc::lightclients::tendermint::v1::ClientState);

    use crate::{client_state::ClientState, fraction};

    impl From<ClientState> for protos::ibc::lightclients::tendermint::v1::ClientState {
        fn from(value: ClientState) -> Self {
            #[allow(deprecated)]
            Self {
                chain_id: value.chain_id,
                trust_level: Some(value.trust_level.into()),
                trusting_period: Some(value.trusting_period.into()),
                unbonding_period: Some(value.unbonding_period.into()),
                max_clock_drift: Some(value.max_clock_drift.into()),
                frozen_height: Some(value.frozen_height.unwrap_or_default().into()),
                latest_height: Some(value.latest_height.into()),
                proof_specs: value.proof_specs.into_iter().map(Into::into).collect(),
                upgrade_path: value.upgrade_path,
                // these are default true: https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-026-ibc-client-recovery-mechanisms.md#decision
                allow_update_after_expiry: true,
                allow_update_after_misbehaviour: true,
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid trust level")]
        TrustLevel(#[source] fraction::proto::Error),
        #[error("invalid trusting period")]
        TrustingPeriod(#[source] DurationError),
        #[error("invalid unbonding period")]
        UnbondingPeriod(#[source] DurationError),
        #[error("invalid max clock drift")]
        MaxClockDrift(#[source] DurationError),
        #[error("invalid proof specs")]
        ProofSpecs(#[source] TryFromProofSpecError),
    }

    impl TryFrom<protos::ibc::lightclients::tendermint::v1::ClientState> for ClientState {
        type Error = Error;

        fn try_from(
            value: protos::ibc::lightclients::tendermint::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                chain_id: value.chain_id,
                trust_level: required!(value.trust_level)?
                    .try_into()
                    .map_err(Error::TrustLevel)?,
                trusting_period: required!(value.trusting_period)?
                    .try_into()
                    .map_err(Error::TrustingPeriod)?,
                unbonding_period: required!(value.unbonding_period)?
                    .try_into()
                    .map_err(Error::TrustingPeriod)?,
                max_clock_drift: required!(value.max_clock_drift)?
                    .try_into()
                    .map_err(Error::TrustingPeriod)?,
                frozen_height: Some(value.frozen_height.unwrap_or_default().into()),
                latest_height: required!(value.latest_height)?.into(),
                proof_specs: value
                    .proof_specs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::ProofSpecs)?,
                upgrade_path: value.upgrade_path,
                // contract address is not needed for native impl which uses proto
                contract_address: H256::default(),
            })
        }
    }
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
        encoding::{Bincode, Json, Proto},
        google::protobuf::any::Any,
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
            contract_address: H256::default(),
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
    fn proto_iso() {
        assert_codec_iso::<_, Proto>(&mk_client_state());
    }

    #[test]
    fn bincode_bytes_iso() {
        // voyager rpc client-state union-testnet-10 6 --height 1206751
        let bz = hex!(
            "0a00000000000000" "62626e2d746573742d35" // chain_id
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
            "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4" // contract_address
        );

        assert_codec_iso_bytes::<_, Bincode>(
            &ClientState {
                chain_id: "bbn-test-5".to_string(),
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
                contract_address: hex!(
                    "bcf923a74d8b8914e0235d28c6b59e62b547af5ce366c6aafcb006bce7bb3ba4"
                )
                .into(),
            },
            &bz,
        );
    }

    #[test]
    fn proto_bytes_iso() {
        // voyager rpc client-state xion-testnet-2 "07-tendermint-1" --ibc-spec-id ibc-classic --height 4083948
        // ibc-classic wraps all states in Any
        // format below is (field number, field value)
        let bz = hex!(
            // type_url
            "0a" "2b" "2f6962632e6c69676874636c69656e74732e74656e6465726d696e742e76312e436c69656e745374617465"
            // value
            "12" "7f"

            "0a" "07" "6772616e642d31" // chain_id

            "12" "04"                  // trust_level
                "08" "02"              // numerator
                "10" "03"              // denominator

            "1a" "04"                  // trusting_period
                "08" "80ea49"          // seconds
                                       // nanos (omitted)

            "22" "04"                  // unbonding_period
                "08" "80df6e"          // seconds
                                       // nanos (omitted)

            "2a" "02"                  // max_clock_drift
                "08" "28"              // seconds
                                       // nanos (omitted)

            "32" "00"                  // frozen_height (empty struct, NOT omitted!)

            "3a" "07"                  // latest_height
                "08" "01"              // revision_number
                "10" "f4d28c0c"        // revision_height

            "42" "19"                   // proof_specs
                "0a" "09"               // leaf_spec
                    "08" "01"           // hash
                                        // prehash_key (omitted)
                    "18" "01"           // prehash_value
                    "20" "01"           // length
                    "2a" "01" "00"      // prefix

                "12" "0c"               // inner_spec
                    "0a" "02" "00" "01" // child_order
                    "10" "21"           // child_size
                    "18" "04"           // min_prefix_length
                    "20" "0c"           // max_prefix_length
                                        // empty_child (omitted)
                    "30" "01"           // hash
                                        // max_depth (omitted)
                                        // min_depth (omitted)
                                        // prehash_key_before_comparison (omitted)
            "42" "19"
                "0a" "09"               // leaf_spec
                    "08" "01"           // hash
                                        // prehash_key (omitted)
                    "18" "01"           // prehash_value
                    "20" "01"           // length
                    "2a" "01" "00"      // prefix

                "12" "0c"               // inner_spec
                    "0a" "02" "00" "01" // child_order
                    "10" "20"           // child_size
                    "18" "01"           // min_prefix_length
                    "20" "01"           // max_prefix_length
                                        // empty_child (omitted)
                    "30" "01"           // hash
                                        // max_depth (omitted)
                                        // min_depth (omitted)
                                        // prehash_key_before_comparison (omitted)

            // upgrade_path
            "4a" "07" "75706772616465"
            "4a" "10" "75706772616465644942435374617465"

            "50" "01" // allow_update_after_expiry
            "58" "01" // allow_update_after_misbehaviour
        );

        assert_codec_iso_bytes::<_, Proto>(
            &Any(ClientState {
                chain_id: "grand-1".to_string(),
                trust_level: Fraction {
                    numerator: 2,
                    denominator: NonZeroU64::new(3).unwrap(),
                },
                trusting_period: Duration::new(1209600, 0).unwrap(),
                unbonding_period: Duration::new(1814400, 0).unwrap(),
                max_clock_drift: Duration::new(40, 0).unwrap(),
                frozen_height: Some(Height::default()),
                latest_height: Height::new_with_revision(1, 25373044),
                proof_specs: SDK_SPECS.to_vec(),
                upgrade_path: ["upgrade".to_owned(), "upgradedIBCState".to_owned()].to_vec(),
                contract_address: hex!(
                    "0000000000000000000000000000000000000000000000000000000000000000"
                )
                .into(),
            }),
            &bz,
        );
    }
}
