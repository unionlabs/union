use serde::{Deserialize, Serialize};

#[cfg(feature = "ethabi")]
use crate::InlineFields;
use crate::{
    errors::{required, MissingField},
    ibc::core::client::height::Height,
    Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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

#[cfg(feature = "ethabi")]
impl crate::EthAbi for ClientState {
    type EthAbi = InlineFields<contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData>;
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
impl From<ClientState>
    for InlineFields<contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData>
{
    fn from(value: ClientState) -> Self {
        Self(value.into())
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

#[cfg(feature = "ethabi")]
impl TryFrom<InlineFields<contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData>>
    for ClientState
{
    type Error = TryFromClientStateError;

    fn try_from(
        value: InlineFields<contracts::glue::UnionIbcLightclientsCometblsV1ClientStateData>,
    ) -> Result<Self, Self::Error> {
        value.0.try_into()
    }
}
