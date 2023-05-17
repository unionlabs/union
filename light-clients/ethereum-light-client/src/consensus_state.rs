use crate::{errors::Error, eth_types::SYNC_COMMITTEE_SIZE};
use ethereum_consensus::{beacon::Slot, bls::PublicKey, sync_protocol::SyncCommittee};
use ethereum_light_client_verifier::state::SyncCommitteeView;
use ibc::core::ics23_commitment::commitment::CommitmentRoot;
use prost::Message;
use protos::{
    google::protobuf::Any as IBCAny,
    ibc::lightclients::tendermint::v1::ConsensusState as RawTmConsensusState,
    union::ibc::lightclients::{
        cometbls::v1::ConsensusState as RawCometConsensusState,
        ethereum::v1::ConsensusState as RawConsensusState,
    },
};

pub const ETHEREUM_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.ethereum.v1.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConsensusState {
    /// finalized header's slot
    pub slot: Slot,
    /// the storage root of the IBC contract
    pub storage_root: CommitmentRoot,
    /// timestamp from execution payload
    pub timestamp: u64,
    /// aggregate public key of current sync committee
    pub current_sync_committee: PublicKey,
    /// aggregate public key of next sync committee
    pub next_sync_committee: Option<PublicKey>,
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self {
            slot: Default::default(),
            storage_root: CommitmentRoot::from_bytes(Default::default()),
            timestamp: Default::default(),
            current_sync_committee: Default::default(),
            next_sync_committee: Default::default(),
        }
    }
}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = Error;

    fn try_from(value: RawConsensusState) -> Result<Self, Self::Error> {
        let next_sync_committee = if value.next_sync_committee.is_empty() {
            None
        } else {
            Some(
                PublicKey::try_from(value.next_sync_committee)
                    .map_err(|_| Error::InvalidPublicKey)?,
            )
        };
        Ok(Self {
            slot: value.slot.into(),
            storage_root: value.storage_root.into(),
            timestamp: value.timestamp,
            current_sync_committee: PublicKey::try_from(value.current_sync_committee)
                .map_err(|_| Error::InvalidPublicKey)?,
            next_sync_committee,
        })
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        let next_sync_committee = match value.next_sync_committee {
            Some(next_sync_committee) => next_sync_committee.0.to_vec(),
            None => Vec::new(),
        };
        Self {
            slot: value.slot.into(),
            storage_root: value.storage_root.into_vec(),
            timestamp: value.timestamp,
            current_sync_committee: value.current_sync_committee.to_vec(),
            next_sync_committee,
        }
    }
}

impl TryFrom<IBCAny> for ConsensusState {
    type Error = Error;

    fn try_from(raw: IBCAny) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            ETHEREUM_CONSENSUS_STATE_TYPE_URL => RawConsensusState::decode(raw.value.as_slice())
                .map_err(|_| Error::decode("when decoding proto consensus state (Any)"))?
                .try_into()
                .map_err(|_| Error::decode("when converting to consensus state (Any)")),
            _ => Err(Error::UnknownTypeUrl),
        }
    }
}

pub fn tendermint_to_cometbls_consensus_state(
    state: RawTmConsensusState,
) -> RawCometConsensusState {
    RawCometConsensusState {
        timestamp: state.timestamp,
        root: state.root,
        next_validators_hash: state.next_validators_hash,
    }
}

// impl From<ConsensusState> for IBCAny {
//     fn from(consensus_state: ConsensusState) -> Self {
//         Self {
//             type_url: ETHEREUM_CONSENSUS_STATE_TYPE_URL.to_string(),
//             value: Protobuf::<RawConsensusState>::encode_vec(&consensus_state),
//         }
//     }
// }

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustedConsensusState {
    state: ConsensusState,
    current_sync_committee: Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>,
    next_sync_committee: Option<SyncCommittee<SYNC_COMMITTEE_SIZE>>,
}

impl TrustedConsensusState {
    pub fn new(
        consensus_state: ConsensusState,
        sync_committee: SyncCommittee<SYNC_COMMITTEE_SIZE>,
        is_next: bool,
    ) -> Result<Self, Error> {
        sync_committee
            .validate()
            .map_err(|_| Error::InvalidSyncCommittee)?;
        if !is_next {
            return if sync_committee.aggregate_pubkey == consensus_state.current_sync_committee {
                Ok(Self {
                    state: consensus_state,
                    current_sync_committee: Some(sync_committee),
                    next_sync_committee: None,
                })
            } else {
                Err(Error::InvalidSyncCommittee)
            };
        }

        if let Some(next_sync_committee) = consensus_state.next_sync_committee.clone() {
            if sync_committee.aggregate_pubkey == next_sync_committee {
                Ok(Self {
                    state: consensus_state,
                    current_sync_committee: None,
                    next_sync_committee: Some(sync_committee),
                })
            } else {
                Err(Error::InvalidSyncCommittee)
            }
        } else {
            Err(Error::NoNextSyncCommittee)
        }
    }

    pub fn current_sync_committee_aggregate_key(&self) -> PublicKey {
        self.state.current_sync_committee.clone()
    }
}

impl SyncCommitteeView<SYNC_COMMITTEE_SIZE> for TrustedConsensusState {
    fn current_slot(&self) -> Slot {
        self.state.slot
    }

    // TODO(aeryz): `current_sync_committee` can be empty, this means that this function
    // should return Result/Option.
    fn current_sync_committee(
        &self,
    ) -> &ethereum_consensus::sync_protocol::SyncCommittee<SYNC_COMMITTEE_SIZE> {
        self.current_sync_committee.as_ref().unwrap()
    }

    fn next_sync_committee(
        &self,
    ) -> Option<&ethereum_consensus::sync_protocol::SyncCommittee<SYNC_COMMITTEE_SIZE>> {
        self.next_sync_committee.as_ref()
    }
}
