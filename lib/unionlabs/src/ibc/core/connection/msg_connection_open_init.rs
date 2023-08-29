use serde::{Deserialize, Serialize};

use crate::{
    ibc::core::connection::{counterparty::Counterparty, version::Version},
    traits::Id as ClientIdTrait,
    CosmosAccountId, EmptyString, MsgIntoProto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "
        ClientId: Serialize,
        CounterpartyClientId: Serialize,
    ",
    deserialize = "
        ClientId: for<'d> Deserialize<'d>,
        CounterpartyClientId: for<'d> Deserialize<'d>,
    ",
))]
pub struct MsgConnectionOpenInit<ClientId, CounterpartyClientId> {
    pub client_id: ClientId,
    pub counterparty: Counterparty<CounterpartyClientId, EmptyString>,
    pub version: Version,
    pub delay_period: u64,
}

impl TypeUrl for protos::ibc::core::connection::v1::MsgConnectionOpenInit {
    const TYPE_URL: &'static str = "/ibc.core.connection.v1.MsgConnectionOpenInit";
}

impl<ClientId: ClientIdTrait, CounterpartyClientId: ClientIdTrait> MsgIntoProto
    for MsgConnectionOpenInit<ClientId, CounterpartyClientId>
{
    type Proto = protos::ibc::core::connection::v1::MsgConnectionOpenInit;

    fn into_proto_with_signer(self, signer: &CosmosAccountId) -> Self::Proto {
        Self::Proto {
            client_id: self.client_id.to_string(),
            counterparty: Some(self.counterparty.into()),
            version: Some(self.version.into()),
            delay_period: self.delay_period,
            signer: signer.to_string(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl<ClientId: ClientIdTrait, CounterpartyClientId: ClientIdTrait>
    From<MsgConnectionOpenInit<ClientId, CounterpartyClientId>>
    for contracts::ibc_handler::MsgConnectionOpenInit
{
    fn from(msg: MsgConnectionOpenInit<ClientId, CounterpartyClientId>) -> Self {
        Self {
            client_id: msg.client_id.to_string(),
            // TODO: Conversions for these types instead of constructing them manually
            counterparty: contracts::ibc_handler::IbcCoreConnectionV1CounterpartyData {
                client_id: msg.counterparty.client_id.to_string(),
                connection_id: msg.counterparty.connection_id.to_string(),
                prefix: contracts::ibc_handler::IbcCoreCommitmentV1MerklePrefixData {
                    key_prefix: msg.counterparty.prefix.key_prefix.into(),
                },
            },
            delay_period: msg.delay_period,
        }
    }
}
