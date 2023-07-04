extern crate alloc;

mod byte_list;
mod byte_vector;
mod context;
pub mod crypto;
mod error;
pub mod primitives;
mod rlp_node_codec;
mod serde;
mod types;
mod utils;
mod verify;

pub use byte_list::*;
pub use byte_vector::*;
pub use context::*;
pub use error::*;
pub use utils::*;
pub use verify::*;

pub use types::*;

pub const FINALIZED_ROOT_INDEX: usize = 105;
pub const FINALIZED_ROOT_SUBTREE_INDEX: usize = 41;
pub const FINALIZED_ROOT_DEPTH: usize = 6;
pub const EXECUTION_PAYLOAD_INDEX: usize = 25;
pub const EXECUTION_PAYLOAD_DEPTH: usize = 4;
pub const NEXT_SYNC_COMMITTEE_SUBTREE_INDEX: usize = 23;
pub const NEXT_SYNC_COMMITTEE_DEPTH: usize = 5;
