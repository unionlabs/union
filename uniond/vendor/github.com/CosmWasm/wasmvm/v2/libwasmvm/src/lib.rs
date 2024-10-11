#![cfg_attr(feature = "backtraces", feature(backtrace))]
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::missing_safety_doc)]

mod api;
mod args;
mod cache;
mod calls;
mod db;
mod error;
mod gas_meter;
mod gas_report;
mod iterator;
mod memory;
mod querier;
mod storage;
mod test_utils;
mod tests;
mod version;
mod vtables;

// We only interact with this crate via `extern "C"` interfaces, not those public
// exports. There are no guarantees those exports are stable.
// We keep them here such that we can access them in the docs (`cargo doc`).
pub use api::{GoApi, GoApiVtable};
pub use cache::{cache_t, load_wasm};
pub use db::{db_t, Db, DbVtable};
pub use error::GoError;
pub use gas_report::GasReport;
pub use iterator::IteratorVtable;
pub use memory::{
    destroy_unmanaged_vector, new_unmanaged_vector, ByteSliceView, U8SliceView, UnmanagedVector,
};
pub use querier::{GoQuerier, QuerierVtable};
pub use storage::GoStorage;
pub use vtables::Vtable;
