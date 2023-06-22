use crate::lightclients::ethereum::proof::Proof;

#[derive(Debug, Clone, PartialEq)]
pub struct AccountUpdate {
    pub proofs: Vec<Proof>,
}

impl From<AccountUpdate> for protos::union::ibc::lightclients::ethereum::v1::AccountUpdate {
    fn from(value: AccountUpdate) -> Self {
        Self {
            proofs: value.proofs.into_iter().map(Into::into).collect(),
        }
    }
}
