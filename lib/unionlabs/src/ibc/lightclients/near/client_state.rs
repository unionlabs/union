use borsh::{BorshDeserialize, BorshSerialize};
use macros::model;
use near_primitives_core::account::id::ParseAccountError;
use near_sdk::AccountId;

use super::validator_stake::{TryFromValidatorStakeView, ValidatorStakeView};

#[model(proto(
    raw(protos::union::ibc::lightclients::near::v1::ClientState),
    into,
    from
))]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientState {
    latest_height: u64,
    ibc_account_id: AccountId,
    initial_block_producers: Option<Vec<ValidatorStakeView>>,
}

impl From<ClientState> for protos::union::ibc::lightclients::near::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            latest_height: value.latest_height,
            account_id: value.ibc_account_id.into(),
            iniitial_block_producers: value
                .initial_block_producers
                .unwrap_or(Vec::new())
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
            latest_height: value.latest_height,
            ibc_account_id: value
                .account_id
                .try_into()
                .map_err(TryFromClientStateError::AccountId)?,
            initial_block_producers: if !value.iniitial_block_producers.is_empty() {
                Some(
                    value
                        .iniitial_block_producers
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(TryFromClientStateError::InitialBlockProducers)?,
                )
            } else {
                None
            },
        })
    }
}
