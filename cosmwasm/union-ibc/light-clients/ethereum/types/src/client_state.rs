use beacon_api_types::{ForkParameters, PresetBaseKind};
use unionlabs::{
    hash::{H160, H256},
    ibc::core::client::height::Height,
    uint::U256,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub chain_id: U256,
    pub chain_spec: PresetBaseKind,
    pub genesis_validators_root: H256,
    pub genesis_time: u64,
    pub fork_parameters: ForkParameters,
    pub latest_slot: u64,
    // even though it would be better to have option, ethabicodec don't handle it as zero struct...
    pub frozen_height: Height,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}

#[cfg(feature = "proto")]
pub mod proto {
    use std::{str::FromStr, sync::Arc};

    use beacon_api_types::PresetBaseKind;
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        impl_proto_via_try_from_into, required,
        uint::{FromDecStrErr, U256},
    };

    use crate::{fork_parameters_proto, ClientState};

    impl_proto_via_try_from_into!(ClientState => protos::union::ibc::lightclients::ethereum::v1::ClientState);

    impl From<ClientState> for protos::union::ibc::lightclients::ethereum::v1::ClientState {
        fn from(value: ClientState) -> Self {
            Self {
                chain_id: value.chain_id.to_string(),
                chain_spec: value.chain_spec.to_string(),
                genesis_validators_root: value.genesis_validators_root.into(),
                genesis_time: value.genesis_time,
                fork_parameters: Some(fork_parameters_proto::into_proto(value.fork_parameters)),
                latest_slot: value.latest_slot,
                frozen_height: Some(value.frozen_height.into()),
                ibc_contract_address: value.ibc_contract_address.into(),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromClientStateError {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid chain spec: {0}")]
        ChainSpec(String),
        #[error("invalid chain id: {0:?}")]
        ChainId(Arc<FromDecStrErr>),
        #[error("invalid fork parameters")]
        ForkParameters(#[from] fork_parameters_proto::Error),
        #[error("invalid genesis validators root")]
        GenesisValidatorsRoot(#[source] InvalidLength),
        #[error("invalid ibc contract address")]
        IbcContractAddress(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ClientState> for ClientState {
        type Error = TryFromClientStateError;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                chain_id: U256::from_str(&value.chain_id)
                    .map_err(|err| TryFromClientStateError::ChainId(Arc::new(err)))?,
                chain_spec: PresetBaseKind::from_str(&value.chain_spec)
                    .map_err(TryFromClientStateError::ChainSpec)?,
                genesis_validators_root: value
                    .genesis_validators_root
                    .try_into()
                    .map_err(TryFromClientStateError::GenesisValidatorsRoot)?,
                genesis_time: value.genesis_time,
                fork_parameters: required!(value.fork_parameters)
                    .map(fork_parameters_proto::try_from_proto)??,
                latest_slot: value.latest_slot,
                frozen_height: value.frozen_height.unwrap_or_default().into(),
                ibc_contract_address: value
                    .ibc_contract_address
                    .try_into()
                    .map_err(TryFromClientStateError::IbcContractAddress)?,
            })
        }
    }
}
