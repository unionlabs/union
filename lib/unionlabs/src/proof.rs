use core::{
    fmt::{Debug, Display},
    num::NonZeroU64,
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
    traits::{self, Chain, ClientIdOf, HeightOf},
    MaybeArbitrary,
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
pub trait IbcPath<Hc: Chain, Tr: Chain>:
    Debug
    + Clone
    + PartialEq
    + Serialize
    + for<'de> Deserialize<'de>
    + Display
    + Sized
    + TryFrom<Path<ClientIdOf<Hc>, HeightOf<Tr>>, Error = Path<ClientIdOf<Hc>, HeightOf<Tr>>>
    + Into<Path<ClientIdOf<Hc>, HeightOf<Tr>>>
    + MaybeArbitrary
    + Send
    + Sync
{
    type Output: Debug + Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + MaybeArbitrary;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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

impl<Hc: Chain, Tr: Chain> IbcPath<Hc, Tr> for ClientStatePath<Hc::ClientId> {
    type Output = Hc::StoredClientState<Tr>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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

impl<Hc: Chain, Tr: Chain> IbcPath<Hc, Tr> for ClientConsensusStatePath<Hc::ClientId, Tr::Height> {
    type Output = Hc::StoredConsensusState<Tr>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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

impl<Hc: Chain, Tr: Chain> IbcPath<Hc, Tr> for ConnectionPath {
    type Output = ConnectionEnd<Hc::ClientId, Tr::ClientId, String>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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

impl<Hc: Chain, Tr: Chain> IbcPath<Hc, Tr> for ChannelEndPath {
    type Output = Channel;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[display(fmt = "commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct CommitmentPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
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

impl<Hc: Chain, Tr: Chain> IbcPath<Hc, Tr> for CommitmentPath {
    type Output = H256;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, derive_more::Display, clap::Args)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[display(fmt = "acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct AcknowledgementPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: NonZeroU64,
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

impl<Hc: Chain, Tr: Chain> IbcPath<Hc, Tr> for AcknowledgementPath {
    type Output = H256;
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    derive_more::Display,
    clap::Subcommand,
    enumorph::Enumorph,
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[serde(
    bound(
        serialize = "ClientId: Serialize",
        deserialize = "ClientId: for<'d> Deserialize<'d>",
    ),
    tag = "@type",
    content = "@value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
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
                sequence: 1.try_into().unwrap()
            })
        );
        assert_eq!(
            "acks/ports/port/channels/channel-0/sequences/1"
                .parse::<PathT>()
                .unwrap(),
            Path::AcknowledgementPath(AcknowledgementPath {
                port_id: "port".to_string().validate().unwrap(),
                channel_id: "channel-0".to_string().validate().unwrap(),
                sequence: 1.try_into().unwrap()
            })
        );
    }

    // TODO: Migrate these to fuzz targets
    // mod arbtest {
    //     use arbitrary::Arbitrary;

    //     use crate::{
    //         ibc::core::client::height::Height,
    //         id::ClientId,
    //         proof::Path,
    //         test_utils::{assert_json_roundtrip, assert_string_roundtrip},
    //     };

    //     #[test]
    //     pub(crate) fn parse() {
    //         arbtest::builder().budget_ms(4000).minimize().run(|u| {
    //             // we don't care if it succeeds (it probably won't), we just want to ensure it doesn't panic
    //             let _ = String::arbitrary(u)?.parse::<Path<ClientId, Height>>();
    //             Ok(())
    //         });
    //     }

    //     #[test]
    //     pub(crate) fn roundtrip() {
    //         let mut oks = 0;
    //         let mut errs = 0;
    //         arbtest::builder().budget_ms(4000).minimize().run(|u| {
    //             dbg!(u.len());
    //             let mut tries = 0;
    //             loop {
    //                 if u.is_empty() {
    //                     eprintln!("exhausted buffer");
    //                     break;
    //                 }

    //                 if let Ok(ok) = <Path<ClientId, Height>>::arbitrary(u) {
    //                     oks += 1;
    //                     assert_json_roundtrip(&ok);
    //                     assert_string_roundtrip(&ok);
    //                     break;
    //                 }

    //                 tries += 1;
    //                 if tries >= 1024 {
    //                     errs += 1;
    //                     break;
    //                 };
    //             }
    //             Ok(())
    //         });

    //         dbg!(oks, errs);
    //     }
    // }

    // const _: fn() = || {
    //     fn assert_impl_all<T: for<'a> Arbitrary<'a>>() {}

    //     assert_impl_all::<Path<ClientId, Height>>();
    // };
}
