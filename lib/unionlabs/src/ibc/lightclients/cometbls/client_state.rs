use serde::{Deserialize, Serialize};

use crate::{
    errors::MissingField,
    ibc::{
        core::client::height::Height, google::protobuf::duration::Duration,
        lightclients::tendermint::fraction::Fraction,
    },
    Proto, TypeUrl,
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

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ClientState> for ClientState {
    type Error = MissingField;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id,
            trust_level: value.trust_level.ok_or(MissingField("trust_level"))?.into(),
            trusting_period: value
                .trusting_period
                .ok_or(MissingField("trusting_period"))?
                .into(),
            unbonding_period: value
                .unbonding_period
                .ok_or(MissingField("unbonding_period"))?
                .into(),
            max_clock_drift: value
                .max_clock_drift
                .ok_or(MissingField("max_clock_drift"))?
                .into(),
            frozen_height: value
                .frozen_height
                .ok_or(MissingField("frozen_height"))?
                .into(),
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
impl From<contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData> for ClientState {
    fn from(value: contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData) -> Self {
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
