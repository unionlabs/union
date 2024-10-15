use macros::model;

use crate::{
    bounded::BoundedU32, hash::H160, ibc::core::client::height::Height, id::ClientId, uint::U256,
};

#[model(proto(
    raw(protos::union::ibc::lightclients::arbitrum::v1::ClientState),
    from,
    into
))]
pub struct ClientState {
    pub l1_client_id: ClientId,
    pub l1_client_type: String,
    pub chain_id: U256,
    pub l1_latest_slot: u64,
    pub l1_contract_address: H160,
    pub l1_next_node_num_slot: U256,
    pub l1_nodes_slot: U256,
    // this is a u32 because protobuf sucks
    pub l1_next_node_num_slot_offset_bytes: BoundedU32<0, 24>,
    // TODO: Rename this in the protos
    pub l1_nodes_confirm_data_offset: U256,
    pub frozen_height: Height,
    pub l2_ibc_contract_address: H160,
    pub l2_ibc_commitment_slot: U256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use core::str::FromStr;

    use crate::{
        bounded::BoundedIntError,
        errors::{required, InvalidLength, MissingField, UnknownEnumVariant},
        ibc::lightclients::arbitrum::client_state::ClientState,
        id::{ClientId, ParsePrefixedIdError},
        uint::U256,
    };

    impl TryFrom<protos::union::ibc::lightclients::arbitrum::v1::ClientState> for ClientState {
        type Error = TryFromClientStateError;

        fn try_from(
            value: protos::union::ibc::lightclients::arbitrum::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            let (l1_client_type, l1_client_id) = ClientId::parse_prefixed(&value.l1_client_id)
                .map_err(TryFromClientStateError::L1ClientId)?;

            Ok(Self {
                l1_client_id,
                l1_client_type: l1_client_type.to_owned(),
                chain_id: value
                    .chain_id
                    .parse()
                    .map_err(TryFromClientStateError::ChainId)?,
                l1_latest_slot: value.l1_latest_slot,
                l1_contract_address: value
                    .l1_contract_address
                    .try_into()
                    .map_err(TryFromClientStateError::L1ContractAddress)?,
                l1_next_node_num_slot: U256::try_from_be_bytes(&value.l1_next_node_num_slot)
                    .map_err(TryFromClientStateError::L1NextNodeNumSlot)?,
                l1_nodes_slot: U256::try_from_be_bytes(&value.l1_nodes_slot)
                    .map_err(TryFromClientStateError::L1NodesSlot)?,
                l1_next_node_num_slot_offset_bytes: value
                    .l1_next_node_num_slot_offset_bytes
                    .try_into()
                    .map_err(TryFromClientStateError::L1NextNodeNumSlotOffsetBytes)?,
                l1_nodes_confirm_data_offset: U256::try_from_be_bytes(
                    &value.l1_nodes_confirm_data_offset,
                )
                .map_err(TryFromClientStateError::ConfirmDataOffset)?,
                frozen_height: required!(value.frozen_height)?.into(),
                l2_ibc_contract_address: value
                    .l2_ibc_contract_address
                    .try_into()
                    .map_err(TryFromClientStateError::L2IbcContractAddress)?,
                l2_ibc_commitment_slot: U256::try_from_be_bytes(&value.l2_ibc_commitment_slot)
                    .map_err(TryFromClientStateError::L2IbcCommitmentSlot)?,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromClientStateError {
        #[error(transparent)]
        MissingField(MissingField),
        #[error("invalid l1_client_id")]
        L1ClientId(#[source] ParsePrefixedIdError),
        #[error("invalid l1_contract_address")]
        ChainId(#[source] <U256 as FromStr>::Err),
        #[error("invalid l1_latest_confirmed_slot")]
        L1ContractAddress(#[source] InvalidLength),
        #[error("invalid l1_nodes_slot")]
        L1NextNodeNumSlot(#[source] InvalidLength),
        #[error("invalid confirm_data_offset")]
        L1NodesSlot(#[source] InvalidLength),
        #[error("invalid l1_next_node_num_slot_offset_bytes")]
        L1NextNodeNumSlotOffsetBytes(#[source] BoundedIntError<u32>),
        #[error("invalid frozen_height")]
        ConfirmDataOffset(#[source] InvalidLength),
        #[error("invalid l2_ibc_commitment_slot")]
        L2IbcContractAddress(#[source] InvalidLength),
        #[error("invalid l2_ibc_commitment_slot")]
        L2IbcCommitmentSlot(#[source] InvalidLength),
        #[error("invalid finality")]
        Finality(UnknownEnumVariant<i32>),
    }

    impl From<ClientState> for protos::union::ibc::lightclients::arbitrum::v1::ClientState {
        fn from(value: ClientState) -> Self {
            Self {
                l1_client_id: value.l1_client_id.to_string_prefixed(&value.l1_client_type),
                chain_id: value.chain_id.to_string(),
                l1_latest_slot: value.l1_latest_slot,
                l1_contract_address: value.l1_contract_address.into(),
                l1_next_node_num_slot: value.l1_next_node_num_slot.to_be_bytes().to_vec(),
                l1_nodes_slot: value.l1_nodes_slot.to_be_bytes().to_vec(),
                l1_next_node_num_slot_offset_bytes: value.l1_next_node_num_slot_offset_bytes.into(),
                l1_nodes_confirm_data_offset: value
                    .l1_nodes_confirm_data_offset
                    .to_be_bytes()
                    .to_vec(),
                frozen_height: Some(value.frozen_height.into()),
                l2_ibc_contract_address: value.l2_ibc_contract_address.into(),
                l2_ibc_commitment_slot: value.l2_ibc_commitment_slot.to_be_bytes().to_vec(),
            }
        }
    }
}
