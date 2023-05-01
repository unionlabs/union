use crate::{
    errors::Error,
    types::{
        convert_consensus_update_to_proto, convert_proto_to_consensus_update, TrustedSyncCommittee,
    },
    update::ConsensusUpdateInfo,
};
use ethereum_light_client_verifier::misbehaviour::{
    FinalizedHeaderMisbehaviour, Misbehaviour as MisbehaviourData, NextSyncCommitteeMisbehaviour,
};
use ibc::core::ics24_host::identifier::ClientId;
use ibc_proto::google::protobuf::Any as IBCAny;
use ibc_proto::ibc::lightclients::ethereum::v1::{
    FinalizedHeaderMisbehaviour as RawFinalizedHeaderMisbehaviour,
    NextSyncCommitteeMisbehaviour as RawNextSyncCommitteeMisbehaviour,
};
use ibc_proto::protobuf::Protobuf;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const ETHEREUM_FINALIZED_HEADER_MISBEHAVIOUR_TYPE_URL: &str =
    "/ibc.lightclients.ethereum.v1.FinalizedHeaderMisbehaviour";
pub const ETHEREUM_NEXT_SYNC_COMMITTEE_MISBEHAVIOUR_TYPE_URL: &str =
    "/ibc.lightclients.ethereum.v1.NextSyncCommitteeMisbehaviour";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Misbehaviour<const SYNC_COMMITTEE_SIZE: usize> {
    pub client_id: ClientId,
    pub trusted_sync_committee: TrustedSyncCommittee<SYNC_COMMITTEE_SIZE>,
    pub data: MisbehaviourData<SYNC_COMMITTEE_SIZE, ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE>>,
}

impl<const SYNC_COMMITTEE_SIZE: usize> Protobuf<RawFinalizedHeaderMisbehaviour>
    for Misbehaviour<SYNC_COMMITTEE_SIZE>
{
}

impl<const SYNC_COMMITTEE_SIZE: usize> Protobuf<RawNextSyncCommitteeMisbehaviour>
    for Misbehaviour<SYNC_COMMITTEE_SIZE>
{
}

impl<const SYNC_COMMITTEE_SIZE: usize> TryFrom<RawFinalizedHeaderMisbehaviour>
    for Misbehaviour<SYNC_COMMITTEE_SIZE>
{
    type Error = Error;
    fn try_from(value: RawFinalizedHeaderMisbehaviour) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: ClientId::from_str(&value.client_id).map_err(|_| Error::InvalidClientId)?,
            trusted_sync_committee: value
                .trusted_sync_committee
                .ok_or(Error::DecodeError)?
                .try_into()?,
            data: MisbehaviourData::FinalizedHeader(FinalizedHeaderMisbehaviour {
                consensus_update_1: convert_proto_to_consensus_update(
                    value.consensus_update_1.ok_or(Error::DecodeError)?,
                )?,
                consensus_update_2: convert_proto_to_consensus_update(
                    value.consensus_update_2.ok_or(Error::DecodeError)?,
                )?,
            }),
        })
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> TryFrom<RawNextSyncCommitteeMisbehaviour>
    for Misbehaviour<SYNC_COMMITTEE_SIZE>
{
    type Error = Error;
    fn try_from(value: RawNextSyncCommitteeMisbehaviour) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: ClientId::from_str(&value.client_id).map_err(|_| Error::InvalidClientId)?,
            trusted_sync_committee: value
                .trusted_sync_committee
                .ok_or(Error::DecodeError)?
                .try_into()?,
            data: MisbehaviourData::NextSyncCommittee(NextSyncCommitteeMisbehaviour {
                consensus_update_1: convert_proto_to_consensus_update(
                    value.consensus_update_1.ok_or(Error::DecodeError)?,
                )?,
                consensus_update_2: convert_proto_to_consensus_update(
                    value.consensus_update_2.ok_or(Error::DecodeError)?,
                )?,
            }),
        })
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> From<Misbehaviour<SYNC_COMMITTEE_SIZE>>
    for RawFinalizedHeaderMisbehaviour
{
    fn from(value: Misbehaviour<SYNC_COMMITTEE_SIZE>) -> Self {
        let data = match value.data {
            MisbehaviourData::FinalizedHeader(data) => data,
            _ => panic!("unexpected misbehaviour type"),
        };
        Self {
            client_id: value.client_id.as_str().to_string(),
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            consensus_update_1: Some(convert_consensus_update_to_proto(data.consensus_update_1)),
            consensus_update_2: Some(convert_consensus_update_to_proto(data.consensus_update_2)),
        }
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> From<Misbehaviour<SYNC_COMMITTEE_SIZE>>
    for RawNextSyncCommitteeMisbehaviour
{
    fn from(value: Misbehaviour<SYNC_COMMITTEE_SIZE>) -> Self {
        let data = match value.data {
            MisbehaviourData::NextSyncCommittee(data) => data,
            _ => panic!("unexpected misbehaviour type"),
        };
        Self {
            client_id: value.client_id.as_str().to_string(),
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            consensus_update_1: Some(convert_consensus_update_to_proto(data.consensus_update_1)),
            consensus_update_2: Some(convert_consensus_update_to_proto(data.consensus_update_2)),
        }
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> Protobuf<IBCAny> for Misbehaviour<SYNC_COMMITTEE_SIZE> {}

impl<const SYNC_COMMITTEE_SIZE: usize> TryFrom<IBCAny> for Misbehaviour<SYNC_COMMITTEE_SIZE> {
    type Error = Error;

    fn try_from(raw: IBCAny) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            ETHEREUM_FINALIZED_HEADER_MISBEHAVIOUR_TYPE_URL => {
                RawFinalizedHeaderMisbehaviour::decode(raw.value.as_slice())
                    .map_err(|_| Error::DecodeError)?
                    .try_into()
            }
            ETHEREUM_NEXT_SYNC_COMMITTEE_MISBEHAVIOUR_TYPE_URL => {
                RawNextSyncCommitteeMisbehaviour::decode(raw.value.as_slice())
                    .map_err(|_| Error::DecodeError)?
                    .try_into()
            }
            _ => Err(Error::UnknownTypeUrl),
        }
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> From<Misbehaviour<SYNC_COMMITTEE_SIZE>> for IBCAny {
    fn from(value: Misbehaviour<SYNC_COMMITTEE_SIZE>) -> Self {
        match value.data {
            MisbehaviourData::FinalizedHeader(_) => Self {
                type_url: ETHEREUM_FINALIZED_HEADER_MISBEHAVIOUR_TYPE_URL.to_string(),
                value: Protobuf::<RawFinalizedHeaderMisbehaviour>::encode_vec(&value),
            },
            MisbehaviourData::NextSyncCommittee(_) => Self {
                type_url: ETHEREUM_NEXT_SYNC_COMMITTEE_MISBEHAVIOUR_TYPE_URL.to_string(),
                value: Protobuf::<RawNextSyncCommitteeMisbehaviour>::encode_vec(&value),
            },
        }
    }
}
