use unionlabs::{
    cosmos::ics23::proof_spec::ProofSpec, google::protobuf::duration::Duration, hash::H256,
    ibc::core::client::height::Height,
};

use crate::Fraction;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        google::protobuf::duration::DurationError, hash::H256, impl_proto_via_try_from_into,
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
                frozen_height: value.frozen_height.map(Into::into),
                latest_height: Some(value.latest_height.into()),
                proof_specs: value.proof_specs.into_iter().map(Into::into).collect(),
                upgrade_path: value.upgrade_path,
                allow_update_after_expiry: Default::default(),
                allow_update_after_misbehaviour: Default::default(),
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
                frozen_height: value.frozen_height.map(Into::into),
                latest_height: required!(value.latest_height)?.into(),
                proof_specs: value
                    .proof_specs
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::ProofSpecs)?,
                upgrade_path: value.upgrade_path,
                // FIXME: we need to define the tm proto ourself and regenerate
                contract_address: H256::default(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use unionlabs::{
        cosmos::ics23::{
            hash_op::HashOp, inner_spec::InnerSpec, leaf_op::LeafOp, length_op::LengthOp,
        },
        encoding::{Bincode, Json, Proto},
        primitives::Bytes,
        test_utils::assert_codec_iso,
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
}
