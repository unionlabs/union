use macros::model;

use crate::{hash::H160, uint::U256};

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
    pub ibc_commitment_slot: U256,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{
        errors::InvalidLength, ibc::lightclients::evm_in_cosmos::client_state::ClientState,
        uint::U256,
    };

    impl From<ClientState> for protos::union::ibc::lightclients::evmincosmos::v1::ClientState {
        fn from(value: ClientState) -> Self {
            Self {
                l1_client_id: value.l1_client_id,
                l2_client_id: value.l2_client_id,
                latest_slot: value.latest_slot,
                ibc_contract_address: value.ibc_contract_address.into(),
                ibc_commitment_slot: value.ibc_commitment_slot.to_be_bytes().into(),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromClientStateError {
        #[error("invalid ibc contract address")]
        IbcContractAddress(#[source] InvalidLength),
        #[error("invalid ibc commitment slot")]
        IbcCommitmentSlot(#[source] InvalidLength),
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
                ibc_commitment_slot: U256::try_from_be_bytes(&value.ibc_commitment_slot)
                    .map_err(TryFromClientStateError::IbcCommitmentSlot)?,
            })
        }
    }
}
