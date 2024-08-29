use core::str::FromStr;

use macros::model;

use crate::{
    aptos::account::AccountAddress,
    errors::{ExpectedLength, InvalidLength},
    hash::H160,
    ibc::core::client::height::Height,
    id::ClientId,
};

#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::ClientState),
    into,
    from
))]
pub struct ClientState {
    pub l1_client_id: ClientId,
    pub l1_contract_address: H160,
    // TODO(aeryz): this is not H160
    pub l2_contract_address: H160,
    pub table_handle: AccountAddress,
    pub frozen_height: Height,
    pub latest_block_num: u64,
}

impl From<ClientState> for protos::union::ibc::lightclients::movement::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            l1_client_id: value.l1_client_id.to_string(),
            l1_contract_address: value.l1_contract_address.into(),
            l2_contract_address: value.l2_contract_address.into(),
            table_handle: value.table_handle.0.to_vec(),
            frozen_height: Some(value.frozen_height.into()),
            latest_block_num: value.latest_block_num,
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error("invalid l1 client id")]
    L1ClientId(#[source] <ClientId as FromStr>::Err),
    #[error("invalid l1 contract address")]
    L1ContractAddress(#[source] InvalidLength),
    #[error("invalid l2 contract address")]
    L2ContractAddress(#[source] InvalidLength),
    #[error("invalid table handle")]
    TableHandle(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_client_id: ClientId::from_str(&value.l1_client_id)
                .map_err(TryFromClientStateError::L1ClientId)?,
            l1_contract_address: value
                .l1_contract_address
                .try_into()
                .map_err(TryFromClientStateError::L1ContractAddress)?,
            l2_contract_address: value
                .l2_contract_address
                .try_into()
                .map_err(TryFromClientStateError::L2ContractAddress)?,
            table_handle: AccountAddress::new(value.table_handle.as_slice().try_into().map_err(
                |_| {
                    TryFromClientStateError::TableHandle(InvalidLength {
                        expected: ExpectedLength::Exact(AccountAddress::LENGTH),
                        found: value.table_handle.len(),
                    })
                },
            )?),
            frozen_height: value.frozen_height.unwrap_or_default().into(),
            latest_block_num: value.latest_block_num,
        })
    }
}
