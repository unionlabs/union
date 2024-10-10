use macros::model;

use crate::{
    errors::{required, ExpectedLength, InvalidLength, MissingField},
    ibc::core::client::height::Height,
};

#[model(
    proto(
        raw(protos::union::ibc::lightclients::cometbls::v1::ClientState),
        into,
        from
    ),
    ethabi(raw(ibc_solidity::cometbls::ClientState), into, from)
)]
pub struct ClientState {
    pub chain_id: CometblsChainId,
    pub trusting_period: u64,
    pub max_clock_drift: u64,
    pub frozen_height: Height,
    pub latest_height: Height,
}

// TODO: Generalize this to a reusable type? Also see https://github.com/cometbft/cometbft/blob/54098b0c19099a38d2ce43aa30e6aee4c3f90978/types/genesis.go#L21
#[model]
#[serde(transparent)]
pub struct CometblsChainId(String);

impl CometblsChainId {
    pub const MAX_LEN: usize = 31;

    pub fn from_string(s: impl Into<String>) -> Result<Self, InvalidLength> {
        let s = s.into();

        if s.len() > Self::MAX_LEN {
            Err(InvalidLength {
                expected: ExpectedLength::Between(0, Self::MAX_LEN),
                found: s.len(),
            })
        } else {
            Ok(Self(s))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[cfg(feature = "ethabi")]
    pub fn try_from_fixed_bytes(
        bz: alloy_core::primitives::FixedBytes<31>,
    ) -> Result<Self, alloc::string::FromUtf8Error> {
        String::from_utf8(bz.into_iter().skip_while(|b| *b == 0).collect()).map(Self)
    }

    #[cfg(feature = "ethabi")]
    #[must_use]
    pub fn into_fixed_bytes(self) -> alloy_core::primitives::FixedBytes<31> {
        let mut bz = <alloy_core::primitives::FixedBytes<31>>::default();

        bz[Self::MAX_LEN - self.0.len()..].copy_from_slice(self.0.as_bytes());

        bz
    }
}

impl From<ClientState> for protos::union::ibc::lightclients::cometbls::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            chain_id: value.chain_id.0,
            trusting_period: value.trusting_period,
            max_clock_drift: value.max_clock_drift,
            frozen_height: Some(value.frozen_height.into()),
            latest_height: Some(value.latest_height.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid chain id")]
    ChainId(#[from] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::cometbls::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::cometbls::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: CometblsChainId::from_string(value.chain_id)?,
            trusting_period: value.trusting_period,
            max_clock_drift: value.max_clock_drift,
            frozen_height: required!(value.frozen_height)?.into(),
            latest_height: required!(value.latest_height)?.into(),
        })
    }
}

#[cfg(feature = "ethabi")]
impl From<ClientState> for ibc_solidity::cometbls::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            chainId: value.chain_id.into_fixed_bytes(),
            trustingPeriod: value.trusting_period,
            maxClockDrift: value.max_clock_drift,
            frozenHeight: value.frozen_height.revision(),
            latestHeight: value.latest_height.revision(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromEthAbiClientStateError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid chain id")]
    ChainId(#[from] alloc::string::FromUtf8Error),
}

#[cfg(feature = "ethabi")]
impl TryFrom<ibc_solidity::cometbls::ClientState> for ClientState {
    type Error = TryFromEthAbiClientStateError;

    fn try_from(value: ibc_solidity::cometbls::ClientState) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: CometblsChainId::try_from_fixed_bytes(value.chainId)?,
            trusting_period: value.trustingPeriod,
            max_clock_drift: value.maxClockDrift,
            frozen_height: Height::new(value.frozenHeight),
            latest_height: Height::new(value.latestHeight),
        })
    }
}
