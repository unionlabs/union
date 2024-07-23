use borsh::{BorshDeserialize, BorshSerialize};
use macros::model;
use near_account_id::AccountId;
use near_primitives_core::account::id::ParseAccountError;

use super::validator_stake_view::{TryFromValidatorStakeView, ValidatorStakeView};

#[model(proto(
    raw(protos::union::ibc::lightclients::near::v1::ClientState),
    into,
    from
))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientState {
    pub chain_id: String,
    pub latest_height: u64,
    pub ibc_account_id: AccountId,
    pub initial_block_producers: Vec<ValidatorStakeView>,
    // TODO: Make option
    pub frozen_height: u64,
}

impl From<ClientState> for protos::union::ibc::lightclients::near::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            chain_id: value.chain_id,
            latest_height: value.latest_height,
            account_id: value.ibc_account_id.into(),
            iniitial_block_producers: value
                .initial_block_producers
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error(transparent)]
    AccountId(#[from] ParseAccountError),
    #[error(transparent)]
    InitialBlockProducers(#[from] TryFromValidatorStakeView),
}

impl TryFrom<protos::union::ibc::lightclients::near::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::near::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id,
            latest_height: value.latest_height,
            ibc_account_id: value
                .account_id
                .try_into()
                .map_err(TryFromClientStateError::AccountId)?,
            initial_block_producers: value
                .iniitial_block_producers
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromClientStateError::InitialBlockProducers)?,
            frozen_height: 0,
        })
    }
}
