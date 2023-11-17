// The async-traits lints are still a bit wonky, hence we go for manual implementations.
#![allow(clippy::manual_async_fn, clippy::needless_lifetimes)]
pub mod datastore;
pub mod eth;
pub use datastore::hasura;
