use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::{
        client::height::Height,
        connection::{counterparty::Counterparty, version::Version},
    },
    CosmosAccountId, MsgIntoProto,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MsgConnectionOpenInit {
    pub client_id: String,
    pub counterparty: Counterparty,
    pub version: Version,
    pub delay_period: u64,
}

// REVIEW(benluelo): Is it possible to get this on ethereum? Or at least construct it somehow?
pub struct MsgConnectionOpenInitResponse {
    pub connection_id: String,
    pub inclusion_height: Height,
}

impl MsgIntoProto for MsgConnectionOpenInit {
    type Proto = protos::ibc::core::connection::v1::MsgConnectionOpenInit;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            client_id: self.client_id,
            counterparty: Some(self.counterparty.into()),
            version: Some(self.version.into()),
            delay_period: self.delay_period,
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<MsgConnectionOpenInit> for contracts::ibc_handler::MsgConnectionOpenInit {
    fn from(msg: MsgConnectionOpenInit) -> Self {
        Self {
            client_id: msg.client_id,
            // TODO: Conversions for these types instead of constructing them manually
            counterparty: contracts::ibc_handler::IbcCoreConnectionV1CounterpartyData {
                client_id: msg.counterparty.client_id,
                connection_id: msg.counterparty.connection_id,
                prefix: contracts::ibc_handler::IbcCoreCommitmentV1MerklePrefixData {
                    key_prefix: msg.counterparty.prefix.key_prefix.into(),
                },
            },
            delay_period: msg.delay_period,
        }
    }
}
