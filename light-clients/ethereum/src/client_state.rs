use crate::errors::Error;
use ethereum_consensus::{
    beacon::{Epoch, Root, Slot, Version},
    fork::ForkParameters,
    preset,
    types::{Address, H256, U64},
};
use ethereum_light_client_verifier::context::Fraction;
use ibc::{
    core::ics02_client::{
        client_state::ClientState as Ics2ClientState, client_type::ClientType, error::ClientError,
    },
    Height,
};
use ibc_proto::{
    google::protobuf::Any,
    ibc::lightclients::ethereum::v1::{ClientState as RawClientState, Fork},
    protobuf::Protobuf,
};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const ETHEREUM_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.ethereum.v1.ClientState";

pub type MainnetClientState = ClientState<{ preset::mainnet::PRESET.SYNC_COMMITTEE_SIZE }>;
pub type MinimalClientState = ClientState<{ preset::minimal::PRESET.SYNC_COMMITTEE_SIZE }>;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClientState<const SYNC_COMMITTEE_SIZE: usize> {
    /// Chain parameters
    pub genesis_validators_root: Root,
    pub min_sync_committee_participants: U64,
    pub genesis_time: U64,
    pub fork_parameters: ForkParameters,
    pub seconds_per_slot: U64,
    pub slots_per_epoch: Slot,
    pub epochs_per_sync_committee_period: Epoch,

    /// IBC Solidity parameters
    pub ibc_address: Address,
    pub ibc_commitments_slot: H256,

    /// Light Client parameters
    pub trust_level: Fraction,
    pub trusting_period: Duration,

    /// State
    pub latest_slot: Slot,
    pub latest_execution_block_number: U64,
    pub frozen_height: Option<Height>,
}

impl<const SYNC_COMMITTEE_SIZE: usize> Protobuf<RawClientState>
    for ClientState<SYNC_COMMITTEE_SIZE>
{
}

impl<const SYNC_COMMITTEE_SIZE: usize> TryFrom<RawClientState>
    for ClientState<SYNC_COMMITTEE_SIZE>
{
    type Error = Error;

    fn try_from(value: RawClientState) -> Result<Self, Self::Error> {
        fn bytes_to_version(bz: Vec<u8>) -> Version {
            assert_eq!(bz.len(), 4);
            let mut version = Version::default();
            version.0.copy_from_slice(&bz);
            version
        }

        let fork_parameters = value.fork_parameters.ok_or(Error::DecodeError)?;
        let trust_level = value.trust_level.ok_or(Error::DecodeError)?;
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
                        .ok_or(Error::DecodeError)?
                        .version,
                ),
                altair_fork_epoch: fork_parameters
                    .altair
                    .ok_or(Error::DecodeError)?
                    .epoch
                    .into(),
                bellatrix_fork_version: bytes_to_version(
                    fork_parameters
                        .bellatrix
                        .clone()
                        .ok_or(Error::DecodeError)?
                        .version,
                ),
                bellatrix_fork_epoch: fork_parameters
                    .bellatrix
                    .ok_or(Error::DecodeError)?
                    .epoch
                    .into(),
                capella_fork_version: bytes_to_version(
                    fork_parameters
                        .capella
                        .clone()
                        .ok_or(Error::DecodeError)?
                        .version,
                ),
                capella_fork_epoch: fork_parameters
                    .capella
                    .ok_or(Error::DecodeError)?
                    .epoch
                    .into(),
                eip4844_fork_version: bytes_to_version(
                    fork_parameters
                        .eip4844
                        .clone()
                        .ok_or(Error::DecodeError)?
                        .version,
                ),
                eip4844_fork_epoch: fork_parameters
                    .eip4844
                    .ok_or(Error::DecodeError)?
                    .epoch
                    .into(),
            },
            seconds_per_slot: value.seconds_per_slot.into(),
            slots_per_epoch: value.slots_per_epoch.into(),
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period.into(),
            ibc_address: value
                .ibc_address
                .as_slice()
                .try_into()
                .map_err(|_| Error::DecodeError)?,
            ibc_commitments_slot: H256::from_slice(&value.ibc_commitments_slot),
            trust_level: Fraction::new(trust_level.numerator, trust_level.denominator),
            trusting_period: Duration::from_secs(value.trusting_period),
            latest_slot: value.latest_slot.into(),
            latest_execution_block_number: value.latest_execution_block_number.into(),
            frozen_height: value
                .frozen_height
                .map(|h| Height::new(h.revision_number, h.revision_height).unwrap()),
        })
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> From<ClientState<SYNC_COMMITTEE_SIZE>> for RawClientState {
    fn from(value: ClientState<SYNC_COMMITTEE_SIZE>) -> Self {
        use ibc_proto::ibc::core::client::v1::Height as ProtoHeight;
        use ibc_proto::ibc::lightclients::ethereum::v1::{
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
            ibc_address: value.ibc_address.0.to_vec(),
            ibc_commitments_slot: value.ibc_commitments_slot.as_bytes().to_vec(),
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

impl<const SYNC_COMMITTEE_SIZE: usize> Protobuf<Any> for ClientState<SYNC_COMMITTEE_SIZE> {}

impl<const SYNC_COMMITTEE_SIZE: usize> TryFrom<Any> for ClientState<SYNC_COMMITTEE_SIZE> {
    type Error = Error;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            ETHEREUM_CLIENT_STATE_TYPE_URL => RawClientState::decode(raw.value.as_slice())
                .map_err(|_| Error::DecodeError)?
                .try_into(),
            _ => Err(Error::UnknownTypeUrl),
        }
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> From<ClientState<SYNC_COMMITTEE_SIZE>> for Any {
    fn from(value: ClientState<SYNC_COMMITTEE_SIZE>) -> Self {
        Self {
            type_url: ETHEREUM_CLIENT_STATE_TYPE_URL.to_string(),
            value: Protobuf::<RawClientState>::encode_vec(&value),
        }
    }
}

pub fn downcast_eth_client_state<const SYNC_COMMITTEE_SIZE: usize>(
    cs: &dyn Ics2ClientState,
) -> Result<&ClientState<SYNC_COMMITTEE_SIZE>, ClientError> {
    cs.as_any()
        .downcast_ref::<ClientState<SYNC_COMMITTEE_SIZE>>()
        .ok_or_else(|| ClientError::ClientArgsTypeMismatch {
            client_type: ClientType::new("08-wasm".into()),
        })
}
