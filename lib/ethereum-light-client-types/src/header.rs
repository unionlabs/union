use unionlabs::ibc::core::client::height::Height;

use crate::{AccountProof, LightClientUpdate};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Header {
    /// The currently trusted height of the light client to apply this update against.
    pub trusted_height: Height,

    /// The actual update data to be applied.
    pub consensus_update: LightClientUpdate,

    /// Proof of the IBC handler contract against the execution state root provided in `consensus_update`.
    pub ibc_account_proof: AccountProof,
}
