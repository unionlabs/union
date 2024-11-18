use tendermint_light_client_types::Fraction;
use unionlabs::{
    cosmos::ics23::proof_spec::ProofSpec, google::protobuf::duration::Duration, hash::H160,
    ibc::core::client::height::Height, uint::U256,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub consensus_chain_id: String,
    pub execution_chain_id: U256,

    // TENDERMINT
    pub trust_level: Fraction,
    pub trusting_period: Duration,
    pub max_clock_drift: Duration,
    pub frozen_height: Option<Height>,
    pub latest_height: Height,
    pub proof_specs: Vec<ProofSpec>,
    pub upgrade_path: Vec<String>,

    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}

#[cfg(feature = "proto")]
pub mod proto {
    use std::{str::FromStr, sync::Arc};

    use tendermint_light_client_types::fraction;
    use unionlabs::{
        cosmos::ics23::proof_spec::TryFromProofSpecError,
        errors::{InvalidLength, MissingField},
        google::protobuf::duration::DurationError,
        impl_proto_via_try_from_into, required,
        uint::{FromDecStrErr, U256},
    };

    use crate::ClientState;

    impl_proto_via_try_from_into!(ClientState => protos::union::ibc::lightclients::berachain::v1::ClientState);

    impl TryFrom<protos::union::ibc::lightclients::berachain::v1::ClientState> for ClientState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::berachain::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                consensus_chain_id: value.consensus_chain_id,
                execution_chain_id: U256::from_str(&value.execution_chain_id)
                    .map_err(|e| Error::ExecutionChainId(Arc::new(e)))?,
                trust_level: required!(value.trust_level)?
                    .try_into()
                    .map_err(Error::TrustLevel)?,
                trusting_period: required!(value.trusting_period)?
                    .try_into()
                    .map_err(Error::TrustingPeriod)?,
                max_clock_drift: required!(value.max_clock_drift)?
                    .try_into()
                    .map_err(Error::MaxClockDrift)?,
                frozen_height: value.frozen_height.map(Into::into),
                latest_height: required!(value.latest_height)?.into(),
                proof_specs: value
                    .proof_specs
                    .into_iter()
                    .map(|ps| ps.try_into().map_err(Error::ProofSpecs))
                    .collect::<Result<Vec<_>, _>>()?,
                upgrade_path: value.upgrade_path,
                ibc_contract_address: value
                    .ibc_contract_address
                    .try_into()
                    .map_err(Error::IbcContractAddress)?,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid execution chain id")]
        // arc bc not clone
        ExecutionChainId(#[source] Arc<FromDecStrErr>),
        #[error("invalid trust level")]
        TrustLevel(#[from] fraction::proto::Error),
        #[error("invalid trusting period")]
        TrustingPeriod(#[source] DurationError),
        #[error("invalid max clock drift")]
        MaxClockDrift(#[source] DurationError),
        #[error("invalid proof specs")]
        ProofSpecs(#[from] TryFromProofSpecError),
        #[error("invalid ibc contract address")]
        IbcContractAddress(#[source] InvalidLength),
    }

    impl From<ClientState> for protos::union::ibc::lightclients::berachain::v1::ClientState {
        fn from(value: ClientState) -> Self {
            Self {
                consensus_chain_id: value.consensus_chain_id,
                execution_chain_id: value.execution_chain_id.to_string(),
                trust_level: Some(value.trust_level.into()),
                trusting_period: Some(value.trusting_period.into()),
                max_clock_drift: Some(value.max_clock_drift.into()),
                frozen_height: value.frozen_height.map(Into::into),
                latest_height: Some(value.latest_height.into()),
                proof_specs: value.proof_specs.into_iter().map(Into::into).collect(),
                upgrade_path: value.upgrade_path,
                ibc_contract_address: value.ibc_contract_address.into(),
            }
        }
    }
}
