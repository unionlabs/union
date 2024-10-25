use unionlabs::{
    bounded::BoundedU32, hash::H160, ibc::core::client::height::Height, id::ClientId, uint::U256,
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub l1_client_id: ClientId,
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
    use std::sync::Arc;

    use unionlabs::{
        bounded::BoundedIntError,
        errors::{InvalidLength, MissingField, UnknownEnumVariant},
        id::ClientIdValidator,
        impl_proto_via_try_from_into, required,
        uint::{FromDecStrErr, U256},
        validated::{Validate, ValidateT},
    };

    use crate::ClientState;

    impl_proto_via_try_from_into!(ClientState => protos::union::ibc::lightclients::arbitrum::v1::ClientState);

    impl TryFrom<protos::union::ibc::lightclients::arbitrum::v1::ClientState> for ClientState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::arbitrum::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                l1_client_id: value.l1_client_id.validate().map_err(Error::L1ClientId)?,
                chain_id: value
                    .chain_id
                    .parse()
                    .map_err(|err| Error::ChainId(Arc::new(err)))?,
                l1_latest_slot: value.l1_latest_slot,
                l1_contract_address: value
                    .l1_contract_address
                    .try_into()
                    .map_err(Error::L1ContractAddress)?,
                l1_next_node_num_slot: U256::try_from_be_bytes(&value.l1_next_node_num_slot)
                    .map_err(Error::L1NextNodeNumSlot)?,
                l1_nodes_slot: U256::try_from_be_bytes(&value.l1_nodes_slot)
                    .map_err(Error::L1NodesSlot)?,
                l1_next_node_num_slot_offset_bytes: value
                    .l1_next_node_num_slot_offset_bytes
                    .try_into()
                    .map_err(Error::L1NextNodeNumSlotOffsetBytes)?,
                l1_nodes_confirm_data_offset: U256::try_from_be_bytes(
                    &value.l1_nodes_confirm_data_offset,
                )
                .map_err(Error::ConfirmDataOffset)?,
                frozen_height: required!(value.frozen_height)?.into(),
                l2_ibc_contract_address: value
                    .l2_ibc_contract_address
                    .try_into()
                    .map_err(Error::L2IbcContractAddress)?,
                l2_ibc_commitment_slot: U256::try_from_be_bytes(&value.l2_ibc_commitment_slot)
                    .map_err(Error::L2IbcCommitmentSlot)?,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid l1_client_id")]
        L1ClientId(#[source] <ClientIdValidator as Validate<String>>::Error),
        #[error("invalid l1_contract_address")]
        ChainId(#[source] Arc<FromDecStrErr>),
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
                l1_client_id: value.l1_client_id.to_string(),
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
