use macros::model;

use crate::{
    errors::{required, MissingField},
    ibc::core::client::height::Height,
};

#[model(
    borsh,
    proto(
        raw(protos::union::ibc::lightclients::cometbls::v1::ClientState),
        into,
        from
    ),
    ethabi(
        raw(contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData),
        into,
        from
    )
)]
#[derive(Default)]
pub struct ClientState {
    pub chain_id: String,
    pub trusting_period: u64,
    pub unbonding_period: u64,
    pub max_clock_drift: u64,
    pub frozen_height: Height,
    pub latest_height: Height,
}

impl From<ClientState> for protos::union::ibc::lightclients::cometbls::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            trusting_period: value.trusting_period,
            unbonding_period: value.unbonding_period,
            max_clock_drift: value.max_clock_drift,
            frozen_height: Some(value.frozen_height.into()),
            latest_height: Some(value.latest_height.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error(transparent)]
    MissingField(MissingField),
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id,
            trusting_period: value.trusting_period,
            unbonding_period: value.unbonding_period,
            max_clock_drift: value.max_clock_drift,
            frozen_height: required!(value.frozen_height)?.into(),
            latest_height: required!(value.latest_height)?.into(),
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<ClientState> for contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData {
    fn from(value: ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            trusting_period: value.trusting_period,
            unbonding_period: value.unbonding_period,
            max_clock_drift: value.max_clock_drift,
            frozen_height: value.frozen_height.into(),
            latest_height: value.latest_height.into(),
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
            trusting_period: value.trusting_period,
            unbonding_period: value.unbonding_period,
            max_clock_drift: value.max_clock_drift,
            frozen_height: value.frozen_height.into(),
            latest_height: value.latest_height.into(),
        })
    }
}
