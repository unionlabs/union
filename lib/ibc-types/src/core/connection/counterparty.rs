use contracts::ibc_handler::IbcCoreConnectionV1CounterpartyData;

use crate::{core::commitment::merkle_prefix::MerklePrefix, errors::MissingField};

#[derive(Debug, Clone)]
pub struct Counterparty {
    pub client_id: String,
    pub connection_id: String,
    pub prefix: MerklePrefix,
}

impl From<Counterparty> for protos::ibc::core::connection::v1::Counterparty {
    fn from(value: Counterparty) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: Some(value.prefix.into()),
        }
    }
}

impl TryFrom<protos::ibc::core::connection::v1::Counterparty> for Counterparty {
    type Error = MissingField;

    fn try_from(
        value: protos::ibc::core::connection::v1::Counterparty,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.ok_or(MissingField("prefix"))?.into(),
        })
    }
}

impl From<Counterparty> for IbcCoreConnectionV1CounterpartyData {
    fn from(value: Counterparty) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.into(),
        }
    }
}

impl From<IbcCoreConnectionV1CounterpartyData> for Counterparty {
    fn from(value: IbcCoreConnectionV1CounterpartyData) -> Self {
        Self {
            client_id: value.client_id,
            connection_id: value.connection_id,
            prefix: value.prefix.into(),
        }
    }
}
