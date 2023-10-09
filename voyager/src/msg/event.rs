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

#[derive(DebugNoBound, CloneNoBound, PartialEqNoBound, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub enum Command<L: LightClient> {
    UpdateClient {
        client_id: L::ClientId,
        counterparty_client_id: <L::Counterparty as LightClient>::ClientId,
    },
}
