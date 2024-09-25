use color_eyre::eyre::Report;
use tendermint_rpc::Error;

use super::api::IndexerError;

mod block_handle;
pub mod config;
mod context;
mod create_client_tracker;
mod fetcher_client;
mod postgres;
mod provider;

impl From<Error> for IndexerError {
    fn from(error: Error) -> Self {
        Self::ProviderError(Report::from(error))
    }
}
