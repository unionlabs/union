pub use crate::types::{
    channel::{Channel, ChannelState},
    connection::{Connection, ConnectionState},
    packet::Packet,
};

pub mod channel;
pub mod connection;
mod packet;

pub type ClientId = u32;
pub type ConnectionId = u32;
pub type ChannelId = u32;
