use serde::{Deserialize, Serialize};

use crate::{ibc::lightclients::ethereum::proof::Proof, Proto, TypeUrl};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountUpdate {
    pub proofs: Vec<Proof>,
}

impl From<AccountUpdate> for protos::union::ibc::lightclients::ethereum::v1::AccountUpdate {
    fn from(value: AccountUpdate) -> Self {
        Self {
            proofs: value
                .proofs
                .into_iter()
                .map(|proof| Into::<Proof>::into(proof).into())
                .collect(),
        }
    }
}

impl From<protos::union::ibc::lightclients::ethereum::v1::AccountUpdate> for AccountUpdate {
    fn from(value: protos::union::ibc::lightclients::ethereum::v1::AccountUpdate) -> Self {
        Self {
            proofs: value.proofs.into_iter().map(Into::into).collect(),
        }
    }
}

impl Proto for AccountUpdate {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::AccountUpdate;
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::AccountUpdate {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.AccountUpdate";
}
