use crate::{crypto::AuthorityPublicKeyBytes, U64};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Committee {
    pub epoch: U64,
    pub voting_rights: Vec<(AuthorityPublicKeyBytes, U64)>,
}
