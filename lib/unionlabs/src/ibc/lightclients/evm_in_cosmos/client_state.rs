use core::fmt::Debug;

use macros::model;

use crate::{errors::InvalidLength, hash::H160, uint::U256};

#[model(proto(
    raw(protos::union::ibc::lightclients::evmincosmos::v1::ClientState),
    into,
    from
))]
pub struct ClientState {
    // TODO: This should be ClientId
    pub l1_client_id: String,
    pub l2_client_id: String,
    pub latest_slot: u64,
    pub ibc_contract_address: H160,
}

impl From<ClientState> for protos::union::ibc::lightclients::evmincosmos::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            l1_client_id: value.l1_client_id,
            l2_client_id: value.l2_client_id,
            latest_slot: value.latest_slot,
            ibc_contract_address: value.ibc_contract_address.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error("invalid ibc contract address")]
    IbcContractAddress(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::evmincosmos::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::evmincosmos::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_client_id: value.l1_client_id,
            l2_client_id: value.l2_client_id,
            latest_slot: value.latest_slot,
            ibc_contract_address: value
                .ibc_contract_address
                .try_into()
                .map_err(TryFromClientStateError::IbcContractAddress)?,
        })
    }
}
