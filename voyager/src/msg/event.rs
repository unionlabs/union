use std::fmt::Display;

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::ethereum::H256;

use crate::{
    chain::{ChainOf, HeightOf, LightClient},
    msg::{any_enum, identified},
};

any_enum! {
    #[any = AnyEvent(identified!(Event<L>))]
    pub enum Event<L: LightClient> {
        Ibc(IbcEvent<L>),
        Command(Command<L>),
    }
}

impl<L: LightClient> Display for Event<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Ibc(_) => write!(f, "Ibc"),
            Event::Command(cmd) => write!(f, "{cmd}"),
        }
    }
}

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct IbcEvent<L: LightClient> {
    pub block_hash: H256,
    pub height: HeightOf<ChainOf<L>>,
    pub event: unionlabs::events::IbcEvent<
        L::ClientId,
        L::ClientType,
        <L::Counterparty as LightClient>::ClientId,
    >,
}

#[derive(
    DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize, derive_more::Display,
)]
#[serde(bound(serialize = "", deserialize = ""))]
#[display(fmt = "Command::{}")]
pub enum Command<L: LightClient> {
    #[display(fmt = "UpdateClient({client_id}, {counterparty_client_id})")]
    UpdateClient {
        client_id: L::ClientId,
        counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
    },
}
