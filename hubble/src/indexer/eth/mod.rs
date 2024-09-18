use alloy::transports::{RpcError, TransportErrorKind};
use color_eyre::eyre::Report;

use super::api::IndexerError;

mod block_handle;
pub mod config;
mod context;
mod create_client_tracker;
mod fetcher_client;
mod postgres;

impl From<RpcError<TransportErrorKind>> for IndexerError {
    fn from(error: RpcError<TransportErrorKind>) -> Self {
        Self::ProviderError(Report::from(error))
    }
}
