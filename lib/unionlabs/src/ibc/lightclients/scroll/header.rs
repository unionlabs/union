use serde::{Deserialize, Serialize};

use crate::{
    errors::{required, MissingField},
    hash::H256,
    ibc::{
        core::client::height::Height,
        lightclients::{
            ethereum::{
                account_proof::{AccountProof, TryFromAccountProofError},
                storage_proof::StorageProof,
            },
            scroll::finalized_proof::{FinalizedProof, TryFromScrollFinalizedProofError},
        },
    },
    Proto, TypeUrl,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Header {
    pub l1_height: Height,
    pub l1_account_proof: AccountProof,
    pub l2_state_root: H256,
    pub finalized_proof: AccountProof,
    pub last_batch_index: u64,
    pub last_batch_index_proof: StorageProof,
    pub l2_ibc_account_proof: AccountProof,
}

impl Proto for Header {
    type Proto = protos::union::ibc::lightclients::scroll::v1::Header;
}

impl TypeUrl for protos::union::ibc::lightclients::scroll::v1::Header {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.scroll.v1.Header";
}

impl From<Header> for protos::union::ibc::lightclients::scroll::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            l1_account_proof: Some(value.l1_account_proof.into()),
            finalized_proof: Some(value.finalized_proof.into()),
            ibc_account_proof: Some(value.l2_ibc_account_proof.into()),
        }
    }
}

#[derive(Debug)]
pub enum TryFromHeaderError {
    MissingField(MissingField),
    L1AccountProof(TryFromAccountProofError),
    FinalizedProof(TryFromScrollFinalizedProofError),
    IbcAccountProof(TryFromAccountProofError),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::scroll::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_height: required!(value.l1_height)?.into(),
            l1_account_proof: required!(value.l1_account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L1AccountProof)?,
            finalized_proof: required!(value.finalized_proof)?
                .try_into()
                .map_err(TryFromHeaderError::FinalizedProof)?,
            l2_ibc_account_proof: required!(value.ibc_account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::IbcAccountProof)?,
        })
    }
}
