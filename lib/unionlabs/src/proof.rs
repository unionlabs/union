use std::{
    fmt::{Debug, Display},
    future::Future,
    str::FromStr,
};

use clap::builder::{StringValueParser, TypedValueParser};
use serde::{Deserialize, Serialize};

use crate::{
    hash::H256,
    ibc::core::{
        channel::channel::Channel, client::height::IsHeight,
        connection::connection_end::ConnectionEnd,
    },
    id::{ChannelId, ConnectionId, PortId},
    traits::{self, Chain},
};

fn eat_static_segment(s: Option<&str>, expecting: &'static str) -> Result<(), PathParseError> {
    match s {
        Some(segment) => {
            if segment == expecting {
                Ok(())
            } else {
                Err(PathParseError::InvalidStaticSegment {
                    expected: expecting,
                    found: segment.to_string(),
                })
            }
        }
        None => Err(PathParseError::MissingStaticSegment(expecting)),
    }
}

fn parse_segment<T>(s: Option<&str>) -> Result<T, PathParseError>
where
    T: FromStr,
    <T as FromStr>::Err: std::error::Error,
{
    match s {
        Some(segment) => segment
            .parse()
            .map_err(|e: <T as FromStr>::Err| PathParseError::Parse(e.to_string())),
        None => Err(PathParseError::MissingSegment),
    }
}

fn ensure_empty(mut s: impl Iterator) -> Result<(), PathParseError> {
    match s.next() {
        None => Ok(()),
        _ => Err(PathParseError::TooManySegments),
    }
}

#[derive(Debug)]
pub enum PathParseError {
    InvalidStaticSegment {
        expected: &'static str,
        found: String,
    },
    MissingStaticSegment(&'static str),
    MissingSegment,
    TooManySegments,
    // contains the stringified parse error
    Parse(String),
}

/// `IbcPath` represents the path to a light client's ibc storage. The values stored at each path
/// are strongly typed, i.e. `connections/{connection_id}` always stores a [`ConnectionEnd`].
pub trait IbcPath<This: Chain, Counterparty: Chain>: Display + Clone + Sized {
    type Output: Debug + Clone + Serialize;
}

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

impl<ClientId: traits::Id> FromStr for ClientStatePath<ClientId> {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('/');

        eat_static_segment(s.next(), "clients")?;
        let client_id = parse_segment(s.next())?;
        eat_static_segment(s.next(), "clientState")?;
        ensure_empty(s)?;

        Ok(Self { client_id })
    }
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

impl<ClientId: traits::Id, Height: IsHeight> FromStr
    for ClientConsensusStatePath<ClientId, Height>
{
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('/');

        eat_static_segment(s.next(), "clients")?;
        let client_id = parse_segment(s.next())?;
        eat_static_segment(s.next(), "consensusStates")?;
        let height = parse_segment(s.next())?;
        ensure_empty(s)?;

        Ok(Self { client_id, height })
    }
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

impl FromStr for ConnectionPath {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('/');

        eat_static_segment(s.next(), "connections")?;
        let connection_id = parse_segment(s.next())?;
        ensure_empty(s)?;

        Ok(Self { connection_id })
    }
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

impl FromStr for ChannelEndPath {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('/');

        eat_static_segment(s.next(), "channelEnds")?;
        eat_static_segment(s.next(), "ports")?;
        let port_id = parse_segment(s.next())?;
        eat_static_segment(s.next(), "channels")?;
        let channel_id = parse_segment(s.next())?;
        ensure_empty(s)?;

        Ok(Self {
            port_id,
            channel_id,
        })
    }
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

impl FromStr for CommitmentPath {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('/');

        eat_static_segment(s.next(), "commitments")?;
        eat_static_segment(s.next(), "ports")?;
        let port_id = parse_segment(s.next())?;
        eat_static_segment(s.next(), "channels")?;
        let channel_id = parse_segment(s.next())?;
        eat_static_segment(s.next(), "sequences")?;
        let sequence = parse_segment(s.next())?;
        ensure_empty(s)?;

        Ok(Self {
            port_id,
            channel_id,
            sequence,
        })
    }
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

impl FromStr for AcknowledgementPath {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.split('/');

        eat_static_segment(s.next(), "acks")?;
        eat_static_segment(s.next(), "ports")?;
        let port_id = parse_segment(s.next())?;
        eat_static_segment(s.next(), "channels")?;
        let channel_id = parse_segment(s.next())?;
        eat_static_segment(s.next(), "sequences")?;
        let sequence = parse_segment(s.next())?;
        ensure_empty(s)?;

        Ok(Self {
            port_id,
            channel_id,
            sequence,
        })
    }
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

impl<ClientId: traits::Id, Height: IsHeight> FromStr for Path<ClientId, Height> {
    type Err = PathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Self::ClientStatePath)
            .or_else(|_| s.parse().map(Self::ClientConsensusStatePath))
            .or_else(|_| s.parse().map(Self::ConnectionPath))
            .or_else(|_| s.parse().map(Self::ChannelEndPath))
            .or_else(|_| s.parse().map(Self::CommitmentPath))
            .or_else(|_| s.parse().map(Self::AcknowledgementPath))
    }
}

pub trait IbcStateRead<Counterparty: Chain, P: IbcPath<Self, Counterparty>>: Chain + Sized {
    fn proof(&self, path: P, at: Self::Height) -> impl Future<Output = Vec<u8>> + '_;
    fn state(&self, path: P, at: Self::Height) -> impl Future<Output = P::Output> + '_;
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ibc::core::client::height::Height, validated::ValidateT};

    #[test]
    fn parse_ibc_paths_from_str() {
        type PathT = Path<String, Height>;
        assert_eq!(
            "clients/08-wasm-0/clientState".parse::<PathT>().unwrap(),
            Path::ClientStatePath(ClientStatePath {
                client_id: "08-wasm-0".to_string()
            })
        );
        assert_eq!(
            "clients/08-wasm-0/consensusStates/0-1"
                .parse::<PathT>()
                .unwrap(),
            Path::ClientConsensusStatePath(ClientConsensusStatePath {
                client_id: "08-wasm-0".to_string(),
                height: Height {
                    revision_number: 0,
                    revision_height: 1
                }
            })
        );
        assert_eq!(
            "connections/connection-0".parse::<PathT>().unwrap(),
            Path::ConnectionPath(ConnectionPath {
                connection_id: "connection-0".to_string().validate().unwrap()
            })
        );
        assert_eq!(
            "channelEnds/ports/port/channels/channel-0"
                .parse::<PathT>()
                .unwrap(),
            Path::ChannelEndPath(ChannelEndPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap()
            })
        );
        assert_eq!(
            "commitments/ports/port/channels/channel-0/sequences/1"
                .parse::<PathT>()
                .unwrap(),
            Path::CommitmentPath(CommitmentPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap(),
                sequence: 1
            })
        );
        assert_eq!(
            "acks/ports/port/channels/channel-0/sequences/1"
                .parse::<PathT>()
                .unwrap(),
            Path::AcknowledgementPath(AcknowledgementPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap(),
                sequence: 1
            })
        );
    }
}
