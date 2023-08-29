use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::{ethereum::H256, events::IbcEvent};

use crate::{
    chain::{ChainOf, HeightOf, LightClient},
    msg::{identified, AnyLightClient},
};

pub enum AnyEvent {}

impl AnyLightClient for AnyEvent {
    type Inner<L: LightClient> = identified!(Event<L>);
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Event<L: LightClient> {
    pub block_hash: H256,
    pub height: HeightOf<ChainOf<L>>,
    pub event: IbcEvent<L::ClientId, L::ClientType, <L::Counterparty as LightClient>::ClientId>,
}
