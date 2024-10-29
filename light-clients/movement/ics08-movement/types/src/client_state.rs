use unionlabs::{
    aptos::account::AccountAddress, hash::H160, ibc::core::client::height::Height, id::ClientId,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub chain_id: String,
    pub l1_client_id: ClientId,
    pub l1_contract_address: H160,
    pub l2_contract_address: AccountAddress,
    pub table_handle: AccountAddress,
    pub frozen_height: Height,
    pub latest_block_num: u64,
}

#[cfg(feature = "proto")]
pub mod proto {
    use std::str::FromStr;

    use unionlabs::{
        aptos::account::AccountAddress, errors::InvalidLength, id::ClientId,
        impl_proto_via_try_from_into,
    };

    use crate::ClientState;

    impl_proto_via_try_from_into!(ClientState => protos::union::ibc::lightclients::movement::v1::ClientState);

    impl From<ClientState> for protos::union::ibc::lightclients::movement::v1::ClientState {
        fn from(value: ClientState) -> Self {
            Self {
                chain_id: value.chain_id,
                l1_client_id: value.l1_client_id.to_string(),
                l1_contract_address: value.l1_contract_address.into(),
                l2_contract_address: value.l2_contract_address.0.into_bytes(),
                table_handle: value.table_handle.0.into_bytes(),
                frozen_height: Some(value.frozen_height.into()),
                latest_block_num: value.latest_block_num,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
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
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::movement::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                l1_client_id: ClientId::from_str(&value.l1_client_id).map_err(Error::L1ClientId)?,
                l1_contract_address: value
                    .l1_contract_address
                    .try_into()
                    .map_err(Error::L1ContractAddress)?,
                l2_contract_address: AccountAddress(
                    value
                        .l2_contract_address
                        .as_slice()
                        .try_into()
                        .map_err(Error::L2ContractAddress)?,
                ),
                table_handle: AccountAddress(
                    value
                        .table_handle
                        .as_slice()
                        .try_into()
                        .map_err(Error::TableHandle)?,
                ),
                frozen_height: value.frozen_height.unwrap_or_default().into(),
                latest_block_num: value.latest_block_num,
                chain_id: value.chain_id,
            })
        }
    }
}
