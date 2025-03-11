use cosmwasm_std::{Addr, Uint256};
use cw_storage_plus::{Item, Map};
use ibc_union_spec::types::Packet;
use unionlabs::primitives::Bytes;

use crate::msg::Config;

/// Stores the configuration for the contract.
// TODO: Remove? Replace with IBC_HOST? Only the ibc_host field is read
pub const CONFIG: Item<Config> = Item::new("config");

/// The address of the token minter contract.
pub const TOKEN_MINTER: Item<Addr> = Item::new("token_minter");

/// Maps token denoms to their origin path, which is used to determine if a token
/// should be unwrapped when sent back to its origin chain.
pub const TOKEN_ORIGIN: Map<String, Uint256> = Map::new("token_origin");

/// Tracks the balance of tokens escrowed for each channel and denom combination.
/// This is used to ensure we don't unescrow more tokens than were originally escrowed.
pub const CHANNEL_BALANCE: Map<(u32, String), Uint256> = Map::new("channel_balance");

/// Temporarily stores the packet being executed to prevent reentrancy attacks.
/// This is cleared after execution is complete.
pub const EXECUTING_PACKET: Item<Packet> = Item::new("executing_packet");

pub const EXECUTING_PACKET_IS_BATCH: Item<()> = Item::new("executing_packet_is_batch");

/// Temporarily stores the acknowledgement from packet execution.
/// This is used to retrieve the acknowledgement in the reply handler.
pub const EXECUTION_ACK: Item<Bytes> = Item::new("execution_ack");

pub const BATCH_EXECUTION_ACKS: Item<Vec<Bytes>> = Item::new("batch_execution_acks");

/// Maps wrapped token denoms to their original token bytes representation.
/// This is used to determine the original token when unwrapping.
pub const HASH_TO_FOREIGN_TOKEN: Map<String, Bytes> = Map::new("hash_to_foreign_token");

/// Maps packet hash to parent packet for forwarded packets.
/// This is used to handle acknowledgements and timeouts for packets that are forwarded
/// through multiple chains. When a forwarded packet is acknowledged or times out,
/// we need to find the original packet that initiated the forward to properly
/// propagate the acknowledgement or timeout back to the source.
pub const IN_FLIGHT_PACKET: Map<Vec<u8>, Packet> = Map::new("in_flight_packet");
