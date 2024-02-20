use serde::{Deserialize, Serialize};

use crate::{
    cosmos::ics23::proof_spec::ProofSpec,
    errors::{required, MissingField},
    google::protobuf::duration::Duration,
    ibc::{
        core::client::height::Height,
        lightclients::tendermint::fraction::{Fraction, TryFromFractionError},
    },
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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
    pub allow_update_after_expiry: bool,
    pub allow_update_after_misbehavior: bool,
}

impl From<ClientState> for protos::ibc::lightclients::tendermint::v1::ClientState {
    #[allow(deprecated)]
    fn from(value: ClientState) -> Self {
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
            allow_update_after_expiry: value.allow_update_after_expiry,
            allow_update_after_misbehaviour: value.allow_update_after_misbehavior,
        }
    }
}

impl TypeUrl for protos::ibc::lightclients::tendermint::v1::ClientState {
    const TYPE_URL: &'static str = "/ibc.lightclients.tendermint.v1.ClientState";
}

impl Proto for ClientState {
    type Proto = protos::ibc::lightclients::tendermint::v1::ClientState;
}

#[derive(Debug)]
pub enum TryFromClientStateError {
    MissingField(MissingField),
    TrustLevel(TryFromFractionError),
    TrustingPeriod(TryFromProtoErrorOf<Duration>),
    UnbondingPeriod(TryFromProtoErrorOf<Duration>),
    MaxClockDrift(TryFromProtoErrorOf<Duration>),
    ProofSpecs(TryFromProtoErrorOf<ProofSpec>),
}

impl TryFrom<protos::ibc::lightclients::tendermint::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    #[allow(deprecated)]
    fn try_from(
        value: protos::ibc::lightclients::tendermint::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id,
            trust_level: required!(value.trust_level)?
                .try_into()
                .map_err(TryFromClientStateError::TrustLevel)?,
            trusting_period: required!(value.trusting_period)?
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            unbonding_period: required!(value.unbonding_period)?
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            max_clock_drift: required!(value.max_clock_drift)?
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            frozen_height: value.frozen_height.map(Into::into),
            latest_height: required!(value.latest_height)?.into(),
            proof_specs: value
                .proof_specs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromClientStateError::ProofSpecs)?,
            upgrade_path: value.upgrade_path,
            allow_update_after_expiry: value.allow_update_after_expiry,
            allow_update_after_misbehavior: value.allow_update_after_misbehaviour,
        })
    }
}
