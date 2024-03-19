pub mod contract;
pub mod error;
pub mod ibc;
pub mod msg;
pub mod protocol;
pub mod state;

#[global_allocator]
static ALLOC: dlmalloc::GlobalDlmalloc = dlmalloc::GlobalDlmalloc;
