use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    ibc::{
        core::client::height::Height, google::protobuf::duration::Duration,
        lightclients::tendermint::fraction::Fraction,
    },
    Proto, TryFromProtoErrorOf, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientState {
    pub chain_id: String,
    pub trust_level: Fraction,
    pub trusting_period: Duration,
    pub unbonding_period: Duration,
    pub max_clock_drift: Duration,
    pub frozen_height: Height,
}

impl From<ClientState> for protos::union::ibc::lightclients::cometbls::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            trust_level: Some(value.trust_level.into()),
            trusting_period: Some(value.trusting_period.into()),
            unbonding_period: Some(value.unbonding_period.into()),
            max_clock_drift: Some(value.max_clock_drift.into()),
            frozen_height: Some(value.frozen_height.into()),
        }
    }
}

impl TypeUrl for protos::union::ibc::lightclients::cometbls::v1::ClientState {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.cometbls.v1.ClientState";
}

impl Proto for ClientState {
    type Proto = protos::union::ibc::lightclients::cometbls::v1::ClientState;
}

#[derive(Debug)]
pub enum TryFromClientStateError {
    MissingField(MissingField),
    TrustingPeriod(TryFromProtoErrorOf<Duration>),
    UnbondingPeriod(TryFromProtoErrorOf<Duration>),
    MaxClockDrift(TryFromProtoErrorOf<Duration>),
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id,
            trust_level: required!(value.trust_level)?.into(),
            trusting_period: required!(value.trusting_period)?
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            unbonding_period: required!(value.unbonding_period)?
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            max_clock_drift: required!(value.max_clock_drift)?
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            frozen_height: required!(value.frozen_height)?.into(),
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<ClientState> for contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData {
    fn from(value: ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            trust_level: value.trust_level.into(),
            trusting_period: value.trusting_period.into(),
            unbonding_period: value.unbonding_period.into(),
            max_clock_drift: value.max_clock_drift.into(),
            frozen_height: value.frozen_height.into(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id,
            trust_level: value.trust_level.into(),
            trusting_period: value
                .trusting_period
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            unbonding_period: value
                .unbonding_period
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            max_clock_drift: value
                .max_clock_drift
                .try_into()
                .map_err(TryFromClientStateError::TrustingPeriod)?,
            frozen_height: value.frozen_height.into(),
        })
    }
}
