use crate::{consensus_state::TrustedConsensusState, errors::Error};
use ethereum_consensus::{
    beacon::{Epoch, Root, Slot, Version},
    context::ChainContext,
    fork::ForkParameters,
    types::{H256, U64},
};
use ethereum_light_client_verifier::{
    consensus::CurrentNextSyncProtocolVerifier,
    context::ConsensusVerificationContext,
    context::{Fraction, LightClientContext},
    execution::ExecutionVerifier,
};

use ibc::{
    core::ics02_client::{
        client_state::ClientState as Ics2ClientState, client_type::ClientType, error::ClientError,
    },
    timestamp::Timestamp,
    Height,
};
use prost::Message;
use protos::{
    google::protobuf::Any,
    ibc::lightclients::{
        tendermint::v1::ClientState as RawTmClientState,
        wasm::v1::ClientState as RawWasmClientState,
    },
    union::ibc::lightclients::{
        cometbls::v1::{ClientState as RawCometClientState, Fraction as RawCometFraction},
        ethereum::v1::{ClientState as RawClientState, Fork},
    },
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientState {
    /// Chain parameters
    pub genesis_validators_root: Root,
    pub min_sync_committee_participants: U64,
    pub genesis_time: U64,
    pub fork_parameters: ForkParameters,
    pub seconds_per_slot: U64,
    pub slots_per_epoch: Slot,
    pub epochs_per_sync_committee_period: Epoch,

    /// Light Client parameters
    pub trust_level: Fraction,
    pub trusting_period: Duration,

    /// State
    pub latest_slot: Slot,
    pub latest_execution_block_number: U64,
    pub frozen_height: Option<Height>,
    pub counterparty_connection_state_slot: Slot,

    /// Verifier
    #[serde(skip)]
    pub consensus_verifier: CurrentNextSyncProtocolVerifier<TrustedConsensusState>,
    #[serde(skip)]
    pub execution_verifier: ExecutionVerifier,
}

impl ClientState {
    pub const TYPE_URL: &str = "/ibc.lightclients.ethereum.v1.ClientState";

    pub fn with_frozen_height(self, h: Height) -> Self {
        Self {
            frozen_height: Some(h),
            ..self
        }
    }

    pub fn build_context(
        &self,
        current_timestamp: Timestamp,
    ) -> Result<impl ChainContext + ConsensusVerificationContext, Error> {
        let current_timestamp = U64::from(
            current_timestamp
                .into_tm_time()
                .ok_or(Error::TimestampNotSet)?
                .unix_timestamp() as u64,
        );
        let current_slot = (current_timestamp - self.genesis_time) / self.seconds_per_slot
            + self.fork_parameters.genesis_slot;
        Ok(LightClientContext::new(
            self.fork_parameters.clone(),
            self.seconds_per_slot,
            self.slots_per_epoch,
            self.epochs_per_sync_committee_period,
            self.genesis_time,
            self.genesis_validators_root.clone(),
            self.min_sync_committee_participants.0 as usize,
            self.trust_level.clone(),
            move || current_slot,
        ))
    }
}

// impl Protobuf<RawClientState> for ClientState {}

impl TryFrom<RawClientState> for ClientState {
    type Error = Error;

    fn try_from(value: RawClientState) -> Result<Self, Self::Error> {
        fn bytes_to_version(bz: Vec<u8>) -> Version {
            assert_eq!(bz.len(), 4);
            let mut version = Version::default();
            version.0.copy_from_slice(&bz);
            version
        }

        let fork_parameters = value
            .fork_parameters
            .ok_or(Error::decode("no `fork_parameters` in `RawClientState`"))?;
        let trust_level = value
            .trust_level
            .ok_or(Error::decode("no `trust_level` in `RawClientState`"))?;
        Ok(Self {
            genesis_validators_root: H256::from_slice(&value.genesis_validators_root),
            min_sync_committee_participants: value.min_sync_committee_participants.into(),
            genesis_time: value.genesis_time.into(),
            fork_parameters: ForkParameters {
                genesis_fork_version: bytes_to_version(fork_parameters.genesis_fork_version),
                genesis_slot: fork_parameters.genesis_slot.into(),
                altair_fork_version: bytes_to_version(
                    fork_parameters
                        .altair
                        .clone()
                        .ok_or(Error::decode("no `altair` in `RawClientState`"))?
                        .version,
                ),
                altair_fork_epoch: fork_parameters
                    .altair
                    .ok_or(Error::decode("no `altair` in `RawClientState`"))?
                    .epoch
                    .into(),
                bellatrix_fork_version: bytes_to_version(
                    fork_parameters
                        .bellatrix
                        .clone()
                        .ok_or(Error::decode("no `bellatrix` in `RawClientState`"))?
                        .version,
                ),
                bellatrix_fork_epoch: fork_parameters
                    .bellatrix
                    .ok_or(Error::decode("no `bellatrix` in `RawClientState`"))?
                    .epoch
                    .into(),
                capella_fork_version: bytes_to_version(
                    fork_parameters
                        .capella
                        .clone()
                        .ok_or(Error::decode("no `capella` in `RawClientState`"))?
                        .version,
                ),
                capella_fork_epoch: fork_parameters
                    .capella
                    .ok_or(Error::decode("no `bellatrix` in `RawClientState`"))?
                    .epoch
                    .into(),
                eip4844_fork_version: bytes_to_version(
                    fork_parameters
                        .eip4844
                        .clone()
                        .ok_or(Error::decode("no `eip4844` in `RawClientState`"))?
                        .version,
                ),
                eip4844_fork_epoch: fork_parameters
                    .eip4844
                    .ok_or(Error::decode("no `eip4844` in `RawClientState`"))?
                    .epoch
                    .into(),
            },
            seconds_per_slot: value.seconds_per_slot.into(),
            slots_per_epoch: value.slots_per_epoch.into(),
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period.into(),
            trust_level: Fraction::new(trust_level.numerator, trust_level.denominator),
            trusting_period: Duration::from_secs(value.trusting_period),
            latest_slot: value.latest_slot.into(),
            latest_execution_block_number: value.latest_execution_block_number.into(),
            frozen_height: if let Some(h) = value.frozen_height {
                Some(
                    Height::new(h.revision_number, h.revision_height).map_err(|_| {
                        Error::decode("Invalid `frozen_height` in `RawClientState`")
                    })?,
                )
            } else {
                None
            },
            consensus_verifier: Default::default(),
            execution_verifier: Default::default(),
            counterparty_connection_state_slot: 3.into(),
        })
    }
}

impl From<ClientState> for RawClientState {
    fn from(value: ClientState) -> Self {
        use protos::ibc::core::client::v1::Height as ProtoHeight;
        use protos::union::ibc::lightclients::ethereum::v1::{
            ForkParameters as ProtoForkParameters, Fraction as ProtoFraction,
        };

        fn make_fork(version: Version, epoch: U64) -> Fork {
            Fork {
                version: version_to_bytes(version),
                epoch: epoch.into(),
            }
        }

        fn version_to_bytes(version: Version) -> Vec<u8> {
            version.0.to_vec()
        }

        let fork_parameters = value.fork_parameters;

        Self {
            genesis_validators_root: value.genesis_validators_root.as_bytes().to_vec(),
            min_sync_committee_participants: value.min_sync_committee_participants.into(),
            genesis_time: value.genesis_time.into(),
            fork_parameters: Some(ProtoForkParameters {
                genesis_fork_version: version_to_bytes(fork_parameters.genesis_fork_version),
                genesis_slot: fork_parameters.genesis_slot.into(),
                altair: Some(make_fork(
                    fork_parameters.altair_fork_version,
                    fork_parameters.altair_fork_epoch,
                )),
                bellatrix: Some(make_fork(
                    fork_parameters.bellatrix_fork_version,
                    fork_parameters.bellatrix_fork_epoch,
                )),
                capella: Some(make_fork(
                    fork_parameters.capella_fork_version,
                    fork_parameters.capella_fork_epoch,
                )),
                eip4844: Some(make_fork(
                    fork_parameters.eip4844_fork_version,
                    fork_parameters.eip4844_fork_epoch,
                )),
            }),
            seconds_per_slot: value.seconds_per_slot.into(),
            slots_per_epoch: value.slots_per_epoch.into(),
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period.into(),
            trust_level: Some(ProtoFraction {
                numerator: value.trust_level.numerator,
                denominator: value.trust_level.denominator,
            }),
            trusting_period: value.trusting_period.as_secs(),
            latest_slot: value.latest_slot.into(),
            latest_execution_block_number: value.latest_execution_block_number.into(),
            frozen_height: value.frozen_height.map(|h| ProtoHeight {
                revision_number: h.revision_number(),
                revision_height: h.revision_height(),
            }),
        }
    }
}

pub fn tendermint_to_cometbls_client_state(state: RawTmClientState) -> RawCometClientState {
    RawCometClientState {
        chain_id: state.chain_id,
        trust_level: state.trust_level.map(|tl| RawCometFraction {
            numerator: tl.numerator,
            denominator: tl.denominator,
        }),
        trusting_period: state.trusting_period,
        unbonding_period: state.unbonding_period,
        max_clock_drift: state.max_clock_drift,
        frozen_height: state.frozen_height,
        latest_height: state.latest_height,
    }
}

pub fn decode_any_to_tendermint_client_state(state: &[u8]) -> Result<RawTmClientState, Error> {
    let any_state = Any::decode(state)
        .map_err(|_| Error::decode("when decoding raw bytes to any in `verify_membership`"))?;

    let wasm_client_state =
        RawWasmClientState::decode(any_state.value.as_slice()).map_err(|_| {
            Error::decode("when decoding any value to wasm client state in `verify_membership`")
        })?;

    let any_state = Any::decode(wasm_client_state.data.as_slice()).map_err(|_| {
        Error::decode("when decoding wasm client state to tm client state in `verify_membership`")
    })?;

    RawTmClientState::decode(any_state.value.as_slice()).map_err(|_| {
        Error::decode("when decoding any state to tm client state in `verify_membership`")
    })
}

// impl Protobuf<Any> for ClientState {}

impl TryFrom<Any> for ClientState {
    type Error = Error;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            Self::TYPE_URL => RawClientState::decode(raw.value.as_slice())
                .map_err(|_| Error::decode("during parsing `RawClientState` from `Any`"))?
                .try_into(),
            _ => Err(Error::UnknownTypeUrl),
        }
    }
}

// impl From<ClientState> for Any {
//     fn from(value: ClientState) -> Self {
//         Self {
//             type_url: ETHEREUM_CLIENT_STATE_TYPE_URL.to_string(),
//             value: Protobuf::<RawClientState>::encode_vec(&value),
//         }
//     }
// }

pub fn downcast_eth_client_state(cs: &dyn Ics2ClientState) -> Result<&ClientState, ClientError> {
    cs.as_any()
        .downcast_ref::<ClientState>()
        .ok_or_else(|| ClientError::ClientArgsTypeMismatch {
            client_type: ClientType::new("08-wasm".into()),
        })
}
