use macros::model;

use crate::cosmos::{
    base::query::page_response::PageResponse,
    staking::validator::{TryFromValidatorError, Validator},
};

#[model(proto(raw(protos::cosmos::staking::v1beta1::QueryValidatorsResponse), into))]
pub struct QueryValidatorsResponse {
    pub validators: Vec<Validator>,
    pub pagination: Option<PageResponse>,
}

#[derive(Debug)]
pub enum TryFromQueryValidatorsResponseError {
    Validators(TryFromValidatorError),
}

impl TryFrom<protos::cosmos::staking::v1beta1::QueryValidatorsResponse>
    for QueryValidatorsResponse
{
    type Error = TryFromQueryValidatorsResponseError;

    fn try_from(
        value: protos::cosmos::staking::v1beta1::QueryValidatorsResponse,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            validators: value
                .validators
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromQueryValidatorsResponseError::Validators)?,
            pagination: value.pagination.map(Into::into),
        })
    }
}
