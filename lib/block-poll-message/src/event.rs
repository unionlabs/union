// use std::fmt::Display;

// use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
// use macros::apply;
// use queue_msg::{HandleEvent, QueueMsg, QueueMsgTypes};
// use serde::{Deserialize, Serialize};
// use unionlabs::{events::IbcEvent, hash::H256, traits::Chain, ClientType};

// use crate::{any_enum, AnyChainIdentified, BlockPollingTypes, ChainExt};

// #[apply(any_enum)]
// #[any = AnyEvent]
// pub enum Event<C: ChainExt> {}

// impl<C: ChainExt> Display for Event<C> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             // Event::IbcEvent(event) => write!(f, "IbcEvent({})", event.event.name()),
//         }
//     }
// }

// impl HandleEvent<BlockPollingTypes> for AnyChainIdentified<AnyEvent> {
//     fn handle(
//         self,
//         store: &<BlockPollingTypes as QueueMsgTypes>::Store,
//     ) -> QueueMsg<BlockPollingTypes> {
//         // dbg!(&self);

//         // let event = self;

//         // any_lc! {
//         //     |event| event.t.handle(store.get_chain(&event.chain_id))
//         // }
//         todo!()
//     }
// }
