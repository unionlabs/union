use unionlabs::{hash::H160, ibc::core::client::height::Height, uint::U256};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientState {
    pub chain_id: U256,
    // TODO: This should be ClientId
    pub l1_client_id: String,
    pub l1_latest_height: Height,
    pub l1_rollup_contract_address: H160,
    pub l1_rollup_current_l2_timestamp_slot: U256,
    pub l1_rollup_current_l2_block_number_slot: U256,
    pub l1_rollup_l2_state_root_hashes_slot: U256,
    pub l2_ibc_contract_address: H160,
    pub frozen_height: Height,
}

#[cfg(feature = "proto")]
pub mod proto {
    use std::{str::FromStr, sync::Arc};

    use unionlabs::{
        errors::{InvalidLength, MissingField},
        impl_proto_via_try_from_into, required,
        uint::{FromDecStrErr, U256},
    };

    use crate::ClientState;

    impl_proto_via_try_from_into!(ClientState => protos::union::ibc::lightclients::linea::v1::ClientState);

    impl From<ClientState> for protos::union::ibc::lightclients::linea::v1::ClientState {
        fn from(value: ClientState) -> Self {
            Self {
                chain_id: value.chain_id.to_string(),
                l1_client_id: value.l1_client_id,
                l1_latest_height: Some(value.l1_latest_height.into()),
                l1_rollup_contract_address: value.l1_rollup_contract_address.into(),
                l1_rollup_current_l2_block_number_slot: value
                    .l1_rollup_current_l2_block_number_slot
                    .to_be_bytes()
                    .to_vec(),
                l1_rollup_current_l2_timestamp_slot: value
                    .l1_rollup_current_l2_timestamp_slot
                    .to_be_bytes()
                    .to_vec(),
                l1_rollup_l2_state_root_hashes_slot: value
                    .l1_rollup_l2_state_root_hashes_slot
                    .to_be_bytes()
                    .to_vec(),
                l2_ibc_contract_address: value.l2_ibc_contract_address.into(),
                frozen_height: Some(value.frozen_height.into()),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        // y no clone?!??
        #[error("unable to parse chain id")]
        ChainId(#[source] Arc<FromDecStrErr>),
        #[error("invalid l1 latest height")]
        L1LatestHeight,
        #[error("invalid rollup contract address")]
        L1RollupContractAddress(#[source] InvalidLength),
        #[error("invalid rollup current_l2_block_number slot")]
        L1RollupCurrentL2BlockNumberSlot(#[source] InvalidLength),
        #[error("invalid rollup current_l2_timestamp slot")]
        L1RollupCurrentL2TimestampSlot(#[source] InvalidLength),
        #[error("invalid rollup l2_state_roots mapping slot")]
        L1RollupL2StateRootHashesSlot(#[source] InvalidLength),
        #[error("invalid l2 ibc contract address")]
        L2IbcContractAddress(#[source] InvalidLength),
    }

    impl TryFrom<protos::union::ibc::lightclients::linea::v1::ClientState> for ClientState {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::linea::v1::ClientState,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                l1_client_id: value.l1_client_id,
                chain_id: U256::from_str(&value.chain_id)
                    .map_err(|err| Error::ChainId(Arc::new(err)))?,
                l1_latest_height: required!(value.l1_latest_height)?.into(),
                l1_rollup_contract_address: value
                    .l1_rollup_contract_address
                    .try_into()
                    .map_err(Error::L1RollupContractAddress)?,
                l1_rollup_current_l2_block_number_slot: U256::try_from_be_bytes(
                    &value.l1_rollup_current_l2_block_number_slot,
                )
                .map_err(Error::L1RollupCurrentL2BlockNumberSlot)?,
                l1_rollup_current_l2_timestamp_slot: U256::try_from_be_bytes(
                    &value.l1_rollup_current_l2_timestamp_slot,
                )
                .map_err(Error::L1RollupCurrentL2TimestampSlot)?,
                l1_rollup_l2_state_root_hashes_slot: U256::try_from_be_bytes(
                    &value.l1_rollup_l2_state_root_hashes_slot,
                )
                .map_err(Error::L1RollupL2StateRootHashesSlot)?,
                l2_ibc_contract_address: value
                    .l2_ibc_contract_address
                    .try_into()
                    .map_err(Error::L2IbcContractAddress)?,
                frozen_height: value.frozen_height.unwrap_or_default().into(),
            })
        }
    }
}
