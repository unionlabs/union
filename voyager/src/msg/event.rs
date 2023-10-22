use std::fmt::Display;

use frame_support_procedural::{CloneNoBound, DebugNoBound, PartialEqNoBound};
use serde::{Deserialize, Serialize};
use unionlabs::ethereum::H256;

use crate::{
    chain::{ChainOf, HeightOf, LightClient, LightClientBase},
    msg::any_enum,
};

any_enum! {
    #[any = AnyEvent]
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
        <L::Counterparty as LightClientBase>::ClientId,
    >,
}

impl<L: LightClient> Display for IbcEvent<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use unionlabs::events::IbcEvent::*;

        match self.event {
            CreateClient(_) => write!(f, "Ibc::CreateClient"),
            UpdateClient(_) => write!(f, "Ibc::UpdateClient"),
            ClientMisbehaviour(_) => write!(f, "Ibc::ClientMisbehaviour"),
            SubmitEvidence(_) => write!(f, "Ibc::SubmitEvidence"),
            ConnectionOpenInit(_) => write!(f, "Ibc::ConnectionOpenInit"),
            ConnectionOpenTry(_) => write!(f, "Ibc::ConnectionOpenTry"),
            ConnectionOpenAck(_) => write!(f, "Ibc::ConnectionOpenAck"),
            ConnectionOpenConfirm(_) => write!(f, "Ibc::ConnectionOpenConfirm"),
            ChannelOpenInit(_) => write!(f, "Ibc::ChannelOpenInit"),
            ChannelOpenTry(_) => write!(f, "Ibc::ChannelOpenTry"),
            ChannelOpenAck(_) => write!(f, "Ibc::ChannelOpenAck"),
            ChannelOpenConfirm(_) => write!(f, "Ibc::ChannelOpenConfirm"),
            WriteAcknowledgement(_) => write!(f, "Ibc::WriteAcknowledgement"),
            RecvPacket(_) => write!(f, "Ibc::RecvPacket"),
            SendPacket(_) => write!(f, "Ibc::SendPacket"),
            AcknowledgePacket(_) => write!(f, "Ibc::AcknowledgePacket"),
            TimeoutPacket(_) => write!(f, "Ibc::TimeoutPacket"),
        }
    }
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
        counterparty_client_id: <L::Counterparty as LightClientBase>::ClientId,
    },
}
