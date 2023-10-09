use std::fmt::{Debug, Display};

use clap::builder::{StringValueParser, TypedValueParser};
use displaydoc::Display;
use futures::Future;
use serde::{Deserialize, Serialize};
use unionlabs::{
    ethereum::H256,
    ibc::core::{
        channel::channel::Channel,
        client::height::{Height, IsHeight},
        connection::connection_end::ConnectionEnd,
    },
    id::{ChannelId, ConnectionId},
    traits,
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

/// `IbcPath` represents the path to a light client's ibc storage. The values stored at each path
/// are strongly typed, i.e. `connections/{connection_id}` always stores a [`ConnectionEnd`].
pub trait IbcPath<This: Chain, Counterparty>: Display + Clone + Sized {
    type Output: Debug + Clone + Serialize;
}

type PortId = String;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, clap::Args)]
#[serde(bound(
    serialize = "ClientId: Serialize",
    deserialize = "ClientId: for<'d> Deserialize<'d>",
))]
#[displaydoc("clients/{client_id}/clientState")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, clap::Args)]
#[serde(bound(
    serialize = "ClientId: Serialize",
    deserialize = "ClientId: for<'d> Deserialize<'d>",
))]
#[displaydoc("clients/{client_id}/consensusStates/{height}")]
pub struct ClientConsensusStatePath<ClientId: traits::Id, Height: IsHeight> {
    #[arg(
        value_parser = StringValueParser::new()
            .try_map(|x|
                x.parse::<ClientId>()
                    .map_err(|err| err.to_string())
            )
    )]
    pub client_id: ClientId,
    #[arg(
        value_parser = StringValueParser::new()
            .try_map(|x|
                x.parse::<Height>()
                    .map_err(|err| err.to_string())
            )
    )]
    pub height: Height,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty>
    for ClientConsensusStatePath<This::ClientId, Counterparty::Height>
{
    type Output = Counterparty::SelfConsensusState;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, clap::Args)]
#[displaydoc("connections/{connection_id}")]
pub struct ConnectionPath {
    pub connection_id: ConnectionId,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for ConnectionPath {
    type Output = ConnectionEnd<This::ClientId, Counterparty::ClientId, String>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, clap::Args)]
#[displaydoc("channelEnds/ports/{port_id}/channels/{channel_id}")]
pub struct ChannelEndPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for ChannelEndPath {
    type Output = Channel;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, clap::Args)]
#[displaydoc("commitments/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct CommitmentPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: u64,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for CommitmentPath {
    type Output = H256;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, clap::Args)]
#[displaydoc("acks/ports/{port_id}/channels/{channel_id}/sequences/{sequence}")]
pub struct AcknowledgementPath {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub sequence: u64,
}

impl<This: Chain, Counterparty: Chain> IbcPath<This, Counterparty> for AcknowledgementPath {
    type Output = H256;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "ClientId: Serialize",
    deserialize = "ClientId: for<'d> Deserialize<'d>",
))]
pub enum Path<ClientId: traits::Id, Height: IsHeight> {
    ClientStatePath(ClientStatePath<ClientId>),
    ClientConsensusStatePath(ClientConsensusStatePath<ClientId, Height>),
    ConnectionPath(ConnectionPath),
    ChannelEndPath(ChannelEndPath),
    CommitmentPath(CommitmentPath),
    AcknowledgementPath(AcknowledgementPath),
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
