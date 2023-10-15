use std::fmt::Debug;

use futures::Future;
use serde::{Deserialize, Serialize};
use unionlabs::{
    ibc::core::client::height::Height,
    proof::{
        AcknowledgementPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath,
        CommitmentPath, ConnectionPath, IbcPath,
    },
};

use crate::chain::Chain;

pub trait IbcStateRead<Counterparty: Chain, P: IbcPath<Self, Counterparty>>: Chain + Sized
where
    StateProof<P::Output>: Debug + Serialize,
{
    fn state_proof(
        &self,
        path: P,
        at: Self::Height,
    ) -> impl Future<Output = StateProof<P::Output>> + '_;
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StateProof<Data> {
    pub state: Data,
    #[serde(with = "::serde_utils::hex_string")]
    pub proof: Vec<u8>,
    pub proof_height: Height,
}

impl<Data: Debug> Debug for StateProof<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateProof")
            .field("state", &self.state)
            .field("proof", &serde_utils::to_hex(&self.proof))
            .field("proof_height", &self.proof_height)
            .finish()
    }
}

// NOTE: Commented out for now, may reuse this in the future
// macro_rules! ibc_paths (
//     (
//         $(
//             #[display($fmt:literal)]
//             #[output($Output:ty)]
//             pub struct $Struct:ident$(<$($generics:ident$(: $bound:ident)?),+>)? {
//                 $(pub $field:ident: $field_ty:ty,)+
//             }
//         )+
//     ) => {
//         $(
//             #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // clap::Args
//             pub struct $Struct$(<$($generics),+>)? {
//                 $(pub $field: $field_ty,)+
//             }

//             impl$(<$($generics: Display),+>)? Display for $Struct {
//                 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//                     let Self { $($field,)+ } = self;
//                     write!(f, $fmt)
//                 }
//             }

//             impl<$($($generics: Display,)+)? This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for $Struct$(<$($generics),+>)? {
//                 type Output = $Output;
//             }

//         )+

//         enum_variants_conversions! {
//             #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//             pub enum Path {
//                 $(
//                     $Struct($Struct),
//                 )+
//             }
//         }

//         pub trait IbcStateReadPaths<Counterparty: Chain>: Chain + $(IbcStateRead<Counterparty, $Struct>+)+ {}

//         impl<Counterparty: Chain, T: Chain> IbcStateReadPaths<Counterparty> for T
//             where
//                 T: $(IbcStateRead<Counterparty, $Struct>+)+
//         {}
//     }
// );

pub trait IbcStateReadPaths<Counterparty: Chain>:
    Chain
    + IbcStateRead<Counterparty, ClientStatePath<<Self as Chain>::ClientId>>
    + IbcStateRead<
        Counterparty,
        ClientConsensusStatePath<<Self as Chain>::ClientId, Counterparty::Height>,
    > + IbcStateRead<Counterparty, ConnectionPath>
    + IbcStateRead<Counterparty, ChannelEndPath>
    + IbcStateRead<Counterparty, CommitmentPath>
    + IbcStateRead<Counterparty, AcknowledgementPath>
{
}

impl<Counterparty: Chain, T: Chain> IbcStateReadPaths<Counterparty> for T where
    T: IbcStateRead<Counterparty, ClientStatePath<Self::ClientId>>
        + IbcStateRead<Counterparty, ClientConsensusStatePath<Self::ClientId, Counterparty::Height>>
        + IbcStateRead<Counterparty, ConnectionPath>
        + IbcStateRead<Counterparty, ChannelEndPath>
        + IbcStateRead<Counterparty, CommitmentPath>
        + IbcStateRead<Counterparty, AcknowledgementPath>
{
}
