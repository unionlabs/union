use crate::{ObjectID, committee::Committee, object::TypeTag};

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum ClientState {
    V1(ClientStateV1),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientStateV1 {
    pub chain_id: String,
    pub latest_checkpoint: u64,
    pub frozen_height: u64,
    /// The main store of IBC where the commitments are added as dynamic fields
    pub ibc_store: ObjectID,
    /// The commitment key type that is used to generate a commitment key.
    ///
    /// Note that this is tied to an address and will be something like `0x0123...::ibc::CommitmentKey`.
    /// We don't want to derive this from the ibc address since the new keys that might be introduces
    /// could have separate addresses. So it's more correct to get the key directly.
    pub commitmeny_key_type: TypeTag,
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub initial_committee: Option<Committee>,
}
