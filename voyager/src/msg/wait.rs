use std::{fmt::Display, marker::PhantomData};

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};

use crate::{
    chain::{ChainOf, HeightOf, LightClient, LightClientBase},
    msg::{any_enum, ChainIdOf},
};

any_enum! {
    /// Defines messages that are sent *to* the lightclient `L`.
    #[any = AnyWait]
    pub enum Wait<L: LightClient> {
        Block(WaitForBlock<L>),
        Timestamp(WaitForTimestamp<L>),
        TrustedHeight(WaitForTrustedHeight<L>),
    }
}

impl<L: LightClient> Display for Wait<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wait::Block(block) => write!(f, "Block({})", block.0),
            Wait::Timestamp(ts) => write!(f, "Timestamp({})", ts.timestamp),
            Wait::TrustedHeight(th) => write!(f, "TrustedHeight({})", th.height),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct WaitForBlock<L: LightClient>(pub HeightOf<ChainOf<L>>);

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct WaitForTimestamp<L: LightClient> {
    pub timestamp: i64,
    #[serde(skip)]
    pub __marker: PhantomData<L>,
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct WaitForTrustedHeight<L: LightClient> {
    pub client_id: L::ClientId,
    pub counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    pub counterparty_chain_id: ChainIdOf<L::Counterparty>,
    pub height: HeightOf<ChainOf<L::Counterparty>>,
}
