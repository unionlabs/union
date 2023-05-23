use crate::{
    errors::Error,
    eth_types::LightClientUpdate,
    // misbehaviour::Misbehaviour,
    types::{
        convert_consensus_update_to_proto, convert_proto_to_consensus_update, AccountUpdateInfo,
        TrustedSyncCommittee,
    },
};
use ibc::timestamp::Timestamp;
use prost::Message;
use protos::google::protobuf::Any as IBCAny;
use protos::union::ibc::lightclients::ethereum::v1::Header as RawHeader;

pub const ETHEREUM_HEADER_TYPE_URL: &str = "/ibc.lightclients.ethereum.v1.Header";

// TODO(aeryz): We might not need to represent the client message like this because the
// the size difference between the invariants are too much.
#[allow(clippy::large_enum_variant)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientMessage {
    Header(Header),
    // Misbehaviour(Misbehaviour),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub trusted_sync_committee: TrustedSyncCommittee,
    pub consensus_update: LightClientUpdate,
    pub account_update: AccountUpdateInfo,
    pub timestamp: Timestamp,
}

const NANO_SECONDS_MULTIPLIER: u64 = 1_000_000_000;

// impl Protobuf<RawHeader> for Header {}

impl TryFrom<RawHeader> for Header {
    type Error = Error;
    fn try_from(value: RawHeader) -> Result<Self, Self::Error> {
        let trusted_sync_committee = value.trusted_sync_committee.ok_or(Error::decode(
            "when decoding `trusted_sync_committee` in `Header`",
        ))?;
        let consensus_update = value
            .consensus_update
            .ok_or(Error::decode("no `consensus_update` in `Header`"))?;
        let account_update = value
            .account_update
            .ok_or(Error::decode("no `account_update` in `Header`"))?;

        Ok(Self {
            trusted_sync_committee: trusted_sync_committee.try_into()?,
            consensus_update: convert_proto_to_consensus_update(consensus_update)?,
            account_update: account_update.try_into()?,
            timestamp: Timestamp::from_nanoseconds(value.timestamp * NANO_SECONDS_MULTIPLIER)
                .map_err(|_| Error::decode("invalid timestamp in `Header`"))?,
        })
    }
}

impl From<Header> for RawHeader {
    fn from(value: Header) -> Self {
        let consensus_update = value.consensus_update;
        let account_update = value.account_update;

        Self {
            trusted_sync_committee: Some(value.trusted_sync_committee.into()),
            consensus_update: Some(convert_consensus_update_to_proto(consensus_update)),
            account_update: Some(account_update.into()),
            timestamp: value.timestamp.nanoseconds() / NANO_SECONDS_MULTIPLIER,
        }
    }
}

// impl Protobuf<IBCAny> for Header {}

impl TryFrom<IBCAny> for Header {
    type Error = Error;

    fn try_from(raw: IBCAny) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            ETHEREUM_HEADER_TYPE_URL => RawHeader::decode(raw.value.as_slice())
                .map_err(|_| Error::decode("when decoding `RawHeader` from `Any`"))?
                .try_into(),
            _ => Err(Error::UnknownTypeUrl),
        }
    }
}

// impl From<Header> for IBCAny {
//     fn from(header: Header) -> Self {
//         Self {
//             type_url: ETHEREUM_HEADER_TYPE_URL.to_string(),
//             value: Protobuf::<RawHeader>::encode_vec(&header),
//         }
//     }
// }
