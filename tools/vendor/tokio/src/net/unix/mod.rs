//! Unix domain socket utility types.

// This module does not currently provide any public API, but it was
// unintentionally defined as a public module. Hide it from the documentation
// instead of changing it to a private module to avoid breakage.
#[doc(hidden)]
pub mod datagram;

pub(crate) mod listener;

mod split;
pub use split::{ReadHalf, WriteHalf};

mod split_owned;
pub use split_owned::{OwnedReadHalf, OwnedWriteHalf, ReuniteError};

mod socketaddr;
pub use socketaddr::SocketAddr;

pub(crate) mod stream;
pub(crate) use stream::UnixStream;

mod ucred;
pub use ucred::UCred;
