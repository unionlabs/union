use std::fmt::{Debug, Display};

use clap::builder::{StringValueParser, TypedValueParser};
use serde::{Deserialize, Serialize};

use crate::{
    ethereum::H256,
    ibc::core::{
        channel::channel::Channel, client::height::IsHeight,
        connection::connection_end::ConnectionEnd,
    },
    id::{ChannelId, ConnectionId},
    traits::{self, Chain},
};

/// `IbcPath` represents the path to a light client's ibc storage. The values stored at each path
/// are strongly typed, i.e. `connections/{connection_id}` always stores a [`ConnectionEnd`].
pub trait IbcPath<This: Chain, Counterparty>: Display + Clone + Sized {
    type Output: Debug + Clone + Serialize;
}

type PortId = String;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[serde(bound(
    serialize = "ClientId: Serialize",
    deserialize = "ClientId: for<'d> Deserialize<'d>",
))]
#[display(fmt = "clients/{client_id}/clientState")]
pub struct ClientStatePath<ClientId: traits::Id> {
    #[arg(
        value_parser = StringValueParser::new()
            .try_map(|x|
                x.parse::<ClientId>()
                    .map_err(|err| err.to_string())
            )
    )]
    pub client_id: ClientId,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty>
    for ClientStatePath<This::ClientId>
{
    type Output = Counterparty::SelfClientState;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[serde(bound(
    serialize = "ClientId: Serialize",
    deserialize = "ClientId: for<'d> Deserialize<'d>",
))]
#[display(fmt = "clients/{client_id}/consensusStates/{height}")]
pub struct ClientConsensusStatePath<ClientId: traits::Id, Height: IsHeight> {
    #[arg(
        value_parser = StringValueParser::new()
            .try_map(|x|
                x.parse::<ClientId>()
                    .map_err(|err| err.to_string())
            )
    )]
    pub client_id: ClientId,
    pub height: Height,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty>
    for ClientConsensusStatePath<This::ClientId, Counterparty::Height>
{
    type Output = Counterparty::SelfConsensusState;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[display(fmt = "connections/{connection_id}")]
pub struct ConnectionPath {
    pub connection_id: ConnectionId,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for ConnectionPath {
    type Output = ConnectionEnd<This::ClientId, Counterparty::ClientId, String>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[display(fmt = "channelEnds/ports/{port_id}/channels/{channel_id}")]
pub struct ChannelEndPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for ChannelEndPath {
    type Output = Channel;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[display(fmt = "commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct CommitmentPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: u64,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for CommitmentPath {
    type Output = H256;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[display(fmt = "acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct AcknowledgementPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: u64,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for AcknowledgementPath {
    type Output = H256;
}

#[derive(
    Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Subcommand,
)]
#[serde(bound(
    serialize = "ClientId: Serialize",
    deserialize = "ClientId: for<'d> Deserialize<'d>",
))]
pub enum Path<ClientId: traits::Id, Height: IsHeight> {
    #[display(fmt = "{_0}")]
    ClientStatePath(ClientStatePath<ClientId>),
    #[display(fmt = "{_0}")]
    ClientConsensusStatePath(ClientConsensusStatePath<ClientId, Height>),
    #[display(fmt = "{_0}")]
    ConnectionPath(ConnectionPath),
    #[display(fmt = "{_0}")]
    ChannelEndPath(ChannelEndPath),
    #[display(fmt = "{_0}")]
    CommitmentPath(CommitmentPath),
    #[display(fmt = "{_0}")]
    AcknowledgementPath(AcknowledgementPath),
}
