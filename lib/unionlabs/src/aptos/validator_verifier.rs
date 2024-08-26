use macros::model;

use super::public_key::PublicKey;
use crate::{
    aptos::account::AccountAddress,
    errors::{required, ExpectedLength, InvalidLength, MissingField},
};

/// Supports validation of signatures for known authors with individual voting powers. This struct
/// can be used for all signature verification operations including block and network signature
/// verification, respectively.
#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::ValidatorVerifier),
    into,
    from
))]
pub struct ValidatorVerifier {
    /// A vector of each validator's on-chain account address to its pubkeys and voting power.
    pub validator_infos: Vec<ValidatorConsensusInfo>,
}

/// Helper struct to manage validator information for validation
#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::ValidatorConsensusInfo),
    into,
    from
))]
pub struct ValidatorConsensusInfo {
    pub address: AccountAddress,
    pub public_key: PublicKey,
    pub voting_power: u64,
}

impl From<ValidatorVerifier> for protos::union::ibc::lightclients::movement::v1::ValidatorVerifier {
    fn from(value: ValidatorVerifier) -> Self {
        Self {
            validator_infos: value.validator_infos.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromValidatorVerifierError {
    #[error("invalid validator infos: {0}")]
    ValidatorInfos(#[from] TryFromValidatorConsensusInfo),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::ValidatorVerifier>
    for ValidatorVerifier
{
    type Error = TryFromValidatorVerifierError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::ValidatorVerifier,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_infos: value
                .validator_infos
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<ValidatorConsensusInfo>
    for protos::union::ibc::lightclients::movement::v1::ValidatorConsensusInfo
{
    fn from(value: ValidatorConsensusInfo) -> Self {
        Self {
            address: value.address.0.to_vec(),
            public_key: Some(value.public_key.into()),
            voting_power: value.voting_power,
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromValidatorConsensusInfo {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid address")]
    Address(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::ValidatorConsensusInfo>
    for ValidatorConsensusInfo
{
    type Error = TryFromValidatorConsensusInfo;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::ValidatorConsensusInfo,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            address: AccountAddress::new(value.address.as_slice().try_into().map_err(|_| {
                TryFromValidatorConsensusInfo::Address(InvalidLength {
                    expected: ExpectedLength::Exact(AccountAddress::LENGTH),
                    found: value.address.len(),
                })
            })?),
            public_key: required!(value.public_key)?.into(),
            voting_power: value.voting_power,
        })
    }
}
