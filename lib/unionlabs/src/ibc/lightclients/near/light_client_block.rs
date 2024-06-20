use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::hash::CryptoHash;

use super::{
    block_header_inner::{BlockHeaderInnerLiteView, TryFromBlockHeaderInnerLiteViewError},
    validator_stake::{TryFromValidatorStakeView, ValidatorStakeView},
};
use crate::{
    errors::{required, MissingField},
    near::types::Signature,
};

#[derive(
    PartialEq,
    Eq,
    Debug,
    Clone,
    BorshDeserialize,
    BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct LightClientBlockView {
    pub prev_block_hash: CryptoHash,
    pub next_block_inner_hash: CryptoHash,
    pub inner_lite: BlockHeaderInnerLiteView,
    pub inner_rest_hash: CryptoHash,
    pub next_bps: Option<Vec<ValidatorStakeView>>,
    pub approvals_after_next: Vec<Option<Box<Signature>>>,
}

impl From<LightClientBlockView>
    for protos::union::ibc::lightclients::near::v1::LightClientBlockView
{
    fn from(value: LightClientBlockView) -> Self {
        Self {
            prev_block_hash: value.prev_block_hash.into(),
            next_block_inner_hash: value.next_block_inner_hash.into(),
            inner_lite: Some(value.inner_lite.into()),
            inner_rest_hash: value.inner_rest_hash.into(),
            next_bps: match value.next_bps {
                Some(next_bps) => next_bps.into_iter().map(Into::into).collect(),
                None => vec![],
            },
            approvals_after_next: value
                .approvals_after_next
                .into_iter()
                .map(
                    |sig| protos::union::ibc::lightclients::near::v1::Signature {
                        signature: sig.map(|sig| {
                            Into::<
                            protos::union::ibc::lightclients::near::v1::signature::Signature,
                        >::into(*sig)
                        }),
                    },
                )
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromLightClientBlockView {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid prev block hash")]
    PrevBlockHash,
    #[error("invalid next block inner hash")]
    NextBlockInnerHash,
    #[error(transparent)]
    InnerLite(#[from] TryFromBlockHeaderInnerLiteViewError),
    #[error("invalid inner rest hash")]
    InnerRestHash,
    #[error(transparent)]
    NextBps(#[from] TryFromValidatorStakeView),
}

impl TryFrom<protos::union::ibc::lightclients::near::v1::LightClientBlockView>
    for LightClientBlockView
{
    type Error = TryFromLightClientBlockView;

    fn try_from(
        value: protos::union::ibc::lightclients::near::v1::LightClientBlockView,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            prev_block_hash: value
                .prev_block_hash
                .as_slice()
                .try_into()
                .map_err(|_| TryFromLightClientBlockView::PrevBlockHash)?,
            next_block_inner_hash: value
                .next_block_inner_hash
                .as_slice()
                .try_into()
                .map_err(|_| TryFromLightClientBlockView::NextBlockInnerHash)?,
            inner_lite: required!(value.inner_lite)?
                .try_into()
                .map_err(TryFromLightClientBlockView::InnerLite)?,
            inner_rest_hash: value
                .inner_rest_hash
                .as_slice()
                .try_into()
                .map_err(|_| TryFromLightClientBlockView::InnerRestHash)?,
            next_bps: Some(
                value
                    .next_bps
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<ValidatorStakeView>, _>>()
                    .map_err(TryFromLightClientBlockView::NextBps)?,
            ),
            approvals_after_next: value
                .approvals_after_next
                .into_iter()
                .map(|item| item.signature.map(|sig| Box::new(sig.into())))
                .collect(),
        })
    }
}
