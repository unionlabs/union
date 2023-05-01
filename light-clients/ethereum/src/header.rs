use crate::errors::Error;
use crate::misbehaviour::Misbehaviour;
use crate::types::{
    convert_consensus_update_to_proto, convert_execution_update_to_proto,
    convert_proto_to_consensus_update, convert_proto_to_execution_update, AccountUpdateInfo,
    TrustedSyncCommittee,
};
use crate::update::{ConsensusUpdateInfo, ExecutionUpdateInfo};
use ibc::timestamp::Timestamp;
use ibc_proto::google::protobuf::Any as IBCAny;
use ibc_proto::ibc::lightclients::ethereum::v1::Header as RawHeader;
use ibc_proto::protobuf::Protobuf;
use prost::Message;

pub const ETHEREUM_HEADER_TYPE_URL: &str = "/ibc.lightclients.ethereum.v1.Header";

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientMessage<const SYNC_COMMITTEE_SIZE: usize> {
    Header(Header<SYNC_COMMITTEE_SIZE>),
    Misbehaviour(Misbehaviour<SYNC_COMMITTEE_SIZE>),
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Header<const SYNC_COMMITTEE_SIZE: usize> {
    pub trusted_sync_committee: TrustedSyncCommittee<SYNC_COMMITTEE_SIZE>,
    pub consensus_update: ConsensusUpdateInfo<SYNC_COMMITTEE_SIZE>,
    pub execution_update: ExecutionUpdateInfo,
    pub account_update: AccountUpdateInfo,
    pub timestamp: Timestamp,
}

impl<const SYNC_COMMITTEE_SIZE: usize> Protobuf<RawHeader> for Header<SYNC_COMMITTEE_SIZE> {}

impl<const SYNC_COMMITTEE_SIZE: usize> TryFrom<RawHeader> for Header<SYNC_COMMITTEE_SIZE> {
    type Error = Error;
    fn try_from(value: RawHeader) -> Result<Self, Self::Error> {
        let trusted_sync_committee = value.trusted_sync_committee.ok_or(Error::DecodeError)?;
        let consensus_update = value.consensus_update.ok_or(Error::DecodeError)?;
        let execution_update = value.execution_update.ok_or(Error::DecodeError)?;
        let account_update = value.account_update.ok_or(Error::DecodeError)?;

        Ok(Self {
            trusted_sync_committee: trusted_sync_committee.try_into()?,
            consensus_update: convert_proto_to_consensus_update(consensus_update)?,
            execution_update: convert_proto_to_execution_update(execution_update),
            account_update: account_update.try_into()?,
            timestamp: Timestamp::from_nanoseconds(value.timestamp * 1_000_000_000)
                .map_err(|_| Error::DecodeError)?,
        })
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> From<Header<SYNC_COMMITTEE_SIZE>> for RawHeader {
    fn from(value: Header<SYNC_COMMITTEE_SIZE>) -> Self {
        let consensus_update = value.consensus_update;
        let execution_update = value.execution_update;
        let account_update = value.account_update;

        Self {
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            consensus_update: Some(convert_consensus_update_to_proto(consensus_update)),
            execution_update: Some(convert_execution_update_to_proto(execution_update)),
            account_update: Some(account_update.into()),
            timestamp: value.timestamp.nanoseconds() / 1_000_000_000,
        }
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> Protobuf<IBCAny> for Header<SYNC_COMMITTEE_SIZE> {}

impl<const SYNC_COMMITTEE_SIZE: usize> TryFrom<IBCAny> for Header<SYNC_COMMITTEE_SIZE> {
    type Error = Error;

    fn try_from(raw: IBCAny) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            ETHEREUM_HEADER_TYPE_URL => RawHeader::decode(raw.value.as_slice())
                .map_err(|_| Error::DecodeError)?
                .try_into(),
            _ => Err(Error::UnknownTypeUrl),
        }
    }
}

impl<const SYNC_COMMITTEE_SIZE: usize> From<Header<SYNC_COMMITTEE_SIZE>> for IBCAny {
    fn from(header: Header<SYNC_COMMITTEE_SIZE>) -> Self {
        Self {
            type_url: ETHEREUM_HEADER_TYPE_URL.to_string(),
            value: Protobuf::<RawHeader>::encode_vec(&header),
        }
    }
}
